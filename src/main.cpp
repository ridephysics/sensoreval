#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QFileInfo>
#include <qmlnative/qmlvideohud.h>
#include <qmlnative/orientation.h>
#include <usfsdevice.h>

static void set_sensordata(QQmlContext *ctx, const SensorData& sd) {
    QVariant sd_variant;
    sd_variant.setValue(sd);

    ctx->setContextProperty("main_quaternion", sd.quat);
    ctx->setContextProperty("main_sensordata", sd_variant);
}

int main(int argc, char *argv[])
{
    if (argc != 2) {
        fprintf(stderr, "Usage: %s FILE\n", argv[0]);
        return -1;
    }

    QCoreApplication::setAttribute(Qt::AA_EnableHighDpiScaling);
    QGuiApplication app(argc, argv);
    QQmlApplicationEngine engine;

    qmlRegisterType<QmlVideoHUD>("Main", 1, 0, "VideoHUD");
    qmlRegisterType<Orientation>("Main", 1, 0, "Orientation");

    engine.rootContext()->setContextProperty("main_videoPath", QUrl::fromLocalFile(QFileInfo(argv[1]).absoluteFilePath()));
    set_sensordata(engine.rootContext(), SensorData());

    USFSDevice dev;
    QObject::connect(&dev, &USFSDevice::onData, [&engine](const SensorData& sd) {
        set_sensordata(engine.rootContext(), sd);
    });

    engine.load(QUrl(QStringLiteral("qrc:/qml/main.qml")));
    if (engine.rootObjects().isEmpty())
        return -1;
    
    return app.exec();
}
