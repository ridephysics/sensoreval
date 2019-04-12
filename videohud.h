#ifndef VIDEOHUD_H
#define VIDEOHUD_H

#include <QtQuick/QQuickPaintedItem>
#include <QPainter>

class VideoHUD : public QQuickPaintedItem
{
    Q_OBJECT
    
public:
    VideoHUD(QQuickItem *parent = 0);
    
    void paint(QPainter *painter);
};

#endif // VIDEOHUD_H