#ifndef SENSOREVAL_RENDER_H
#define SENSOREVAL_RENDER_H

struct sensoreval_hud_mode_handler;

enum sensoreval_render_datasrc {
    SENSOREVAL_RENDER_DATASRC_NONE,
    SENSOREVAL_RENDER_DATASRC_ARR,
    SENSOREVAL_RENDER_DATASRC_EXT,
};

struct sensoreval_render_ctx {
    const struct sensoreval_cfg *cfg;
    const struct sensoreval_data *sdarr;
    size_t sdarrsz;

    const struct sensoreval_hud_mode_handler *handler;
    void *handlerctx;

    double dpi;
    double spi;

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

struct sensoreval_hud_mode_handler {
    int (*init)(struct sensoreval_render_ctx *ctx);
    int (*render_content)(const struct sensoreval_render_ctx *ctx, cairo_t *cr);
    int (*render_overlay)(const struct sensoreval_render_ctx *ctx, cairo_t *cr);
};

extern const struct sensoreval_hud_mode_handler sensoreval_hud_handler_booster;
extern const struct sensoreval_hud_mode_handler sensoreval_hud_handler_swingboat;

static inline void cairo_surface_sz_user(cairo_t *cr, double *pw, double *ph) {
    cairo_surface_t *surface = cairo_get_target(cr);
    double sw = (double)cairo_image_surface_get_width(surface);
    double sh = (double)cairo_image_surface_get_height(surface);

    cairo_device_to_user_distance(cr, &sw, &sh);

    *pw = sw;
    *ph = sh;
}

static inline void cairo_set_source_rgba_u32(cairo_t *cr, uint32_t rgba) {
    uint8_t r = (rgba >> 24) & 0xff;
    uint8_t g = (rgba >> 16) & 0xff;
    uint8_t b = (rgba >> 8) & 0xff;
    uint8_t a = (rgba >> 0) & 0xff;

    double rf = 1.0 / 255.0 * r;
    double gf = 1.0 / 255.0 * g;
    double bf = 1.0 / 255.0 * b;
    double af = 1.0 / 255.0 * a;

    cairo_set_source_rgba(cr, rf, gf, bf, af);
}

#define dp2px(dp) ((dp) * (ctx->dpi / 160.0))
#define sp2px(dp) ((dp) * (ctx->spi / 160.0))

#endif /* SENSOREVAL_RENDER_H */
