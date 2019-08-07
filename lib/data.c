#include <sensoreval.h>
#include <stdio.h>
#include <math.h>
#include <inttypes.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

static inline double pressure_altitude_feet(double pressure) {
    return 145366.45 * (1.0 - pow((pressure/1013.25), 0.190284));
}

double sensoreval_data_altitude(const struct sensoreval_data *sd) {
    return pressure_altitude_feet(sd->pressure) * 0.3048;
}

int sensoreval_id_for_time(const struct sensoreval_data *sdarr, size_t sdarrsz,
    size_t startid, uint64_t us, size_t *pid)
{
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

    endtime = sdarr[sdarrsz - 1].time;

    if (us > endtime) {
        fprintf(stderr, "time %"PRIu64" is out of range\n", us);
        return -1;
    }

    for (i = startid; i < sdarrsz; i++) {
        const struct sensoreval_data *sd = &sdarr[i];

        if (sd->time > us) {
            *pid = i;
            return 0;
        }
    }

    return -1;
}

int sensoreval_data_downscale(const struct sensoreval_data *sdarr, size_t sdarrsz, uint64_t timeframe,
    struct sensoreval_data **plores, size_t *ploressz)
{
    size_t i;
    size_t j;
    size_t k;
    struct sensoreval_data *lores;
    size_t lores_sz;

    lores_sz = sdarr[sdarrsz - 1].time / timeframe;
    lores = malloc(sizeof(*lores) * lores_sz);
    if (!lores)
        return -1;

    // downscale
    for (i=0, j=0; i<lores_sz; i++) {
        size_t nsamples = 0;
        struct sensoreval_data *d = &lores[i];
        bool foundquat = false;
        size_t quatidx = j;

        memset(d, 0, sizeof(*d));
        d->time = i*timeframe + timeframe/2;

        for (; j<sdarrsz; j++) {
            if (sdarr[j].time >= i*timeframe)
                break;

            for (k=0; k<3; k++) {
                d->accel[k] += sdarr[j].accel[k];
                d->gyro[k] += sdarr[j].gyro[k];
                d->mag[k] += sdarr[j].mag[k];
            }

            d->temperature += sdarr[j].temperature;
            d->pressure += sdarr[j].pressure;

            if (!foundquat && sdarr[j].time >= d->time) {
                if (j && d->time - sdarr[j - 1].time < sdarr[j].time - d->time)
                    quatidx = j - 1;
                else
                    quatidx = j;

                foundquat = true;
            }

            nsamples++;
        }

        for (k=0; k<4; k++) {
            d->quat[k] = sdarr[quatidx].quat[k];
        }

        if (nsamples) {
            for (k=0; k<3; k++) {
                d->accel[k] /= (double)nsamples;
                d->gyro[k] /= (double)nsamples;
                d->mag[k] /= (double)nsamples;
            }

            d->temperature /= (double)nsamples;
            d->pressure /= (double)nsamples;
        }
        else if (i) {
            struct sensoreval_data *dprev = &lores[i - 1];

            for (k=0; k<3; k++) {
                d->accel[k] = dprev->accel[k];
                d->gyro[k] = dprev->gyro[k];
                d->mag[k] = dprev->mag[k];
            }

            d->temperature = dprev->temperature;
            d->pressure = dprev->pressure;
        }
    }

    *plores = lores;
    *ploressz = lores_sz;

    return 0;
}
