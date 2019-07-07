#ifndef VIDEOHUDRENDERER_H
#define VIDEOHUDRENDERER_H

#include <sensordata.h>
#include <QPainter>
#include <QFont>

class VideoHUDRenderer {
public:
    VideoHUDRenderer();
    void paintData(QPainter *painter, const SensorData& sd);

private:
    QFont font_big_bold;
    QFont font_small_bold;
    QFont font_small;

    void drawTextMeasurement(QPainter *painter, const QPointF& pos, const QString &value,
        const QString& unit, const QString& name);
};

#endif /* VIDEOHUDRENDERER_H */
