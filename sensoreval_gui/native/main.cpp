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
#include <assert.h>

extern "C" {
#include <global.h>
}

struct context
{
    int argc;
    char *arg0;
    char *argv[2];
    QGuiApplication *app;
    QQmlApplicationEngine *engine;
    QTimer *timer;
    const struct sensorevalgui_cfg *cfg;
};

static void init_rscs(void)
{
    Q_INIT_RESOURCE(qml);
}

extern "C" int sensorevalgui_native_create(struct context **pctx,
                                           const struct sensorevalgui_cfg *cfg)
{
    struct context *ctx = (struct context *)calloc(1, sizeof(*ctx));

    assert(ctx);

    init_rscs();
    QCoreApplication::setAttribute(Qt::AA_DisableHighDpiScaling);

    ctx->argc = 1;

    ctx->arg0 = strdup("native");
    assert(ctx->arg0);

    ctx->argv[0] = ctx->arg0;
    ctx->argv[1] = NULL;

    ctx->app = new QGuiApplication(ctx->argc, ctx->argv);
    assert(ctx->app);

    ctx->engine = new QQmlApplicationEngine();
    assert(ctx->engine);

    ctx->cfg = cfg;

    qmlRegisterType<QmlVideoHUD>("Main", 1, 0, "VideoHUD");
    qmlRegisterType<Orientation>("Main", 1, 0, "Orientation");
    sensorevalgui_native_set_orientation(ctx, (const double[]) { 1, 0, 0, 0 });

    QUrl url;
    if (cfg->videopath) {
        url = QUrl::fromLocalFile(QFileInfo(ctx->cfg->videopath).absoluteFilePath());
    } else {
        url = QUrl();
    }

    ctx->engine->rootContext()->setContextProperty("main_orientationEnabled",
                                                   ctx->cfg->orientation_enabled);
    ctx->engine->rootContext()->setContextProperty("main_videoPath", url);
    ctx->engine->rootContext()->setContextProperty("main_videoStartOffset",
                                                   (double)ctx->cfg->startoff);

    if (ctx->cfg->endoff != UINT64_MAX) {
        ctx->engine->rootContext()->setContextProperty("main_videoEndOffset",
                                                       (double)ctx->cfg->endoff);
    }

    ctx->engine->load(QUrl(QStringLiteral("qrc:/qml/main.qml")));
    assert(!ctx->engine->rootObjects().isEmpty());

    QObject *root = ctx->engine->rootObjects().first();
    Q_ASSERT(root);

    QmlVideoHUD *hud = root->findChild<QmlVideoHUD *>("hud");
    Q_ASSERT(hud);

    QObject *qmlplayer = root->findChild<QObject *>("player");
    Q_ASSERT(qmlplayer);
    QMediaPlayer *player = qvariant_cast<QMediaPlayer *>(qmlplayer->property("mediaObject"));
    Q_ASSERT(player);

    ctx->timer = new QTimer();
    QObject::connect(ctx->timer, &QTimer::timeout, [ctx, hud, player]() {
        if (ctx->cfg->videopath && ctx->cfg->set_ts) {
            ctx->cfg->set_ts(player->position() * 1000, ctx->cfg->pdata);
        }

        hud->update();
    });

    hud->setConfig(ctx->cfg);

    *pctx = ctx;
    return 0;
}

extern "C" int sensorevalgui_native_start(struct context *ctx)
{
    ctx->timer->start(ctx->cfg->timer_ms);
    return ctx->app->exec();
}

extern "C" void sensorevalgui_native_set_orientation(struct context *ctx, const double *raw)
{
    ctx->engine->rootContext()->setContextProperty("main_quaternion",
                                                   QQuaternion(raw[0], raw[1], raw[2], raw[3]));
}

extern "C" void sensorevalgui_native_destroy(struct context *ctx)
{
    if (ctx->timer) {
        delete ctx->timer;
        ctx->timer = nullptr;
    }

    delete ctx->engine;
    ctx->engine = nullptr;

    delete ctx->app;
    ctx->app = nullptr;

    ctx->cfg = nullptr;

    free(ctx->arg0);
    free(ctx);
}
