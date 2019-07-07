#ifndef SENSORDATA_H
#define SENSORDATA_H

#include <QQuaternion>

class SensorData {
public:
    // unit: g
    QVector3D accel;
    // unit: dps
    QVector3D gyro;
    // unit: uT
    QVector3D mag;

    QQuaternion quat;

    // unit: degrees celsius
    float temperature;

    // unit: hPa
    float pressure;

    float pressure_altitude_feet() const;
    float pressure_altitude() const;
};

Q_DECLARE_METATYPE(SensorData);

#endif
