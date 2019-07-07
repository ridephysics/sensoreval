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

static void set_sensordata(QQmlContext *ctx, const struct sensoreval_data *sd) {
    QQuaternion q;

    if (sd) {
        q = QQuaternion(sd->quat[0], sd->quat[1], sd->quat[2], sd->quat[3]);
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

    sensoreval_rd_initctx(&rdctx);

    if (argc != 3) {
        fprintf(stderr, "Usage: %s FILE LIVE\n", argv[0]);
        return -1;
    }
    const char *videopath = argv[1];
    bool islive = !!atoi(argv[2]);

    QCoreApplication::setAttribute(Qt::AA_EnableHighDpiScaling);
    QGuiApplication app(argc, argv);
    QQmlApplicationEngine engine;

    qmlRegisterType<QmlVideoHUD>("Main", 1, 0, "VideoHUD");
    qmlRegisterType<Orientation>("Main", 1, 0, "Orientation");

    engine.rootContext()->setContextProperty("main_videoPath", QUrl::fromLocalFile(QFileInfo(videopath).absoluteFilePath()));
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

        auto conn = std::make_shared<QMetaObject::Connection>();
        *conn = QObject::connect(notifier, &QSocketNotifier::activated, [conn, notifier, &rdctx, &_sd, &engine, hud](int fd) {
            enum sensoreval_rd_ret rdret;

            rdret = sensoreval_load_data_one(&rdctx, fd, &_sd);
            switch (rdret) {
            case SENSOREVAL_RD_RET_OK:
                hud->setSensordata(&_sd);
                set_sensordata(engine.rootContext(), &_sd);
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

        rc = sensoreval_load_data(STDIN_FILENO, &sdarr, &sdarrsz);
        if (rc) {
            fprintf(stderr, "can't load sensordata\n");
            return -1;
        }
        fprintf(stderr, "got %zu samples\n", sdarrsz);

        QObject::connect(timer, &QTimer::timeout, [hud, player, sdarr, sdarrsz, &engine]() {
            struct sensoreval_data *sd = sensoreval_data_for_time(sdarr, sdarrsz, player->position()*1000);
            if (sd) {
                hud->setSensordata(sd);
                set_sensordata(engine.rootContext(), sd);
            }
        });
        timer->start(30);
    }

    return app.exec();
}
