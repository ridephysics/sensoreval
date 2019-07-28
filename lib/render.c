#include <sensoreval.h>
#include <math.h>
#include <stdio.h>
#include <string.h>

static struct sensoreval_hud_mode_handler *mode2handler(enum sensoreval_hud_mode mode) {
    switch (mode) {
    case SENSOREVAL_HUD_MODE_BOOSTER:
        return &sensoreval_hud_handler_booster;
    default:
        return NULL;
    }
}

#define DPI 141.21
#define SPI DPI

static inline double dp2px(double dpi, double dp) {
    return dp * (dpi / 160.0);
}

static inline double px2dp(double dpi, double px) {
    return px / (dpi / 160.0);
}

int sensoreval_render_init(struct sensoreval_render_ctx *ctx,
    const struct sensoreval_cfg *cfg,
    const struct sensoreval_data *sdarr, size_t sdarrsz)
{
    int rc;

    memset(ctx, 0, sizeof(*ctx));

    if (!cfg)
        return -1;

    if (sdarrsz && !sdarr) {
        return -1;
    }

    ctx->cfg = cfg;
    ctx->sdarr = sdarr;
    ctx->sdarrsz = sdarrsz;
    ctx->datasrc = SENSOREVAL_RENDER_DATASRC_NONE;

    ctx->handler = mode2handler(cfg->hud.mode);

    rc = ctx->handler->init(ctx);
    if (rc) {
        return -1;
    }

    return 0;
}

int sensoreval_render_set_ts(struct sensoreval_render_ctx *ctx, uint64_t us) {
    size_t startid = 0;
    size_t id;
    int rc;

    if (!ctx->sdarr)
        return -1;

    if (ctx->datasrc == SENSOREVAL_RENDER_DATASRC_ARR && us >= ctx->u.arr.us) {
        startid = ctx->u.arr.id;
    }

    rc = sensoreval_id_for_time(ctx->sdarr, ctx->sdarrsz, startid, us, &id);
    if (rc) {
        return rc;
    }

    ctx->u.arr.us = us;
    ctx->u.arr.id = id;
    ctx->datasrc = SENSOREVAL_RENDER_DATASRC_ARR;

    return 0;
}

int sensoreval_render_set_data(struct sensoreval_render_ctx *ctx, const struct sensoreval_data *sd) {
    if (!sd)
        return -1;

    ctx->u.ext.data = sd;
    ctx->datasrc = SENSOREVAL_RENDER_DATASRC_EXT;

    return 0;
}

const struct sensoreval_data *sensoreval_current_data(const struct sensoreval_render_ctx *ctx) {
    switch (ctx->datasrc) {
    case SENSOREVAL_RENDER_DATASRC_ARR:
        return &ctx->sdarr[ctx->u.arr.id];

    case SENSOREVAL_RENDER_DATASRC_EXT:
        return ctx->u.ext.data;

    default:
        return NULL;
    }
}

int sensoreval_render(const struct sensoreval_render_ctx *ctx, cairo_t *cr) {
    int rc;
    char txtbuf[50];
    cairo_text_extents_t extents;
    const struct sensoreval_data *sd;

    sd = sensoreval_current_data(ctx);
    if (!sd) {
        fprintf(stderr, "no data\n");
        return -1;
    }

    cairo_save (cr);
    cairo_set_source_rgba (cr, 0, 0, 0, 0);
    cairo_set_operator (cr, CAIRO_OPERATOR_SOURCE);
    cairo_paint (cr);
    cairo_restore (cr);

    if (ctx->handler && ctx->handler->render_content) {
        rc = ctx->handler->render_content(ctx, cr);
        if (rc)
            return rc;
    }

    if (ctx->handler && ctx->handler->render_overlay) {
        rc = ctx->handler->render_overlay(ctx, cr);
        if (rc)
            return rc;
    }

    cairo_select_font_face (cr, "Sans", CAIRO_FONT_SLANT_NORMAL, CAIRO_FONT_WEIGHT_BOLD);
    cairo_set_font_size (cr, dp2px(SPI, 90));

    rc = snprintf(txtbuf, sizeof(txtbuf), "%d m", (int)sensoreval_data_altitude(sd));
    if (rc < 0 || (size_t)rc >= sizeof(txtbuf))
        return -1;
    cairo_text_extents (cr, txtbuf, &extents);
    cairo_move_to (cr, 0, extents.height);
    cairo_text_path (cr, txtbuf);
    cairo_set_source_rgb (cr, 1, 1, 1);
    cairo_fill_preserve (cr);
    cairo_set_source_rgb (cr, 0, 0, 0);
    cairo_set_line_width (cr, dp2px(SPI, 2));
    cairo_stroke (cr);

    return 0;
}
