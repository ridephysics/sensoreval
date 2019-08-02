#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QFileInfo>
#include <QTimer>
#include <QMediaPlayer>
#include <QSocketNotifier>
#include <qmlnative/qmlvideohud.h>
#include <qmlnative/orientation.h>
#include <unistd.h>
#include <fcntl.h>
#include <memory>

static void set_sensordata(QQmlContext *ctx, const struct sensoreval_render_ctx *renderctx)
{
    const struct sensoreval_data *sd;
    QQuaternion q;

    if (renderctx) {
        sd = sensoreval_current_data(renderctx);

        if (sd) {
            q = QQuaternion(sd->quat[0], sd->quat[1], sd->quat[2], sd->quat[3]);
        }
    }

    ctx->setContextProperty("main_quaternion", q);
}


int main(int argc, char *argv[])
{
    int rc;
    QTimer *timer = nullptr;
    QSocketNotifier *notifier = nullptr;
    struct sensoreval_rd_ctx rdctx;
    struct sensoreval_data _sd;
    struct sensoreval_cfg *cfg = NULL;
    struct sensoreval_render_ctx renderctx;

    if (argc < 3 || argc > 4) {
        fprintf(stderr, "Usage: %s FILE LIVE [CONFIG]\n", argv[0]);
        return -1;
    }
    const char *videopath = argv[1];
    bool islive = !!atoi(argv[2]);
    const char *cfgpath = argc >= 4 ? argv[3] : NULL;

    rc = sensoreval_config_load(cfgpath, &cfg);
    if (rc) {
        fprintf(stderr, "can't load config\n");
        return -1;
    }

    sensoreval_rd_initctx(&rdctx, cfg);

    QCoreApplication::setAttribute(Qt::AA_EnableHighDpiScaling);
    QGuiApplication app(argc, argv);
    QQmlApplicationEngine engine;

    qmlRegisterType<QmlVideoHUD>("Main", 1, 0, "VideoHUD");
    qmlRegisterType<Orientation>("Main", 1, 0, "Orientation");

    engine.rootContext()->setContextProperty("main_videoPath", QUrl::fromLocalFile(QFileInfo(videopath).absoluteFilePath()));
    engine.rootContext()->setContextProperty("main_videoStartOffset", (double)cfg->video.startoff);
    engine.rootContext()->setContextProperty("main_videoEndOffset", (double)cfg->video.endoff);
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

    if (islive) {
        notifier = new QSocketNotifier(STDIN_FILENO, QSocketNotifier::Read, nullptr);

        rc = fcntl(notifier->socket(), F_GETFL, 0);
        Q_ASSERT(rc >= 0);

        rc |= O_NONBLOCK;

        rc = fcntl(notifier->socket(), F_SETFL, rc);
        Q_ASSERT(rc >= 0);

        rc = sensoreval_render_init(&renderctx, cfg, NULL, 0);
        Q_ASSERT(rc == 0);

        auto conn = std::make_shared<QMetaObject::Connection>();
        *conn = QObject::connect(notifier, &QSocketNotifier::activated, [conn, notifier, &rdctx, &_sd, &engine, hud, &renderctx](int fd) {
            enum sensoreval_rd_ret rdret;

            rdret = sensoreval_load_data_one(&rdctx, fd, &_sd);
            switch (rdret) {
            case SENSOREVAL_RD_RET_OK:
                sensoreval_render_set_data(&renderctx, &_sd);
                hud->update();

                set_sensordata(engine.rootContext(), &renderctx);
                break;

            case SENSOREVAL_RD_RET_ERR:
                fprintf(stderr, "read error\n");
                QObject::disconnect(*conn);
                break;

            case SENSOREVAL_RD_RET_WOULDBLOCK:
                break;

            case SENSOREVAL_RD_RET_EOF:
                fprintf(stderr, "EOF\n");
                QObject::disconnect(*conn);
                break;

            default:
                fprintf(stderr, "invalid ret: %d\n", rdret);
                Q_ASSERT(0);
            }
        });
    }
    else {
        struct sensoreval_data *sdarr;
        size_t sdarrsz;

        timer = new QTimer();

        rc = sensoreval_load_data(cfg, STDIN_FILENO, &sdarr, &sdarrsz);
        if (rc) {
            fprintf(stderr, "can't load sensordata\n");
            return -1;
        }

        rc = sensoreval_render_init(&renderctx, cfg, sdarr, sdarrsz);
        Q_ASSERT(rc == 0);

        QObject::connect(timer, &QTimer::timeout, [hud, &renderctx, player, sdarr, sdarrsz, &engine, cfg]() {
            sensoreval_render_set_ts(&renderctx, player->position()*1000 + cfg->data.startoff);
            hud->update();

            set_sensordata(engine.rootContext(), &renderctx);
        });
        timer->start(30);
    }

    hud->setRenderCtx(&renderctx);

    return app.exec();
}
