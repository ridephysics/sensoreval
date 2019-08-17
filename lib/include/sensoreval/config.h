#ifndef SENSOREVAL_CONFIG_H
#define SENSOREVAL_CONFIG_H

enum sensoreval_hud_mode {
    SENSOREVAL_HUD_MODE_NORMAL = 0,
    SENSOREVAL_HUD_MODE_BOOSTER,
    SENSOREVAL_HUD_MODE_SWINGBOAT,
};

enum sensoreval_orientation_mode {
    SENSOREVAL_ORIENTATION_MODE_NORMAL = 0,
};

struct sensoreval_cfg {
    struct {
        uint64_t startoff;
        uint64_t endoff;
    } video;

    struct {
        uint64_t startoff;
        double imu_orientation[4];
    } data;

    struct {
        enum sensoreval_hud_mode mode;
        double altitude_ground;

        union {
            struct {
                double radius;
            } booster;

            struct {
                double radius;
            } inversion;
        } u;
    } hud;

    struct {
        enum sensoreval_orientation_mode mode;
    } orientation;
};

int sensoreval_config_load(const char *path, struct sensoreval_cfg **cfg);
void sensoreval_config_dump(const struct sensoreval_cfg *cfg);

#endif /* SENSOREVAL_CONFIG_H */
