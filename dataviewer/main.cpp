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

static void set_sensordata(QQmlContext *ctx, const struct sensoreval_ctx *sectx)
{
    int rc;
    double raw[4];
    QQuaternion q;

    if (sectx) {
        rc = sensoreval_get_quat(sectx, raw);
        if (rc == 0) {
            q = QQuaternion(raw[0], raw[1], raw[2], raw[3]);
        }
    }

    ctx->setContextProperty("main_quaternion", q);
}

int main(int argc, char *argv[])
{
    int rc;
    QTimer *timer = nullptr;
    QSocketNotifier *notifier = nullptr;
    struct sensoreval_ctx *sectx;
    char videopath[PATH_MAX];
    uint64_t startoff;
    uint64_t endoff;

    if (argc != 3) {
        fprintf(stderr, "Usage: %s LIVE CONFIG\n", argv[0]);
        return -1;
    }
    bool islive = !!atoi(argv[1]);
    const char *cfgpath = argv[2];

    sectx = sensoreval_create(cfgpath, islive);
    if (!sectx) {
        fprintf(stderr, "sensoreval_create failed\n");
        return -1;
    }

    rc = sensoreval_get_video_info(sectx, videopath, sizeof(videopath), &startoff, &endoff);
    if (rc) {
        fprintf(stderr, "sensoreval_get_video_info failed: %d\n", rc);
        return -1;
    }

    QCoreApplication::setAttribute(Qt::AA_EnableHighDpiScaling);
    QGuiApplication app(argc, argv);
    QQmlApplicationEngine engine;

    qmlRegisterType<QmlVideoHUD>("Main", 1, 0, "VideoHUD");
    qmlRegisterType<Orientation>("Main", 1, 0, "Orientation");

    engine.rootContext()->setContextProperty("main_videoPath", QUrl::fromLocalFile(QFileInfo(videopath).absoluteFilePath()));
    engine.rootContext()->setContextProperty("main_videoStartOffset", (double)startoff);
    engine.rootContext()->setContextProperty("main_videoEndOffset", (double)endoff);
    set_sensordata(engine.rootContext(), NULL);

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

    fprintf(stderr, "live: %d\n", islive);
    if (islive) {
        notifier = new QSocketNotifier(STDIN_FILENO, QSocketNotifier::Read, nullptr);

        rc = fcntl(notifier->socket(), F_GETFL, 0);
        Q_ASSERT(rc >= 0);

        rc |= O_NONBLOCK;

        rc = fcntl(notifier->socket(), F_SETFL, rc);
        Q_ASSERT(rc >= 0);

        auto conn = std::make_shared<QMetaObject::Connection>();
        *conn = QObject::connect(notifier, &QSocketNotifier::activated, [sectx, conn, notifier, &engine, hud](int fd) {
            (void)(fd);

            int notifyrc = sensoreval_notify_stdin(sectx);
            if (notifyrc < 0) {
                fprintf(stderr, "sensoreval_notify_stdin failed: %d\n", notifyrc);
                exit(1);
            }

            if (notifyrc > 0) {
                hud->update();
                set_sensordata(engine.rootContext(), sectx); 
            }
        });
    }
    else {
        timer = new QTimer();
        QObject::connect(timer, &QTimer::timeout, [sectx, hud, player, &engine]() {
            int settsrc = sensoreval_set_ts(sectx, player->position() * 1000);
            if (settsrc != 0) {
                fprintf(stderr, "can't set ts: %d\n", settsrc);
                return;
            }

            hud->update();
            set_sensordata(engine.rootContext(), sectx);
        });
        timer->start(30);
    }

    hud->setSensorEvalCtx(sectx);

    return app.exec();
}
