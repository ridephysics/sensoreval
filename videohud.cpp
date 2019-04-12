#include "videohud.h"

#define DPI 141.21
#define SPI DPI

enum {
    TOP = 0x1,
    RIGHT = 0x2,
    BOTTOM = 0x4,
    LEFT = 0x8,
};

static inline double dp2px(double dpi, double dp) {
    return dp * (dpi / 160.0);
}

static inline double px2dp(double dpi, double px) {
    return px / (dpi / 160.0);
}

static inline QPointF pointf2px(double dpi, const QPointF& p) {
    return QPointF(dp2px(dpi, p.x()), dp2px(dpi, p.y()));
}

static inline QSizeF sizef2px(double dpi, const QSizeF& p) {
    return QSizeF(dp2px(dpi, p.width()), dp2px(dpi, p.height()));
}

static inline QRect qrect2dp(double dpi, const QRect& r) {
    return QRect(px2dp(dpi, r.x()), px2dp(dpi, r.y()), px2dp(dpi, r.width()), px2dp(dpi, r.height()));
}

static QSizeF drawTextShadowed(QPainter *painter, const QPointF& _pos, const QSizeF& _off,
    const QString& text, uint8_t zero = TOP | LEFT)
{
    QPointF pos = pointf2px(DPI, _pos);
    QSizeF off = sizef2px(DPI, _off);

    QFontMetrics fm(painter->font());
    QRect br = fm.boundingRect(text);
    int bw = fm.width(text);
    int bh = fm.height();

    if (zero & TOP)
        pos += QPointF(0, -br.y());

    if (zero & RIGHT)
        pos -= QPointF(bw, 0);

    painter->save();
    painter->setPen(Qt::black);
    painter->drawText(QPointF(pos.x() + off.width(), pos.y() + off.height()), text);
    painter->restore();

    painter->drawText(pos, text);

    return QSizeF(br.width(), br.height());
}

void VideoHUD::drawTextMeasurement(QPainter *painter, const QPointF& pos, const QString &value,
    const QString& unit, const QString& name)
{
    QSizeF tmpsz;
    QSizeF shadowsz(1, 1);

    painter->setPen(Qt::white);

    painter->setFont(this->font_small_bold);
    tmpsz = drawTextShadowed(painter, QPointF(pos.x(), pos.y()), shadowsz, unit, RIGHT|BOTTOM);

    painter->setFont(this->font_big_bold);
    tmpsz = drawTextShadowed(painter, QPointF(pos.x() - tmpsz.width() - 15, pos.y()), shadowsz, value, RIGHT|BOTTOM);

    painter->setFont(this->font_small);
    drawTextShadowed(painter, QPointF(pos.x(), pos.y()), shadowsz, name, RIGHT|TOP);
}

VideoHUD::VideoHUD(QQuickItem *parent)
    : QQuickPaintedItem(parent)
    , font_big_bold("Roboto", dp2px(SPI, 20))
    , font_small_bold("Roboto", dp2px(SPI, 10))
    , font_small("Roboto", dp2px(SPI, 12))
{
    font_big_bold.setBold(true);
    font_small_bold.setBold(true);
}

void VideoHUD::paint(QPainter *painter)
{
    QRect vp = qrect2dp(DPI, painter->viewport());

    painter->setPen(QColor(0xff, 0xff, 0xff, 0xff));

    this->drawTextMeasurement(painter, QPointF(vp.width() - 50, 100), "40.3", "METERS", "ALTITUDE");
    this->drawTextMeasurement(painter, QPointF(vp.width() - 50, 200), "20.0°", "CELSIUS", "TEMPERATURE");
}
