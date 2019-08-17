#include <sensoreval.h>

static int hud_init(struct sensoreval_render_ctx *ctx) {
    (void)(ctx);
    return 0;
}

static int hud_render_content(const struct sensoreval_render_ctx *ctx, cairo_t *cr) {
    (void)(ctx);
    (void)(cr);
    return 0;
}

static int hud_render_overlay(const struct sensoreval_render_ctx *ctx, cairo_t *cr) {
    (void)(ctx);
    (void)(cr);
    return 0;
}

const struct sensoreval_hud_mode_handler sensoreval_hud_handler_booster = {
    .init = hud_init,
    .render_content = hud_render_content,
    .render_overlay = hud_render_overlay,
};
