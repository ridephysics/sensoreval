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
