#include <cairo/cairo.h>
#include <assert.h>
#include <sensoreval.h>
#include <stdio.h>
#include <unistd.h>

int main(void) {
    int rc;
    cairo_status_t cs;
    cairo_surface_t *s;
    cairo_t *cr;
    struct sensoreval_data *sdarr;
    size_t sdarrsz;

    rc = sensoreval_load_data(STDIN_FILENO, &sdarr, &sdarrsz);
    if (rc) {
        fprintf(stderr, "can't load sensordata\n");
        return -1;
    }
    fprintf(stderr, "got %zu samples\n", sdarrsz);

    s = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, 2720, 1520);
    assert(s);

    cr = cairo_create(s);
    assert(cr);
    cairo_set_antialias(cr, CAIRO_ANTIALIAS_BEST);

    rc = sensoreval_render(cr, &sdarr[0]);
    assert(!rc);

    cairo_surface_flush(s);
    cs = cairo_surface_write_to_png(s, "/tmp/out.png");
    assert(cs==CAIRO_STATUS_SUCCESS);

    return 0;
}
