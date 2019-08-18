#include <sensoreval.h>

#include <cyaml/cyaml.h>
#include <stdio.h>
#include <inttypes.h>
#include <string.h>
#include <stdlib.h>

#define typeof_member(s, m) typeof( ((s*)NULL)->m )

#define CYAML_FIELD_FLOAT_ARRAY( \
		_key, _flags, _type, _index) \
{ \
	.key = _key, \
	.value = { \
		CYAML_VALUE_FLOAT(((_flags) & (~CYAML_FLAG_POINTER)), _type), \
	}, \
	.data_offset = (sizeof(_type) * (_index)) \
}

static const cyaml_strval_t hud_mode_strings[] = {
    { "normal",   SENSOREVAL_HUD_MODE_NORMAL },
    { "booster",   SENSOREVAL_HUD_MODE_BOOSTER },
    { "swingboat",   SENSOREVAL_HUD_MODE_SWINGBOAT },
};

static const cyaml_strval_t orientation_mode_strings[] = {
    { "normal",   SENSOREVAL_ORIENTATION_MODE_NORMAL },
};

static const cyaml_strval_t swingboat_pos_strings[] = {
    { "back",   SENSOREVAL_SWINGBOAT_POS_BACK },
    { "middle",   SENSOREVAL_SWINGBOAT_POS_MIDDLE },
    { "front",   SENSOREVAL_SWINGBOAT_POS_FRONT },
};

static const cyaml_schema_field_t video_fields_schema[] = {
    CYAML_FIELD_UINT("startoff", CYAML_FLAG_DEFAULT, typeof_member(struct sensoreval_cfg, video), startoff),
    CYAML_FIELD_UINT("endoff", CYAML_FLAG_DEFAULT, typeof_member(struct sensoreval_cfg, video), endoff),
    CYAML_FIELD_END
};

static const cyaml_schema_field_t quaternion_fields_schema[] = {
    CYAML_FIELD_FLOAT_ARRAY("w", CYAML_FLAG_DEFAULT, double, 0),
    CYAML_FIELD_FLOAT_ARRAY("x", CYAML_FLAG_DEFAULT, double, 1),
    CYAML_FIELD_FLOAT_ARRAY("y", CYAML_FLAG_DEFAULT, double, 2),
    CYAML_FIELD_FLOAT_ARRAY("z", CYAML_FLAG_DEFAULT, double, 3),
    CYAML_FIELD_END
};


static const cyaml_schema_field_t data_fields_schema[] = {
    CYAML_FIELD_UINT("startoff", CYAML_FLAG_DEFAULT, typeof_member(struct sensoreval_cfg, data), startoff),
    CYAML_FIELD_MAPPING("imu_orientation", CYAML_FLAG_OPTIONAL,
        typeof_member(struct sensoreval_cfg, data), imu_orientation, quaternion_fields_schema),
    CYAML_FIELD_END
};

static const cyaml_schema_field_t booster_fields_schema[] = {
    CYAML_FIELD_FLOAT("radius", CYAML_FLAG_DEFAULT,
        typeof_member(struct sensoreval_cfg, hud.u.booster), radius),
    CYAML_FIELD_END
};

static const cyaml_schema_field_t swingboat_fields_schema[] = {
    CYAML_FIELD_ENUM("position", CYAML_FLAG_DEFAULT, typeof_member(struct sensoreval_cfg, hud.u.swingboat),
        position, swingboat_pos_strings, CYAML_ARRAY_LEN(swingboat_pos_strings)),
    CYAML_FIELD_END
};

static const cyaml_schema_field_t hud_fields_schema[] = {
    CYAML_FIELD_ENUM("mode", CYAML_FLAG_DEFAULT, typeof_member(struct sensoreval_cfg, hud),
        mode, hud_mode_strings, CYAML_ARRAY_LEN(hud_mode_strings)),
    CYAML_FIELD_FLOAT("altitude_ground", CYAML_FLAG_DEFAULT,
        typeof_member(struct sensoreval_cfg, hud), altitude_ground),

    CYAML_FIELD_MAPPING("booster", CYAML_FLAG_OPTIONAL,
        typeof_member(struct sensoreval_cfg, hud), u.booster, booster_fields_schema),
    CYAML_FIELD_MAPPING("swingboat", CYAML_FLAG_OPTIONAL,
        typeof_member(struct sensoreval_cfg, hud), u.swingboat, swingboat_fields_schema),
    CYAML_FIELD_END
};

