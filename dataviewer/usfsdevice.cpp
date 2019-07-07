#include "usfsdevice.h"
#include <QQuaternion>
#include <fcntl.h>
#include <unistd.h>

USFSDevice::USFSDevice() : m_notifier(fileno(stdin), QSocketNotifier::Read, this), m_bufpos(0) {
    int rc;

    rc = fcntl(m_notifier.socket(), F_GETFL, 0);
    Q_ASSERT(rc >= 0);

    rc |= O_NONBLOCK;

    rc = fcntl(m_notifier.socket(), F_SETFL, rc);
    Q_ASSERT(rc >= 0);

    connect(&m_notifier, SIGNAL(activated(int)), this, SLOT(onstdin(int)));
}

USFSDevice::~USFSDevice() {
}

#define rdvalue(dst) ({  \
    typeof(dst) __dst = (dst); \
    size_t __dstsz = sizeof(*__dst); \
    if (rdoff + __dstsz > sizeof(m_buf)) { \
        qDebug() << "tried to read too much from internal buffer"; \
        return; \
    } \
    memcpy(__dst, &m_buf[rdoff], __dstsz); \
    rdoff += __dstsz; \
})

void USFSDevice::onstdin(int fd) {
    ssize_t nbytes;
    size_t rdoff = 0;
    double d;
    uint64_t t_mpu;
    uint64_t t_bmp;
    size_t i;

again:
    nbytes = read(fd, m_buf + m_bufpos, sizeof(m_buf) - m_bufpos);
    if (nbytes < 0) {
        if (errno == EINTR)
            goto again;
        if (errno == EWOULDBLOCK || errno == EAGAIN)
            return;

        perror("read");
        disconnect(&m_notifier, SIGNAL(activated(int)), 0, 0);
        return;
    }
    if (nbytes == 0) {
        qDebug() << "stdin closed";
        disconnect(&m_notifier, SIGNAL(activated(int)), 0, 0);
        return;
    }

    m_bufpos += nbytes;
    if (m_bufpos < sizeof(m_buf))
        return;
    m_bufpos = 0;

    rdvalue(&t_mpu);
    for (i = 0; i < 3; i++) {
        rdvalue(&d);
        m_sensordata.accel[i] = d;
    }
    for (i = 0; i < 3; i++) {
        rdvalue(&d);
        m_sensordata.gyro[i] = d;
    }
    for (i = 0; i < 3; i++) {
        rdvalue(&d);
        m_sensordata.mag[i] = d;
    }

    rdvalue(&t_bmp);

    rdvalue(&d);
    m_sensordata.temperature = d;

    rdvalue(&d);
    m_sensordata.pressure = d;

    rdvalue(&d);
    m_sensordata.quat.setScalar(d);

    rdvalue(&d);
    m_sensordata.quat.setX(d);

    rdvalue(&d);
    m_sensordata.quat.setY(d);

    rdvalue(&d);
    m_sensordata.quat.setZ(d);

    Q_ASSERT(rdoff == sizeof(m_buf));

    emit onData(m_sensordata);
}
