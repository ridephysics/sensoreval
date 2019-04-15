#ifndef USFS_DEVICE_H
#define USFS_DEVICE_H

#include <QTimer>
#include <sensordata.h>

extern "C" {
#include <crossi2c/linux.h>
#include <em7180.h>
}

class USFSDevice : public QObject {
    Q_OBJECT

public:
    USFSDevice();
    ~USFSDevice();

    int start();

signals:
    void onData(const SensorData& sd);

private:
    bool m_initialized;
    QTimer m_timer;

    struct crossi2c_bus i2cbus;
    struct em7180 em7180;

    int init();
    void timeout();
};

#endif /* USFS_DEVICE_H */
