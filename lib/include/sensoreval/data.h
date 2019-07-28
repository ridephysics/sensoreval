#ifndef SENSOREVAL_DATA_H
#define SENSOREVAL_DATA_H

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

double sensoreval_data_altitude(const struct sensoreval_data *sd);
int sensoreval_id_for_time(const struct sensoreval_data *sdarr, size_t sdarrsz,
    size_t startid, uint64_t us, size_t *pid);

#endif /* SENSOREVAL_DATA_H */
