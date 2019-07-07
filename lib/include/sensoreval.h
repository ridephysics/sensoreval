#ifndef SENSOREVAL_H
#define SENSOREVAL_H

#include <cairo/cairo.h>
#include <stddef.h>
#include <stdint.h>

struct sensoreval_data {
    // unit: microseconds
    uint64_t time;

    // unit: g
    double accel[3];
    // unit: dps
    double gyro[3];
    // unit: uT
    double mag[3];

    double quat[4];

    // unit: degrees celsius
    double temperature;

    // unit: hPa
    double pressure;
};

struct sensoreval_rd_ctx {
    uint8_t buf[sizeof(uint64_t) + sizeof(double)*9 + sizeof(uint64_t) + sizeof(double)*2 + sizeof(double)*4];
    size_t bufpos;
};

enum sensoreval_rd_ret {
    SENSOREVAL_RD_RET_OK = 0,
    SENSOREVAL_RD_RET_ERR,
    SENSOREVAL_RD_RET_WOULDBLOCK,
    SENSOREVAL_RD_RET_EOF,
};

static inline void sensoreval_rd_initctx(struct sensoreval_rd_ctx *ctx) {
    ctx->bufpos = 0;
}

double sensoreval_data_altitude(const struct sensoreval_data *sd);
enum sensoreval_rd_ret sensoreval_load_data_one(struct sensoreval_rd_ctx *ctx, int fd, struct sensoreval_data *psd);
int sensoreval_load_data(int fd, struct sensoreval_data **psdarr, size_t *psdarrsz);
struct sensoreval_data * sensoreval_data_for_time(struct sensoreval_data *sdarr, size_t sdarrsz, uint64_t us);

int sensoreval_render(cairo_t *cr, const struct sensoreval_data *sd);

#endif /* SENSOREVAL_H */
