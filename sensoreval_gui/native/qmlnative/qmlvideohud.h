#ifndef QMLVIDEOHUD_H
#define QMLVIDEOHUD_H

#include <QtQuick/QQuickPaintedItem>
#include <QPainter>
#include <QImage>

extern "C" {
#include <cairo/cairo.h>
#include <global.h>
}

class QmlVideoHUD : public QQuickPaintedItem
{
    Q_OBJECT

public:
    QmlVideoHUD(QQuickItem *parent = 0);

    void paint(QPainter *painter);

    void setConfig(const struct sensorevalgui_cfg *cfg);

private:
    const struct sensorevalgui_cfg *m_cfg;

    cairo_surface_t *m_cairo_surface;
    cairo_t *m_cr;
    QImage *m_qimg;

    void checkCreateCairo(QPainter *painter);
};

#endif // QMLVIDEOHUD_H
