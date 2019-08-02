#ifndef SENSOREVAL_PLOT_H
#define SENSOREVAL_PLOT_H

typedef void* (*sensoreval_plot_getdata_t)(size_t id, void *ctx);

int sensoreval_plot(size_t nplots, sensoreval_plot_getdata_t getdata, void *ctx);

#endif /* SENSOREVAL_PLOT_H */
