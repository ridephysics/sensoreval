#ifndef SENSOREVAL_H
#define SENSOREVAL_H

#include <cairo/cairo.h>

struct sensoreval_render_ctx;
struct sensoreval_datareader_ctx;

int sensoreval_render(struct sensoreval_render_ctx *ctx, cairo_t *cr);
int sensoreval_render_set_ts(struct sensoreval_render_ctx *ctx, uint64_t us);
int sensoreval_render_get_quat(const struct sensoreval_render_ctx *ctx, double quat[4]);
int sensoreval_render_get_video_info(const struct sensoreval_render_ctx *ctx,
    char *filename, unsigned int filename_sz,
    uint64_t *pstartoff, uint64_t *pendoff);

int sensoreval_notify_stdin(struct sensoreval_render_ctx *render, struct sensoreval_datareader_ctx *reader);

#endif /* SENSOREVAL_H */
