#include <sensoreval.h>

struct sensordata_swingboat {
    double angle;
};

struct pdata {
    double ppm;

    struct sensordata_swingboat *sdarr;
};

static int hud_init(struct sensoreval_render_ctx *ctx) {
    struct pdata *pdata;
    size_t i;

    pdata = calloc(1, sizeof(*pdata));
    if (!pdata)
        return -1;

    pdata->sdarr = calloc(ctx->sdarrsz, sizeof(*(pdata->sdarr)));
    if (!pdata->sdarr) {
        free(pdata);
        return -1;
    }

    pdata->ppm = 1;

    // calculate rotation
    for (i=0; i<ctx->sdarrsz; i++) {
        double final = 0;
        double vnorth[] = {0,1,0};
        double vnorthrot[3];

        quat_rotated_vec3(vnorthrot, ctx->sdarr[i].quat, vnorth);

        vnorthrot[0] = 0;
        vnorthrot[1] = sqrt(1.0 - pow(vnorthrot[2], 2));

        final = vec3_angle(vnorth, vnorthrot);

        if (vnorthrot[2] < 0)
            final *= -1;

        pdata->sdarr[i].angle = final;
    }

    ctx->handlerctx = pdata;
    return 0;
}

static void clip_bottom(cairo_t *cr, double h) {
    double ssz[2];
    double p[2] = {0, 0};
    double sz[2];

    cairo_surface_sz_user(cr, &ssz[0], &ssz[1]);
    cairo_device_to_user(cr, &p[0], &p[1]);

    sz[0] = ssz[0];
    sz[1] = -p[1] + h;

    cairo_rectangle(cr, p[0], p[1], sz[0], sz[1]);
    cairo_clip(cr);
}

static inline double x_for_height_angle(double height, double angle) {
    const double angle2 = deg2rad(180.0 - 90.0 - angle);
    return tri_opp2adj(height, angle2);
}

static int draw_swingboat(const struct sensoreval_render_ctx *ctx, cairo_t *cr) {
    struct pdata *pdata = ctx->handlerctx;
    const double frame_height = 11.0;
    const double frame_angle = 75.0;
    const double frame_thickness = 1.5;
    const uint32_t frame_color = 0xffffffe5;
    const uint32_t frame_top_color = 0x000000e5;
    const double gondola_height = 10.0;
    const double gondola_angle = 35.0;
    const double gondola_frame_thickness = 0.8;
    const double gondola_thickness = 2.0;
    const uint32_t gondola_color = 0xff8f00e5;

    const double x_frame_2 = x_for_height_angle(frame_height * 2.0, frame_angle/2.0);
    const double x_gondola = x_for_height_angle(gondola_height, gondola_angle/2.0);

    if (ctx->datasrc != SENSOREVAL_RENDER_DATASRC_ARR) {
        return -1;
    }

    double gondola_rotation = pdata->sdarr[ctx->u.arr.id].angle;
    switch(ctx->cfg->hud.u.swingboat.position) {
    case SENSOREVAL_SWINGBOAT_POS_BACK:
        gondola_rotation += deg2rad(gondola_angle / 2.0);
        break;

    case SENSOREVAL_SWINGBOAT_POS_MIDDLE:
        break;

    case SENSOREVAL_SWINGBOAT_POS_FRONT:
        gondola_rotation -= deg2rad(gondola_angle / 2.0);
        break;

    default:
        return -1;
    }

    cairo_save(cr);
    cairo_scale(cr, ctx->dpi/160.0, ctx->dpi/160.0);
    cairo_scale(cr, pdata->ppm, pdata->ppm);

    cairo_save(cr);
    cairo_rotate(cr, gondola_rotation);
    cairo_set_source_rgba_u32(cr, gondola_color);

    // gondola-frame
    cairo_set_line_width(cr, gondola_frame_thickness);
    cairo_move_to(cr, 0, 0);
    cairo_line_to(cr, x_gondola, gondola_height);
    cairo_move_to(cr, 0, 0);
    cairo_line_to(cr, -x_gondola, gondola_height);
    cairo_stroke(cr);

    // gondola
    cairo_set_operator(cr, CAIRO_OPERATOR_SOURCE);
    cairo_set_line_width(cr, gondola_thickness);
    cairo_arc(cr, 0, 0,
        sqrt(pow(x_gondola, 2) + pow(gondola_height, 2)) - (gondola_thickness/2.0),
        deg2rad(90 - gondola_angle/2.0),
        deg2rad(90 + gondola_angle/2.0)
    );
    cairo_stroke(cr);

    cairo_restore(cr);

    // frame
    cairo_save(cr);
    clip_bottom(cr, frame_height);
    cairo_set_source_rgba_u32(cr, frame_color);
    cairo_set_line_width(cr, frame_thickness);
    cairo_move_to(cr, 0, 0);
    cairo_line_to(cr, x_frame_2, frame_height * 2.0);
    cairo_move_to(cr, 0, 0);
    cairo_line_to(cr, -x_frame_2, frame_height * 2.0);
    cairo_stroke(cr);
    cairo_restore(cr);

    // top
    cairo_set_operator(cr, CAIRO_OPERATOR_SOURCE);
    cairo_set_source_rgba_u32(cr, frame_color);
    cairo_set_line_width(cr, 0.2);
    cairo_arc(cr, 0, 0, 1.0, 0, 2*M_PI);
    cairo_fill_preserve(cr);
    cairo_set_source_rgba_u32(cr, frame_top_color);
    cairo_stroke(cr);

    cairo_restore(cr);

    return 0;
}

static int hud_render_content(const struct sensoreval_render_ctx *ctx, cairo_t *cr) {
    struct pdata *pdata = ctx->handlerctx;
    double sw;
    double sh;
    int rc;

    cairo_surface_sz_user(cr, &sw, &sh);

    cairo_save(cr);
    pdata->ppm = 40;
    cairo_translate(cr, sw - (10.0 * pdata->ppm), sh - 11.5 * pdata->ppm);
    rc = draw_swingboat(ctx, cr);
    cairo_restore(cr);

    if (rc) {
        return -1;
    }

    return 0;
}

static int hud_render_overlay(const struct sensoreval_render_ctx *ctx, cairo_t *cr) {
    (void)(ctx);
    (void)(cr);
    return 0;
}

const struct sensoreval_hud_mode_handler sensoreval_hud_handler_swingboat = {
    .init = hud_init,
    .render_content = hud_render_content,
    .render_overlay = hud_render_overlay,
};
