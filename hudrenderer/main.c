#include <cairo/cairo.h>
#include <assert.h>
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

    rc = sensoreval_load_data(STDIN_FILENO, &sdarr, &sdarrsz);
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
    assert(s);

    cr = cairo_create(s);
    assert(cr);
    cairo_set_antialias(cr, CAIRO_ANTIALIAS_BEST);

    rc = sensoreval_render_set_ts(&renderctx, 0);
    assert(!rc);

    rc = sensoreval_render(&renderctx, cr);
    assert(!rc);

    cairo_surface_flush(s);
    cs = cairo_surface_write_to_png(s, "/tmp/out.png");
    assert(cs==CAIRO_STATUS_SUCCESS);

    return 0;
}
