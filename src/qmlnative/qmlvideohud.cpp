#include "qmlvideohud.h"

#include <QQuickWindow>

QmlVideoHUD::QmlVideoHUD(QQuickItem *parent)
    : QQuickPaintedItem(parent)
{
}

void QmlVideoHUD::paint(QPainter *painter)
{
    m_hudrenderer.paintData(painter, m_sensordata);
}

void QmlVideoHUD::setSensordata(SensorData sensordata) {
    m_sensordata = sensordata;
    update();
}
