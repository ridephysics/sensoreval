#include "videohud.h"

VideoHUD::VideoHUD(QQuickItem *parent)
    : QQuickPaintedItem(parent)
{
    
}

void VideoHUD::paint(QPainter *painter)
{
    painter->setBrush(QColor(0xff, 0xff, 0xff, 128));
    
    QRect vp = painter->viewport();
    painter->drawRect(vp);
}
