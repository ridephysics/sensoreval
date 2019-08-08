#include <sensoreval.h>
#include <errno.h>
#include <sys/random.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdbool.h>
#include <pcg_variants.h>

#define XVAL(i) (*((uint64_t*)(arr + (i) * arrisz + xoff)))
#define YVAL(i) ({ \
    double *__py = (double*) (arr + (i) * arrisz + yoff); \
    double __sum = 0; \
    double __ret; \
    if (ynum > 1) { \
        for (size_t __i = 0; __i < ynum; __i++) \
            __sum += pow(__py[__i], 2); \
        __ret = sqrt(__sum); \
    } \
    else { \
        __ret = __py[0]; \
    } \
    __ret; \
})
#define LMSVAL(k, i) LMS[N * ((k) - 1) + ((i) - 1)]

int ampd(const void *arr, size_t arrsz, size_t arrisz,
    size_t yoff, size_t ynum, bool peak, int8_t *result)
{
    size_t i;
    size_t k;
    size_t k2;
    double *LMS;
    size_t N;
    size_t L;
    size_t l = 0;
    double g_at_l;
    pcg32_random_t rng;

    if (!arr || !arrsz || !arrisz || !ynum || !result)
        return -1;

    pcg32_srandom_r(&rng, 0, 0);

    // init
    N = arrsz;
    L = ((size_t)ceil(((double)N)/2.0)) - 1;
    LMS = malloc(sizeof(double) * (L * N));
    if (!LMS) {
        fprintf(stderr, "out of memory for LMS\n");
        return -1;
    }

    // calculate LMS
    for (i=0; i<L*N; i++) {
        LMS[i] = ldexp(pcg32_random_r(&rng), -32) + 1.0;
    }

    for (k=1; k<=L; k++) {
        for (i=k+2; i<=N-k+1; i++) {
            double xim1 = YVAL(i-1);
            double ximkm1 = YVAL(i-k-1);
            double xipkm1 = YVAL(i+k-1);

            if (peak) {
                if ((xim1 > ximkm1) & (xim1 > xipkm1))
                    LMSVAL(k, i) = 0;
            }
            else {
                if ((xim1 < ximkm1) & (xim1 < xipkm1))
                    LMSVAL(k, i) = 0;
            }
        }
    }

    // reshape LMS
    for (k=1; k<=L; k++) {
        double sum = 0;

        for (i=1; i<=N; i++) {
            sum += LMSVAL(k, i);
        }

        if (k==1 || sum < g_at_l) {
            g_at_l = sum;
            l = k;
        }
    }

    // detect peaks
    for (i=1; i<=N; i++) {
        double sum_outer = 0;
        double sum_column = 0;

        for (k2=1; k2<=l; k2++) {
            sum_column += LMSVAL(k2, i);
        }

        for (k=1; k<=l; k++) {
            double tmp;

            tmp = LMSVAL(k, i) - (1.0/l) * sum_column;
            tmp = pow(tmp, 2);
            tmp = pow(tmp, 1.0/2.0);

            sum_outer += tmp;
        }

        double sigma = 1.0 / (l - 1.0) * sum_outer;
        if (fuzzy_is_null(sigma)) {
            if (peak)
                result[i] = 1;
            else
                result[i] = -1;
        }
    }

    return 0;
}
