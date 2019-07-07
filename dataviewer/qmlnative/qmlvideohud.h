#ifndef QMLVIDEOHUD_H
#define QMLVIDEOHUD_H

#include <QtQuick/QQuickPaintedItem>
#include <QPainter>
#include <QFont>
#include <QPointF>
#include <QString>
#include <sensordata.h>
#include <videohudrenderer.h>

class QmlVideoHUD : public QQuickPaintedItem
{
    Q_OBJECT
    Q_PROPERTY(SensorData sensordata READ sensordata WRITE setSensordata NOTIFY sensordataChanged)

public:
    QmlVideoHUD(QQuickItem *parent = 0);
    
    void paint(QPainter *painter);

    SensorData sensordata() const { return m_sensordata; }
    void setSensordata(SensorData sensordata);

signals:
    void sensordataChanged();

private:
    SensorData m_sensordata;
    VideoHUDRenderer m_hudrenderer;
};

#endif // QMLVIDEOHUD_H
