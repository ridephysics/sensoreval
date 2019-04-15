#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QFileInfo>
#include <qmlnative/videohud.h>
#include <qmlnative/orientation.h>
#include <devices/usfsdevice.h>

int main(int argc, char *argv[])
{
    if (argc != 2) {
        fprintf(stderr, "Usage: %s FILE\n", argv[0]);
        return -1;
    }

    QCoreApplication::setAttribute(Qt::AA_EnableHighDpiScaling);
    QGuiApplication app(argc, argv);
    QQmlApplicationEngine engine;

    engine.rootContext()->setContextProperty("videoPath", QUrl::fromLocalFile(QFileInfo(argv[1]).absoluteFilePath()));
    engine.rootContext()->setContextProperty("quaternion", QQuaternion(1,0,0,0));

    USFSDevice dev;
    QObject::connect(&dev, &USFSDevice::onData, [&engine](const SensorData& sd) {
        engine.rootContext()->setContextProperty("quaternion", sd.quat);
    });
    dev.start();

    qmlRegisterType<VideoHUD>("Main", 1, 0, "VideoHUD");
    qmlRegisterType<Orientation>("Main", 1, 0, "Orientation");

    engine.load(QUrl(QStringLiteral("qrc:/qml/main.qml")));
    if (engine.rootObjects().isEmpty())
        return -1;
    
    return app.exec();
}
