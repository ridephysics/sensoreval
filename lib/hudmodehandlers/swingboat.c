#include <sensoreval.h>

struct sensordata_swingboat {
    double angle;
};

struct pdata {
    double ppm;

    struct sensordata_swingboat *sdarr;

    struct sensoreval_data *lores;
    size_t lores_sz;
};

static int hud_init(struct sensoreval_render_ctx *ctx) {
    struct pdata *pdata;
    size_t i;
    int rc;

    pdata = calloc(1, sizeof(*pdata));
    if (!pdata)
        return -1;

    pdata->sdarr = calloc(ctx->sdarrsz, sizeof(*(pdata->sdarr)));
    if (!pdata->sdarr) {
        free(pdata);
        return -1;
    }

    // downscale so we can draw smoother graphs
    rc = sensoreval_data_downscale(ctx->sdarr, ctx->sdarrsz, 1000000.0 / 30.0,
        &pdata->lores, &pdata->lores_sz);
    if (rc) {
        free(pdata);
        free(pdata->sdarr);
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

static int draw_graph(const struct sensoreval_render_ctx *ctx, cairo_t *cr,
    size_t graph_width, size_t graph_height,
    uint64_t dt)
{
    struct pdata *pdata = ctx->handlerctx;
    size_t sdid;
    size_t dataoff;
    cairo_pattern_t *pat;
    int rc;
    double maxval = 3.0;
    double redval = 5.0;

    if (ctx->datasrc != SENSOREVAL_RENDER_DATASRC_ARR) {
        return -1;
    }
    sdid = ctx->u.arr.id;

    rc = sensoreval_id_for_time(pdata->lores, pdata->lores_sz,
        0, ctx->sdarr[sdid].time, &sdid);
    if (rc) {
        return -1;
    }
    const struct sensoreval_data *sdnow = &pdata->lores[sdid];

    cairo_save(cr);
    cairo_rectangle(cr, 0, 0, graph_width, graph_height);
    cairo_clip(cr);

    // graph-line style
    cairo_set_line_width(cr, 6.0);
    pat = cairo_pattern_create_linear(0, graph_height - (graph_height / maxval * redval), 0, graph_height);
    cairo_pattern_add_color_stop_rgb(pat, 0.0, 1, 0, 0);
    cairo_pattern_add_color_stop_rgb(pat, 0.5, 1, 1, 0);
    cairo_pattern_add_color_stop_rgb(pat, 1.0, 0, 1, 0);
    cairo_set_source(cr, pat);

    uint64_t tstart = sdnow->time - dt;
    for (dataoff = sdid; dataoff>0; dataoff--) {
        const struct sensoreval_data *sd = &pdata->lores[dataoff];
        if (sd->time < tstart)
            break;

        double x = graph_width - ((((double)graph_width) / dt * (sdnow->time - sd->time)));
        double y = ((double)graph_height) - (graph_height / maxval * vec3_len(sd->accel));

        if (dataoff == sdid)
            cairo_move_to(cr, x, y);
        else
            cairo_line_to(cr, x, y);
    }

    cairo_stroke(cr);
    cairo_pattern_destroy(pat);
    cairo_restore(cr);

    // border
    cairo_set_source_rgba_u32(cr, 0x000000ff);
    cairo_set_line_width(cr, 3.0);
    cairo_move_to(cr, 0, 0);
    cairo_line_to(cr, 0, graph_height);
    cairo_line_to(cr, graph_width, graph_height);
    cairo_stroke(cr);

    return 0;
}

static int hud_render_overlay(const struct sensoreval_render_ctx *ctx, cairo_t *cr) {
    PangoFontDescription *font;
    const struct sensoreval_data *sd;
    size_t w;

    sd = sensoreval_current_data(ctx);
    if (!sd) {
        return -1;
    }

    font = pango_font_description_new();
    pango_font_description_set_family_static(font, "Archivo Black");
    pango_font_description_set_absolute_size(font, sp2px(100 * PANGO_SCALE));

    cairo_save(cr);
    cairo_translate(cr, dp2px(10), dp2px(10));

    sensoreval_render_font(cr, font, &w, NULL, "%.1fG", vec3_len(sd->accel));

    cairo_save(cr);
    cairo_translate(cr, w, 0);
    cairo_scale(cr, ctx->spi/160.0, ctx->spi/160.0);
    cairo_translate(cr, 10, 0);
    draw_graph(ctx, cr, 200, 100, 10000000);
    cairo_restore(cr);

    cairo_restore(cr);

    pango_font_description_free(font);
    return 0;
}

const struct sensoreval_hud_mode_handler sensoreval_hud_handler_swingboat = {
    .init = hud_init,
    .render_content = hud_render_content,
    .render_overlay = hud_render_overlay,
};
