#ifndef VIDEOHUD_H
#define VIDEOHUD_H

#include <QtQuick/QQuickPaintedItem>
#include <QPainter>
#include <QFont>
#include <QPointF>
#include <QString>

class VideoHUD : public QQuickPaintedItem
{
    Q_OBJECT
    
public:
    VideoHUD(QQuickItem *parent = 0);
    
    void paint(QPainter *painter);

private:
    QFont font_big_bold;
    QFont font_small_bold;
    QFont font_small;

    void drawTextMeasurement(QPainter *painter, const QPointF& pos, const QString &value,
        const QString& unit, const QString& name);
};

#endif // VIDEOHUD_H
