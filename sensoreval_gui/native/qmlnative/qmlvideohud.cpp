#include "qmlvideohud.h"

#include <QQuickWindow>

extern "C" {
#include <math.h>
}

QmlVideoHUD::QmlVideoHUD(QQuickItem *parent)
    : QQuickPaintedItem(parent),
      m_cfg(nullptr),
      m_cairo_surface(nullptr),
      m_cr(nullptr),
      m_qimg(nullptr)
{
}

void QmlVideoHUD::checkCreateCairo(QPainter *painter)
{
    QRect vp = painter->viewport();

    if (!m_cairo_surface
        || (vp.width() != cairo_image_surface_get_width(m_cairo_surface)
            || vp.height() != cairo_image_surface_get_height(m_cairo_surface))) {
        if (m_qimg) {
            delete m_qimg;
            m_qimg = nullptr;
        }

        if (m_cr) {
            cairo_destroy(m_cr);
            m_cr = nullptr;
        }

        if (m_cairo_surface) {
            cairo_surface_destroy(m_cairo_surface);
            m_cairo_surface = nullptr;
        }

        m_cairo_surface = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, vp.width(), vp.height());
        m_cr = cairo_create(m_cairo_surface);
        m_qimg = new QImage(cairo_image_surface_get_data(m_cairo_surface), vp.width(), vp.height(),
                            QImage::Format_ARGB32_Premultiplied);

        cairo_set_antialias(m_cr, CAIRO_ANTIALIAS_BEST);
    }
}

void QmlVideoHUD::paint(QPainter *painter)
{
    checkCreateCairo(painter);

    if (m_cfg && m_cfg->render) {
        m_cfg->render(m_cr, m_cfg->pdata);
    }

    cairo_surface_flush(m_cairo_surface);
    painter->drawImage(0, 0, *m_qimg);
}

void QmlVideoHUD::setConfig(const struct sensorevalgui_cfg *cfg)
{
    m_cfg = cfg;
    update();
}
