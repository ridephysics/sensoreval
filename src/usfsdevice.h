#ifndef USFS_DEVICE_H
#define USFS_DEVICE_H

#include <sensordata.h>
#include <QSocketNotifier>

class USFSDevice : public QObject {
    Q_OBJECT

public:
    USFSDevice();
    ~USFSDevice();

signals:
    void onData(const SensorData& sd);

private slots:
    void onstdin(int fd);

private:
    QSocketNotifier m_notifier;

    uint8_t m_buf[sizeof(uint64_t) + sizeof(double)*9 + sizeof(uint64_t) + sizeof(double)*2 + sizeof(double)*4];
    size_t m_bufpos;
    SensorData m_sensordata;

    void buf2data();
};

#endif /* USFS_DEVICE_H */
