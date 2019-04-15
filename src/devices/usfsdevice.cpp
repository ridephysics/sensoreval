#include "usfsdevice.h"
#include <math.h>
#include <QQuaternion>

extern "C" {
#define CROSSLOG_TAG "usfsdevice"
#include <crosslog.h>
}

USFSDevice::USFSDevice() {
    m_initialized = false;

    QObject::connect(&m_timer, &QTimer::timeout, [this]() { timeout(); });
}

USFSDevice::~USFSDevice() {
    int rc;

    if (m_initialized) {
        rc = em7180_destroy(&em7180);
        if (rc) {
            CROSSLOGW("can't destroy em7180 dev");
            return;
        }

        rc = crossi2c_destroy(&i2cbus);
        if (rc) {
            CROSSLOGW("can't destroy i2cbus");
            return;
        }
        m_initialized = false;
    }
}

int USFSDevice::init() {
    int rc;
    int ret = -1;
    uint8_t status = 0;

    rc = crossi2c_linux_create(&i2cbus, "/dev/i2c-9");
    if (rc) return -1;

    rc = em7180_create(&em7180, &i2cbus);
    if (rc) goto out_i2cbus_destroy;

    rc = em7180_init(&em7180);
    if (rc) goto out_em7180_destroy;

    em7180_set_algorithm(&em7180, 0x00);

    rc = em7180_get_sentral_status(&em7180, &status);
    if (rc) goto out_em7180_destroy;

    m_initialized = true;

    return 0;

out_em7180_destroy:
    rc = em7180_destroy(&em7180);
    if (rc) {
        CROSSLOGW("can't destroy em7180 dev");
    }

out_i2cbus_destroy:
    rc = crossi2c_destroy(&i2cbus);
    if (rc) {
        CROSSLOGW("can't destroy i2cbus");
    }

    if (ret) {
        CROSSLOGE("EXIT WITH ERROR");
    }

    return ret;
}

int USFSDevice::start() {
    int rc;

    if (!m_initialized) {
        rc = init();
        if (rc) return rc;
    }

    m_timer.start(30);

    return 0;
}

static float u32_to_f(uint32_t v) {
    union {
        uint32_t ui32;
        float f;
    } u;

    u.ui32 = v;

    return u.f;
}

void USFSDevice::timeout() {
    int rc;
    uint8_t event_status;
    uint8_t alg_status;
    uint8_t sensor_status;
    uint8_t error_reg;
    uint8_t data_raw[50];

    rc = em7180_get_event_status(&em7180, &event_status);
    if (rc) {
        CROSSLOGE("Unable to get event status (err %d)", rc);
        return;
    }

    if (event_status & EM7180_EVENT_ERROR) {
        rc = em7180_get_error_register(&em7180, &error_reg);
        if (rc) {
            CROSSLOGE("Unable to get error register (err %d)", rc);
            return;
        }

        em7180_print_error((enum em7180_error)error_reg);
        return;
    }

    rc = em7180_get_algorithm_status(&em7180, &alg_status);
    if (rc) {
        CROSSLOGE("Unable to get algorithm status (err %d)", rc);
        return;
    }

    rc = em7180_get_sensor_status(&em7180, &sensor_status);
    if (rc) {
        CROSSLOGE("Unable to get sensor status (err %d)", rc);
        return;
    }

    if (alg_status) {
        em7180_print_algorithm_status(alg_status);
    }

    if (sensor_status) {
        em7180_print_sensor_status(sensor_status);
    }

    if (!event_status) {
        return;
    }

    rc = em7180_get_data_all_raw(&em7180, data_raw);
    if (rc) {
        CROSSLOGE("Unable to get raw data (err %d)", rc);
        return;
    }

    SensorData sd;

    if (event_status & EM7180_EVENT_QUAT_RES) {
        uint32_t quat_raw[4];
        em7180_parse_data_quaternion(&data_raw[EM7180_RAWDATA_OFF_Q], quat_raw, NULL);

        sd.quat = QQuaternion(
            u32_to_f(quat_raw[3]),
            u32_to_f(quat_raw[0]),
            u32_to_f(quat_raw[1]),
            u32_to_f(quat_raw[2])
        );
    }

    emit onData(sd);
}
