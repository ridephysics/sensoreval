#ifndef SENSOREVAL_H
#define SENSOREVAL_H

#include <cairo/cairo.h>

struct sensoreval_ctx;

struct sensoreval_ctx *sensoreval_create(const char *path, bool islive);
void sensoreval_destroy(struct sensoreval_ctx *ctx);
int sensoreval_render(const struct sensoreval_ctx *ctx, cairo_t *cr);

int sensoreval_set_ts(struct sensoreval_ctx *ctx, uint64_t us);
int sensoreval_notify_stdin(struct sensoreval_ctx *ctx);

int sensoreval_get_quat(const struct sensoreval_ctx *ctx, double quat[4]);
int sensoreval_get_video_info(const struct sensoreval_ctx *ctx,
    char *filename, unsigned int filename_sz,
    uint64_t *pstartoff, uint64_t *pendoff);

#endif /* SENSOREVAL_H */
