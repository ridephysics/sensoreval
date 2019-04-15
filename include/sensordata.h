#ifndef SENSORDATA_H
#define SENSORDATA_H

#include <QQuaternion>

class SensorData {
public:
    QQuaternion quat;

    // unit: hPa
    float pressure;

    // unit: degrees celsius
    float temperature;

    // unit: g
    QVector3D acceleration;

    float pressure_altitude_feet() const;
    float pressure_altitude() const;
};

Q_DECLARE_METATYPE(SensorData);

#endif
