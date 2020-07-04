#ifndef GLOBAL_H
#define GLOBAL_H

#include <stdint.h>
#include <stdbool.h>
#include <cairo/cairo.h>

struct context;

struct sensorevalgui_cfg
{
    uint64_t timer_ms;

    bool orientation_enabled;

    const char *videopath;
    uint64_t startoff;
    uint64_t endoff;

    void (*set_ts)(uint64_t, void *);
    void (*render)(cairo_t *, void *);

    void *pdata;
};

int sensorevalgui_native_create(struct context **pctx, const struct sensorevalgui_cfg *cfg);
int sensorevalgui_native_start(struct context *ctx);
void sensorevalgui_native_set_orientation(struct context *ctx, const double *raw);
void sensorevalgui_native_destroy(struct context *ctx);

#endif /* GLOBAL_H */