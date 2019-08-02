#include <sensoreval.h>
#include <stdio.h>
#include <math.h>
#include <inttypes.h>

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
