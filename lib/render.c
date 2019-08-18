#include <sensoreval.h>
#include <math.h>
#include <stdio.h>
#include <string.h>

#define ARRAY_SIZE(x) (sizeof(x) / sizeof((x)[0]))

static const struct sensoreval_hud_mode_handler *modemap[] = {
    [SENSOREVAL_HUD_MODE_BOOSTER] = &sensoreval_hud_handler_booster,
    [SENSOREVAL_HUD_MODE_SWINGBOAT] = &sensoreval_hud_handler_swingboat,
};

static const struct sensoreval_hud_mode_handler *mode2handler(enum sensoreval_hud_mode mode) {
    if (mode >= ARRAY_SIZE(modemap))
        return NULL;

    return modemap[mode];
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

    ctx->dpi = 141.21;
    ctx->spi = ctx->dpi;

    ctx->handler = mode2handler(cfg->hud.mode);

    if (ctx->handler && ctx->handler->init) {
        rc = ctx->handler->init(ctx);
        if (rc) {
            return -1;
        }
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
    const struct sensoreval_data *sd;

    sd = sensoreval_current_data(ctx);
    if (!sd) {
        fprintf(stderr, "no data\n");
        return -1;
    }

    cairo_save (cr);

    // clear
    cairo_save (cr);
    cairo_set_source_rgba (cr, 0, 0, 0, 0);
    cairo_set_operator (cr, CAIRO_OPERATOR_SOURCE);
    cairo_paint (cr);
    cairo_restore (cr);

    if (ctx->handler && ctx->handler->render_content) {
        rc = ctx->handler->render_content(ctx, cr);
        if (rc) {
            cairo_restore (cr);
            return rc;
        }
    }

    if (ctx->handler && ctx->handler->render_overlay) {
        rc = ctx->handler->render_overlay(ctx, cr);
        if (rc) {
            cairo_restore (cr);
            return rc;
        }
    }

    cairo_restore (cr);
    return 0;
}

int sensoreval_render_font(cairo_t *cr, PangoFontDescription *font, size_t *pw, size_t *ph,
    const char *fmt, ...)
{
    char buf[100];
    int rc;
    PangoLayout *layout;
    int w;
    int h;

    va_list args;
    va_start(args, fmt);
    rc = vsnprintf(buf, sizeof(buf), fmt, args);
    va_end (args);
    if (rc < 0 || (size_t)rc >= sizeof(buf)) {
        return -1;
    }

    layout = pango_cairo_create_layout(cr);
    if (!layout) {
        return -1;
    }

    pango_layout_set_font_description(layout, font);
    pango_layout_set_text(layout, buf, -1);

    cairo_set_source_rgba_u32(cr, 0xffffffff);
    pango_cairo_update_layout(cr, layout);
    pango_cairo_show_layout(cr, layout);

    cairo_set_line_width(cr, 1.0);
    cairo_set_source_rgba_u32(cr, 0x000000ff);
    pango_cairo_layout_path(cr, layout);
    cairo_stroke(cr);

    pango_layout_get_pixel_size(layout, &w, &h);
    if (pw)
        *pw = (size_t)w;
    if (ph)
        *ph = (size_t)h;

    g_object_unref(layout);

    return 0;
}
