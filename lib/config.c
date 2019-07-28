#include <sensoreval.h>

#include <cyaml/cyaml.h>
#include <stdio.h>
#include <inttypes.h>
#include <string.h>
#include <stdlib.h>

#define typeof_member(s, m) typeof( ((s*)NULL)->m )

static const cyaml_strval_t hud_mode_strings[] = {
    { "normal",   SENSOREVAL_HUD_MODE_NORMAL },
    { "booster",   SENSOREVAL_HUD_MODE_BOOSTER },
};

static const cyaml_strval_t orientation_mode_strings[] = {
    { "normal",   SENSOREVAL_ORIENTATION_MODE_NORMAL },
};

static const cyaml_schema_field_t video_fields_schema[] = {
    CYAML_FIELD_UINT("startoff", CYAML_FLAG_DEFAULT, typeof_member(struct sensoreval_cfg, video), startoff),
    CYAML_FIELD_UINT("endoff", CYAML_FLAG_DEFAULT, typeof_member(struct sensoreval_cfg, video), endoff),
    CYAML_FIELD_END
};

static const cyaml_schema_field_t data_fields_schema[] = {
    CYAML_FIELD_UINT("startoff", CYAML_FLAG_DEFAULT, typeof_member(struct sensoreval_cfg, data), startoff),
    CYAML_FIELD_END
};

static const cyaml_schema_field_t booster_fields_schema[] = {
    CYAML_FIELD_FLOAT("radius", CYAML_FLAG_DEFAULT,
        typeof_member(struct sensoreval_cfg, hud.u.booster), radius),
    CYAML_FIELD_END
};

static const cyaml_schema_field_t hud_fields_schema[] = {
    CYAML_FIELD_ENUM("mode", CYAML_FLAG_DEFAULT, typeof_member(struct sensoreval_cfg, hud),
        mode, hud_mode_strings, CYAML_ARRAY_LEN(hud_mode_strings)),
    CYAML_FIELD_FLOAT("altitude_ground", CYAML_FLAG_DEFAULT,
        typeof_member(struct sensoreval_cfg, hud), altitude_ground),

    CYAML_FIELD_MAPPING("booster", CYAML_FLAG_OPTIONAL,
        typeof_member(struct sensoreval_cfg, hud), u.booster, booster_fields_schema),
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

    return 0;
}

void sensoreval_config_dump(const struct sensoreval_cfg *cfg) {
    fprintf(stderr, "video:\n");
    fprintf(stderr, "\tstartoff: %"PRIu64"\n", cfg->video.startoff);
    fprintf(stderr, "\tendoff: %"PRIu64"\n", cfg->video.endoff);

    fprintf(stderr, "data:\n");
    fprintf(stderr, "\tstartoff: %"PRIu64"\n", cfg->data.startoff);

    fprintf(stderr, "hud:\n");
    fprintf(stderr, "\tmode: %"PRId64"\n", (int64_t)cfg->hud.mode);
    fprintf(stderr, "\taltitude_ground: %f\n", cfg->hud.altitude_ground);

    switch (cfg->hud.mode) {
    case SENSOREVAL_HUD_MODE_BOOSTER:
        fprintf(stderr, "\tbooster:\n");
        fprintf(stderr, "\t\tradius: %f\n", cfg->hud.u.booster.radius);
        break;

    default:
        break;
    }

    fprintf(stderr, "orientation:\n");
    fprintf(stderr, "\tmode: %"PRId64"\n", (int64_t)cfg->orientation.mode);
}
