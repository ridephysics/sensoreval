#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QFileInfo>
#include <QTimer>
#include <QMediaPlayer>
#include <QSocketNotifier>
#include <qmlnative/qmlvideohud.h>
#include <qmlnative/orientation.h>
#include <memory>

extern "C" {
#include <unistd.h>
#include <fcntl.h>
#include <sensoreval.h>
}

static void set_sensordata(QQmlContext *ctx, const struct sensoreval_render_ctx *render)
{
    int rc;
    double raw[4];
    QQuaternion q;

    if (render) {
        rc = sensoreval_render_get_quat(render, raw);
        if (rc == 0) {
            q = QQuaternion(raw[0], raw[1], raw[2], raw[3]);
        }
    }

    ctx->setContextProperty("main_quaternion", q);
}

static int dataviewer_main_real(int argc, char **argv, struct sensoreval_render_ctx *render, struct sensoreval_datareader_ctx *reader)
{
    int rc;
    QTimer *timer = nullptr;
    QSocketNotifier *notifier = nullptr;
    char videopath[PATH_MAX];
    uint64_t startoff;
    uint64_t endoff;

    Q_INIT_RESOURCE(qml);

    rc = sensoreval_render_get_video_info(render, videopath, sizeof(videopath), &startoff, &endoff);
    if (rc) {
        fprintf(stderr, "sensoreval_render_get_video_info failed: %d\n", rc);
        return -1;
    }

    QCoreApplication::setAttribute(Qt::AA_DisableHighDpiScaling);
    QGuiApplication app(argc, argv);
    QQmlApplicationEngine engine;

    qmlRegisterType<QmlVideoHUD>("Main", 1, 0, "VideoHUD");
    qmlRegisterType<Orientation>("Main", 1, 0, "Orientation");

    engine.rootContext()->setContextProperty("main_videoPath", QUrl::fromLocalFile(QFileInfo(videopath).absoluteFilePath()));
    engine.rootContext()->setContextProperty("main_videoStartOffset", (double)startoff);
    engine.rootContext()->setContextProperty("main_videoEndOffset", (double)endoff);
    set_sensordata(engine.rootContext(), nullptr);

    engine.load(QUrl(QStringLiteral("qrc:/qml/main.qml")));
    if (engine.rootObjects().isEmpty())
        return -1;

    QObject *root = engine.rootObjects().first();
    Q_ASSERT(root);

    QmlVideoHUD *hud = root->findChild<QmlVideoHUD*>("hud");
    Q_ASSERT(hud);

    QObject *qmlplayer = root->findChild<QObject*>("player");
    Q_ASSERT(qmlplayer);
    QMediaPlayer *player = qvariant_cast<QMediaPlayer *>(qmlplayer->property("mediaObject"));
    Q_ASSERT(player);

    if (reader) {
        notifier = new QSocketNotifier(STDIN_FILENO, QSocketNotifier::Read, nullptr);

        rc = fcntl(notifier->socket(), F_GETFL, 0);
        Q_ASSERT(rc >= 0);

        rc |= O_NONBLOCK;

        rc = fcntl(notifier->socket(), F_SETFL, rc);
        Q_ASSERT(rc >= 0);

        auto conn = std::make_shared<QMetaObject::Connection>();
        *conn = QObject::connect(notifier, &QSocketNotifier::activated, [render, reader, conn, notifier, &engine, hud](int fd) {
            (void)(fd);

            int notifyrc = sensoreval_notify_stdin(render, reader);
            if (notifyrc < 0) {
                fprintf(stderr, "sensoreval_notify_stdin failed: %d\n", notifyrc);
                exit(1);
            }

            if (notifyrc > 0) {
                hud->update();
                set_sensordata(engine.rootContext(), render);
            }
        });
    }
    else {
        timer = new QTimer();
        QObject::connect(timer, &QTimer::timeout, [render, hud, player, &engine]() {
            int settsrc = sensoreval_render_set_ts(render, player->position() * 1000);
            if (settsrc != 0) {
                fprintf(stderr, "can't set ts: %d\n", settsrc);
                return;
            }

            hud->update();
            set_sensordata(engine.rootContext(), render);
        });
        timer->start(30);
    }

    hud->setSensorEvalRenderCtx(render);

    return app.exec();
}

extern "C" int dataviewer_main(struct sensoreval_render_ctx *render, struct sensoreval_datareader_ctx *reader)
{
    char *arg0 = strdup("dataviewer");
    if (!arg0)
        return -1;
    char *argv[] = {arg0, NULL};

    int rc = dataviewer_main_real(1, argv, render, reader);
    free(arg0);
    return rc;
}
