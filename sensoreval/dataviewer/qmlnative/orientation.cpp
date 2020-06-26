#include "orientation.h"

#include <QtQuick/qquickwindow.h>
#include <QtGui/QOpenGLContext>

#include <GL/glu.h>
#include <math.h>
#include <QtMath>

#define ARRAY_SIZE(a)                                                                              \
    ((sizeof(a) / sizeof(*(a))) / static_cast<size_t>(!(sizeof(a) % sizeof(*(a)))))

static const GLdouble cam_pos[] = { 0.2, 0.2, 0 };
static const GLdouble cam_target[] = { 0, 0, -1 };
static const GLdouble cam_up[] = { 0, 1, 0 };

static const GLfloat color_red[] = { 0.8, 0, 0 };
static const GLfloat color_green[] = { 0, 0.8, 0 };
static const GLfloat color_blue[] = { 0, 0, 0.8 };
static const GLfloat color_gray[] = { 0.7, 0.7, 0.6 };
static const GLfloat color_white[] = { 1, 1, 1 };

static GLfloat delta = 0.01;
static const GLfloat vertices[][3] = {
    { -0.2, delta, 0 },  { 0.2, delta, 0 },  { 0, delta, -0.6 },
    { -0.2, -delta, 0 }, { 0.2, -delta, 0 }, { 0, -delta, -0.6 },
};
static const GLfloat *vertex_colors[] = {
    color_red, color_red, color_red, color_gray, color_gray, color_gray,
};

static size_t edges[][2] = {
    { 0, 1 }, { 0, 2 }, { 0, 3 }, { 1, 2 }, { 1, 4 }, { 2, 5 }, { 3, 4 }, { 3, 5 }, { 4, 5 },
};

static const GLfloat axes_endpts[][3] = {
    { -1, 0, 0 }, { 1, 0, 0 }, { 0, -1, 0 }, { 0, 1, 0 }, { 0, 0, -1 }, { 0, 0, 1 },
};

static void rotateVertex(const QQuaternion &q, GLfloat dst[3], const GLfloat src[3])
{
    QVector3D vin(src[0], -src[2], src[1]);
    QVector3D vout = q.rotatedVector(vin);

    dst[0] = vout.x();
    dst[1] = vout.z();
    dst[2] = -vout.y();
}

Orientation::Orientation() : m_renderer(nullptr), m_quat(1, 0, 0, 0)
{
    connect(this, &QQuickItem::windowChanged, this, &Orientation::handleWindowChanged);
}

void Orientation::handleWindowChanged(QQuickWindow *win)
{
    if (win) {
        connect(win, &QQuickWindow::beforeSynchronizing, this, &Orientation::sync,
                Qt::DirectConnection);
        connect(win, &QQuickWindow::sceneGraphInvalidated, this, &Orientation::cleanup,
                Qt::DirectConnection);
    }
}

void Orientation::sync()
{
    if (!m_renderer) {
        m_renderer = new OrientationRenderer();
        connect(window(), &QQuickWindow::afterRendering, m_renderer, &OrientationRenderer::paint,
                Qt::DirectConnection);
    }
    m_renderer->setViewportSize(window()->size() * window()->devicePixelRatio());
    m_renderer->setQuaternion(m_quat);
    m_renderer->setWindow(window());
}

void Orientation::cleanup()
{
    if (m_renderer) {
        delete m_renderer;
        m_renderer = nullptr;
    }
}

void Orientation::setQuat(QQuaternion quat)
{
    if (quat == m_quat)
        return;

    m_quat = quat;
    emit quatChanged();

    if (window())
        window()->update();
}

OrientationRenderer::~OrientationRenderer() {}

void OrientationRenderer::paint()
{
    m_window->resetOpenGLState();

    if (!m_initialized) {
        initializeOpenGLFunctions();
        m_initialized = true;
    }

    // viewport
    glViewport(0, 0, m_viewportSize.width(), m_viewportSize.height());

    // camera setup
    glMatrixMode(GL_PROJECTION);
    glLoadIdentity();

    gluPerspective(45, float(m_viewportSize.width()) / float(m_viewportSize.height()), 0.1, 50.0);
    glTranslatef(0, 0, -3);

    // general setup
    glClear(GL_DEPTH_BUFFER_BIT);
    glEnable(GL_DEPTH_TEST);

    // camera position
    glMatrixMode(GL_MODELVIEW);
    glLoadIdentity();
    gluLookAt(cam_pos[0], cam_pos[1], cam_pos[2], cam_target[0], cam_target[1], cam_target[2],
              cam_up[0], cam_up[1], cam_up[2]);

    // scene elements
    draw_pointer();
    draw_axes();

    // cleanup
    m_window->resetOpenGLState();
}

void OrientationRenderer::draw_cone()
{
    GLUquadricObj *quadratic;

    quadratic = gluNewQuadric();
    Q_ASSERT(quadratic);

    gluCylinder(quadratic, 0.05f, 0, 0.25f, 32, 32);

    gluDeleteQuadric(quadratic);
}

void OrientationRenderer::draw_pointer()
{
    GLfloat tmpvertex[3];

    glBegin(GL_TRIANGLES);
    for (size_t i = 0; i < ARRAY_SIZE(vertices); i++) {
        rotateVertex(m_quat, tmpvertex, vertices[i]);

        glColor3fv(vertex_colors[i]);
        glVertex3fv(tmpvertex);
    }
    glEnd();

    glBegin(GL_LINES);
    glColor3fv(color_white);
    for (size_t i = 0; i < ARRAY_SIZE(edges); i++) {
        rotateVertex(m_quat, tmpvertex, vertices[edges[i][0]]);
        glVertex3fv(tmpvertex);

        rotateVertex(m_quat, tmpvertex, vertices[edges[i][1]]);
        glVertex3fv(tmpvertex);
    }
    glEnd();
}

void OrientationRenderer::draw_axes()
{
    glBegin(GL_LINES);

    glColor3fv(color_red);
    glVertex3fv(axes_endpts[0]);
    glVertex3fv(axes_endpts[1]);

    glColor3fv(color_green);
    glVertex3fv(axes_endpts[2]);
    glVertex3fv(axes_endpts[3]);

    glColor3fv(color_blue);
    glVertex3fv(axes_endpts[4]);
    glVertex3fv(axes_endpts[5]);

    glEnd();

    glColor3fv(color_red);
    glPushMatrix();
    glTranslatef(1.0, 0, 0);
    glRotatef(90.0f, 0.0f, 1.0f, 0.0f);
    draw_cone();
    glPopMatrix();

    glColor3fv(color_green);
    glPushMatrix();
    glTranslatef(0, 1.0, 0);
    glRotatef(-90.0f, 1.0f, 0.0f, 0.0f);
    draw_cone();
    glPopMatrix();

    glColor3fv(color_blue);
    glPushMatrix();
    glTranslatef(0, 0, 1.0);
    draw_cone();
    glPopMatrix();
}
