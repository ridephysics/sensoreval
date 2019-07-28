#include <sensoreval.h>
#include <math.h>
#include <assert.h>
#include <string.h>
#include <stdio.h>
#include <errno.h>
#include <unistd.h>
#include <stdbool.h>
#include <stdlib.h>

static inline double pressure_altitude_feet(double pressure) {
    return 145366.45 * (1.0 - pow((pressure/1013.25), 0.190284));
}

double sensoreval_data_altitude(const struct sensoreval_data *sd) {
    return pressure_altitude_feet(sd->pressure) * 0.3048;
}

#define rdvalue(dst) ({  \
    typeof(dst) __dst = (dst); \
    size_t __dstsz = sizeof(*__dst); \
    if (rdoff + __dstsz > sizeof(ctx->buf)) { \
        fprintf(stderr, "tried to read too much from internal buffer\n"); \
        return SENSOREVAL_RD_RET_ERR; \
    } \
    memcpy(__dst, &ctx->buf[rdoff], __dstsz); \
    rdoff += __dstsz; \
})

enum sensoreval_rd_ret sensoreval_load_data_one(struct sensoreval_rd_ctx *ctx, int fd, struct sensoreval_data *psd) {
    ssize_t nbytes;
    size_t rdoff = 0;
    double d;
    uint64_t t_bmp;
    size_t i;

again:
    nbytes = read(fd, ctx->buf + ctx->bufpos, sizeof(ctx->buf) - ctx->bufpos);
    if (nbytes < 0) {
        if (errno == EINTR)
            goto again;
        if (errno == EWOULDBLOCK || errno == EAGAIN)
            return SENSOREVAL_RD_RET_WOULDBLOCK;

        perror("read");
        return SENSOREVAL_RD_RET_ERR;
    }
    if (nbytes == 0) {
        fprintf(stderr, "stdin closed\n");
        return SENSOREVAL_RD_RET_EOF;
    }

    ctx->bufpos += nbytes;
    if (ctx->bufpos < sizeof(ctx->buf)) {
        goto again;
    }
    ctx->bufpos = 0;

    rdvalue(&psd->time);
    for (i = 0; i < 3; i++) {
        rdvalue(&d);
        psd->accel[i] = d;
    }
    for (i = 0; i < 3; i++) {
        rdvalue(&d);
        psd->gyro[i] = d;
    }
    for (i = 0; i < 3; i++) {
        rdvalue(&d);
        psd->mag[i] = d;
    }

    rdvalue(&t_bmp);

    rdvalue(&d);
    psd->temperature = d;

    rdvalue(&d);
    psd->pressure = d;

    rdvalue(&d);
    psd->quat[0] = d;

    rdvalue(&d);
    psd->quat[1] = d;

    rdvalue(&d);
    psd->quat[2] = d;

    rdvalue(&d);
    psd->quat[3] = d;

    assert(rdoff == sizeof(ctx->buf));

    return SENSOREVAL_RD_RET_OK;
}

int sensoreval_load_data(int fd, struct sensoreval_data **psdarr, size_t *psdarrsz) {
    enum sensoreval_rd_ret rdret;
    struct sensoreval_rd_ctx ctx;
    struct sensoreval_data *sdarr = NULL;
    size_t sdarrsz = 0;
    size_t sdarrpos = 0;
    bool keep_going = true;

    sensoreval_rd_initctx(&ctx);

    while (keep_going) {
        bool ok = false;

        if (!sdarr || sdarrpos == sdarrsz) {
            void *nsdarr = realloc(sdarr, (sdarrsz + 1000) * sizeof(*sdarr));
            if (!nsdarr) {
                fprintf(stderr, "OOM\n");
                free(sdarr);
                return -1;
            }

            sdarr = nsdarr;
            sdarrsz += 1000;
        }

        rdret = sensoreval_load_data_one(&ctx, fd, &sdarr[sdarrpos]);
        switch (rdret) {
        case SENSOREVAL_RD_RET_OK:
            ok = true;
            break;

        case SENSOREVAL_RD_RET_ERR:
            goto err;

        case SENSOREVAL_RD_RET_WOULDBLOCK: {
            fd_set rfds;
            fd_set efds;

            FD_ZERO(&rfds);
            FD_SET(fd, &rfds);

            FD_ZERO(&efds);
            FD_SET(fd, &efds);

            select(fd + 1, &rfds, NULL, &efds, NULL);

            break;
        }

        case SENSOREVAL_RD_RET_EOF:
            keep_going = false;
            break;

        default:
            fprintf(stderr, "invalid ret: %d\n", rdret);
            goto err;
        }

        if (!ok)
            continue;

        sdarrpos++;
    }

    *psdarr = sdarr;
    *psdarrsz = sdarrpos;

    fprintf(stderr, "got %zu samples, start=%"PRIu64" end=%"PRIu64" duration=%fs\n", sdarrpos,
        sdarr[0].time, sdarr[sdarrpos-1].time,
        (sdarr[sdarrpos-1].time - sdarr[0].time) / 1000000.0
    );

    return 0;

err:
    if (sdarr)
        free(sdarr);
    return -1;
}

int sensoreval_id_for_time(const struct sensoreval_data *sdarr, size_t sdarrsz,
    size_t startid, uint64_t us, size_t *pid)
{
    uint64_t starttime;
    uint64_t endtime;
    size_t i;

    if (sdarrsz == 0) {
        fprintf(stderr, "no data available\n");
        return -1;
    }

    if (startid >= sdarrsz) {
        fprintf(stderr, "startid exceeds array size\n");
        return -1;
    }

    if (startid == 0)
        startid = 1;

    starttime = sdarr[0].time;
    endtime = sdarr[sdarrsz - 1].time;

    if (us > endtime - starttime) {
        fprintf(stderr, "time is out of range\n");
        return -1;
    }

    for (i = startid; i < sdarrsz; i++) {
        const struct sensoreval_data *sd = &sdarr[i];

        if (sd->time - starttime > us) {
            *pid = i;
            return 0;
        }
    }

    return -1;
}
