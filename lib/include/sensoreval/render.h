#ifndef SENSOREVAL_RENDER_H
#define SENSOREVAL_RENDER_H


enum sensoreval_render_datasrc {
    SENSOREVAL_RENDER_DATASRC_NONE,
    SENSOREVAL_RENDER_DATASRC_ARR,
    SENSOREVAL_RENDER_DATASRC_EXT,
};

struct sensoreval_render_ctx {
    const struct sensoreval_cfg *cfg;
    const struct sensoreval_data *sdarr;
    size_t sdarrsz;

    enum sensoreval_render_datasrc datasrc;
    union {
        struct {
            uint64_t us;
            size_t id;
        } arr;

        struct {
            const struct sensoreval_data *data;
        } ext;
    } u;
};

int sensoreval_render_init(struct sensoreval_render_ctx *ctx,
    const struct sensoreval_cfg *cfg,
    const struct sensoreval_data *sdarr, size_t sdarrsz);

int sensoreval_render_set_ts(struct sensoreval_render_ctx *ctx, uint64_t us);
int sensoreval_render_set_data(struct sensoreval_render_ctx *ctx, const struct sensoreval_data *sd);
const struct sensoreval_data *sensoreval_current_data(const struct sensoreval_render_ctx *ctx);

int sensoreval_render(const struct sensoreval_render_ctx *ctx, cairo_t *cr);

#endif /* SENSOREVAL_RENDER_H */
