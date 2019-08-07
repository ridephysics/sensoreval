#include <cairo/cairo.h>
#include <sensoreval.h>
#include <stdio.h>
#include <unistd.h>

int main(int argc, char **argv) {
    int rc;
    cairo_status_t cs;
    cairo_surface_t *s;
    cairo_t *cr;
    struct sensoreval_data *sdarr;
    size_t sdarrsz;
    struct sensoreval_cfg *cfg;
    struct sensoreval_render_ctx renderctx;

    if (argc != 2) {
        fprintf(stderr, "Usage: %s CONFIG\n", argv[0]);
        return -1;
    }
    const char *cfgpath = argv[1];

    rc = sensoreval_config_load(cfgpath, &cfg);
    if (rc) {
        fprintf(stderr, "can't load config\n");
        return -1;
    }

    sensoreval_config_dump(cfg);

    rc = sensoreval_load_data(cfg, STDIN_FILENO, &sdarr, &sdarrsz);
    if (rc) {
        fprintf(stderr, "can't load sensordata\n");
        return -1;
    }

    rc = sensoreval_render_init(&renderctx, cfg, sdarr, sdarrsz);
    if (rc) {
        fprintf(stderr, "can't init render context\n");
        return -1;
    }

    s = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, 2720, 1520);
    if (!s) {
        fprintf(stderr, "cairo_image_surface_create failed\n");
        return -1;
    }

    cr = cairo_create(s);
    if (!cr) {
        fprintf(stderr, "cairo_create failed\n");
        return -1;
    }
    cairo_set_antialias(cr, CAIRO_ANTIALIAS_BEST);

    rc = sensoreval_render_set_ts(&renderctx, 0);
    if (rc) {
        fprintf(stderr, "sensoreval_render_set_ts failed\n");
        return -1;
    }

    rc = sensoreval_render(&renderctx, cr);
    if (rc) {
        fprintf(stderr, "sensoreval_render failed\n");
        return -1;
    }

    cairo_surface_flush(s);

    cs = cairo_surface_write_to_png(s, "/tmp/out.png");
    if (cs != CAIRO_STATUS_SUCCESS) {
        fprintf(stderr, "cairo_surface_write_to_png failed\n");
        return -1;
    }

    return 0;
}
