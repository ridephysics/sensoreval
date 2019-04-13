#ifndef ORIENTATION_H
#define ORIENTATION_H

#include <QtQuick/QQuickItem>
#include <QtGui/QOpenGLFunctions_3_0>
#include <QQuaternion>

class OrientationRenderer : public QObject, protected QOpenGLFunctions_3_0
{
    Q_OBJECT
public:
    OrientationRenderer() : m_initialized(false), quat(1.0, 0.0, 0.0, 0.0) { }
    ~OrientationRenderer();

    void setViewportSize(const QSize &size) { m_viewportSize = size; }
    void setWindow(QQuickWindow *window) { m_window = window; }

public slots:
    void paint();

private:
    QSize m_viewportSize;
    bool m_initialized;
    QQuickWindow *m_window;
    QQuaternion quat;

    void draw_cone();
    void draw_pointer();
    void draw_axes();
};

class Orientation : public QQuickItem
{
    Q_OBJECT

public:
    Orientation();

public slots:
    void sync();
    void cleanup();

private slots:
    void handleWindowChanged(QQuickWindow *win);

private:
    OrientationRenderer *m_renderer;
};

#endif // ORIENTATION_H