static const cyaml_schema_field_t orientation_fields_schema[] = {
    CYAML_FIELD_ENUM("mode", CYAML_FLAG_DEFAULT, typeof_member(struct sensoreval_cfg, orientation),
        mode, orientation_mode_strings, CYAML_ARRAY_LEN(orientation_mode_strings)),
    CYAML_FIELD_END
};

static const cyaml_schema_field_t top_mapping_schema[] = {
    CYAML_FIELD_MAPPING("video", CYAML_FLAG_DEFAULT, struct sensoreval_cfg, video, video_fields_schema),
    CYAML_FIELD_MAPPING("data", CYAML_FLAG_DEFAULT, struct sensoreval_cfg, data, data_fields_schema),
    CYAML_FIELD_MAPPING("hud", CYAML_FLAG_DEFAULT,
        struct sensoreval_cfg, hud, hud_fields_schema),
    CYAML_FIELD_MAPPING("orientation", CYAML_FLAG_DEFAULT,
        struct sensoreval_cfg, orientation, orientation_fields_schema),
    CYAML_FIELD_END
};

static const cyaml_schema_value_t top_schema = {
    CYAML_VALUE_MAPPING(CYAML_FLAG_POINTER, struct sensoreval_cfg, top_mapping_schema),
};


static const cyaml_config_t cyaml_config = {
    .log_level = CYAML_LOG_WARNING,
    .log_fn = cyaml_log,
    .mem_fn = cyaml_mem,
};

int sensoreval_config_load(const char *path, struct sensoreval_cfg **cfg) {
    cyaml_err_t cyerr;

    if (!cfg)
        return -1;

    if (!path) {
        *cfg = calloc(1, sizeof(*cfg));
        if (!(*cfg))
            return -1;

        return 0;
    }

    cyerr = cyaml_load_file(path, &cyaml_config, &top_schema, (cyaml_data_t**)cfg, NULL);
    if (cyerr != CYAML_OK) {
        fprintf(stderr, "ERROR: %s\n", cyaml_strerror(cyerr));
        return -1;
    }

    if (!(*cfg)->data.imu_orientation[0] && !(*cfg)->data.imu_orientation[1]
        && !(*cfg)->data.imu_orientation[2] && !(*cfg)->data.imu_orientation[3])
    {
        (*cfg)->data.imu_orientation[0] = 1;
    }

    return 0;
}

void sensoreval_config_dump(const struct sensoreval_cfg *cfg) {
    fprintf(stderr, "video:\n");
    fprintf(stderr, "\tstartoff: %"PRIu64"\n", cfg->video.startoff);
    fprintf(stderr, "\tendoff: %"PRIu64"\n", cfg->video.endoff);

    fprintf(stderr, "data:\n");
    fprintf(stderr, "\tstartoff: %"PRIu64"\n", cfg->data.startoff);
    fprintf(stderr, "\timu_orientation:\n");
    fprintf(stderr, "\t\tw: %f\n", cfg->data.imu_orientation[0]);
    fprintf(stderr, "\t\tx: %f\n", cfg->data.imu_orientation[1]);
    fprintf(stderr, "\t\ty: %f\n", cfg->data.imu_orientation[2]);
    fprintf(stderr, "\t\tz: %f\n", cfg->data.imu_orientation[3]);

    fprintf(stderr, "hud:\n");
    fprintf(stderr, "\tmode: %"PRId64"\n", (int64_t)cfg->hud.mode);
    fprintf(stderr, "\taltitude_ground: %f\n", cfg->hud.altitude_ground);

    switch (cfg->hud.mode) {
    case SENSOREVAL_HUD_MODE_BOOSTER:
        fprintf(stderr, "\tbooster:\n");
        fprintf(stderr, "\t\tradius: %f\n", cfg->hud.u.booster.radius);
        break;

    case SENSOREVAL_HUD_MODE_SWINGBOAT:
        fprintf(stderr, "\tswingboat:\n");
        fprintf(stderr, "\t\tposition: %d\n", cfg->hud.u.swingboat.position);
        break;

    default:
        break;
    }

    fprintf(stderr, "orientation:\n");
    fprintf(stderr, "\tmode: %"PRId64"\n", (int64_t)cfg->orientation.mode);
}
