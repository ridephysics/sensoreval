#include <sensoreval.h>
#include <math.h>
#include <stdio.h>

#define DPI 141.21
#define SPI DPI

static inline double dp2px(double dpi, double dp) {
    return dp * (dpi / 160.0);
}

static inline double px2dp(double dpi, double px) {
    return px / (dpi / 160.0);
}

int sensoreval_render(cairo_t *cr, const struct sensoreval_data *sd) {
    int rc;
    char txtbuf[50];
    cairo_text_extents_t extents;

    cairo_save (cr);
    cairo_set_source_rgba (cr, 0, 0, 0, 0);
    cairo_set_operator (cr, CAIRO_OPERATOR_SOURCE);
    cairo_paint (cr);
    cairo_restore (cr);

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
