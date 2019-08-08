#include <sensoreval.h>
#include <errno.h>
#include <sys/random.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdbool.h>
#include <string.h>
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

double mean(const double *data, size_t len) {
    size_t i;
    double sum = 0.0;
    double meanval = 0.0;

    for (i=0; i<len; i++) {
        sum += data[i];
    }

    meanval = sum / len;

    return meanval;
}

double stddev(const double *data, size_t len) {
    size_t i;
    double sum = 0.0;
    double meanval = mean(data, len);

    for (i=0; i<len; i++) {
        sum += pow(data[i] - meanval, 2);
    }

    return sqrt(sum / len);
}

// Reference: https://ieeexplore.ieee.org/document/7884365
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

// Reference: https://towardsdatascience.com/linear-regression-using-least-squares-a4c3456e8570
int lls(const void *arr, size_t arrsz, size_t arrisz,
    size_t xoff, size_t yoff, size_t ynum, double *a, double *b)
{
    size_t i;
    double num = 0;
    double den = 0;
    double xmean = 0;
    double ymean = 0;

    if (!arr || !arrsz || !arrisz || !ynum || !a || !b)
        return -1;

    if (arrsz == 1) {
        *a = 0.0;
        *b = YVAL(0);
        return 0;
    }

    for (i=0; i<arrsz; i++) {
        xmean += XVAL(i);
        ymean += YVAL(i);
    }
    xmean /= (double)arrsz;
    ymean /= (double)arrsz;

    for (i=0; i<arrsz; i++) {
        num += (XVAL(i) - xmean) * (YVAL(i) - ymean);
        den += pow(XVAL(i) - xmean, 2);
    }

    *a = num / den;
    *b = ymean - (*a) * xmean;

    return 0;
}

// Source: https://stackoverflow.com/a/22640362
int thresholding(const void *arr, size_t arrsz, size_t arrisz,
    size_t yoff, size_t ynum, int8_t *signals,
    size_t lag, double threshold, double influence)
{
    size_t i;
    double *filteredY;
    double *avgFilter;
    double *stdFilter;

    if (!arr || !arrsz || !arrisz || !ynum || !signals)
        return -1;

    if (lag >= arrsz)
        return -1;

    filteredY = malloc(arrsz * sizeof(*filteredY));
    if (!filteredY)
        return -1;

    avgFilter = malloc(arrsz * sizeof(*avgFilter));
    if (!avgFilter) {
        free(filteredY);
        return -1;
    }

    stdFilter = malloc(arrsz * sizeof(*stdFilter));
    if (!stdFilter) {
        free(avgFilter);
        free(filteredY);
        return -1;
    }

    memset(signals, 0, arrsz * sizeof(*signals));

    for (i=0; i<arrsz; i++) {
        filteredY[i] = YVAL(i);
    }

    avgFilter[lag] = mean(filteredY, lag);
    stdFilter[lag] = stddev(filteredY, lag);

    for (i = lag + 1; i < arrsz; i++) {
        // if new value is a specified number of deviations away
        if (fabs(YVAL(i) - avgFilter[i-1]) > threshold * stdFilter[i-1]) {
            if (YVAL(i) > avgFilter[i-1]) {
                // positive signal
                signals[i] = 1;
            }
            else {
                // negative signal
                signals[i] = -1;
            }

            // make influence lower
            filteredY[i] = influence * YVAL(i) + (1.0 - influence) * filteredY[i-1];
        }
        else {
            // no signal
            signals[i] = 0;
        }

        // adjust the filters
        avgFilter[i] = mean(&filteredY[i - lag], i);
        stdFilter[i] = stddev(&filteredY[i - lag], i);
    }

    free(stdFilter);
    free(avgFilter);
    free(filteredY);

    return 0;
}

// Reference: https://ieeexplore.ieee.org/document/4566694
int pt_momentum(const void *arr, size_t arrsz, size_t arrisz,
    size_t xoff, size_t yoff, size_t ynum, int8_t *pt,
    double friction, double initial_min_momentum)
{
    size_t i;
    double momentum = 0.0;
    bool executing = false;
    size_t npt = 0;
    double ptvalue = 0.0;
    size_t ptvalue_id = 0;
    bool flip = false;

    if (!arr || !arrsz || !arrisz || !ynum || !pt)
        return -1;

    memset(pt, 0, arrsz * sizeof(*pt));

    for (i=0; i<arrsz; i++) {
        double dt = i ? ((double)(XVAL(i) - XVAL(i-1))) / 1000000.0 : 0;

        if (!executing) {
            if (fabs(momentum) < initial_min_momentum) {
                if (i && npt) {
                    momentum += (YVAL(i) - ptvalue) / dt;
                }
                else if (i) {
                    momentum += (YVAL(i) - YVAL(i - 1)) / dt;

                    if (YVAL(i) < YVAL(i - 1))
                        flip = false;
                    else
                        flip = true;
                }

                if (fabs(momentum) >= initial_min_momentum && i) {
                    executing = true;
                    momentum = fabs(momentum) * (1.0 - friction);
                }
            }
            else {
                executing = true;
            }
        }

        if (executing) {
            double value = YVAL(i);
            double valueprev = YVAL(i - 1);
            double friction_current = friction;

            if (flip) {
                value *= -1.0;
                valueprev *= -1.0;
            }

            if (value < ptvalue) {
                ptvalue = value;
                ptvalue_id = i;
            }

            double velocity = -(value - valueprev) / dt;
            if (velocity < 0)
                friction_current *= -1.0;

            momentum += (1.0 - friction_current) * velocity;

            if (momentum <= 0) {
                if (flip) {
                    flip = false;
                    pt[ptvalue_id] = 1;

                    if (value > ptvalue) {
                        ptvalue = value * -1.0;
                        ptvalue_id = i;
                    }
                }
                else {
                    flip = true;
                    pt[ptvalue_id] = -1;

                    if (-ptvalue > -value) {
                        ptvalue = -value;
                        ptvalue_id = i;
                    }
                }

                momentum = fabs(value - ptvalue) * (1.0 - friction_current);
                npt++;
                executing = false;
            }
        }
    }

    return 0;
}
