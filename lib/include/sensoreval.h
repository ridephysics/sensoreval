#ifndef SENSOREVAL_H
#define SENSOREVAL_H

#include <cairo/cairo.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

#include <sensoreval/data.h>
#include <sensoreval/config.h>
#include <sensoreval/readdata.h>
#include <sensoreval/render.h>
#include <sensoreval/math.h>
#include <sensoreval/plot.h>

#define sensoreval_unlikely(x) (__builtin_expect(!!(x), 0))
#define sensoreval_assert_se(expr) do { \
    if (sensoreval_unlikely(!(expr))) { \
        fprintf(stderr, "Assertion '%s' failed at %s:%u. Aborting.\n", #expr , __FILE__, __LINE__); \
        abort(); \
    } \
} while(0)

#endif /* SENSOREVAL_H */
