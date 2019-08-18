#ifndef SENSOREVAL_MATH_H
#define SENSOREVAL_MATH_H

#include <math.h>
#include <stdbool.h>

static inline double deg2rad(double deg) {
    return deg * (M_PI / 180.0);
}

static inline double rad2deg(double rad) {
    return rad * 180.0 / M_PI;
}

static inline int fuzzy_is_null(double d)
{
    return fabs(d) <= 0.000000000001;
}

static inline double vec3_len(const double v[3])
{
    double len = v[0] * v[0] +
                 v[1] * v[1] +
                 v[2] * v[2];
    return sqrt(len);
}

static inline double vec3_dotproduct(const double v1[3], const double v2[3])
{
    return v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2];
}

static inline void vec3_crossproduct(double dst[3], const double v1[3], const double v2[3])
{
    double x = v1[1] * v2[2] - v1[2] * v2[1];
    double y = v1[2] * v2[0] - v1[0] * v2[2];
    double z = v1[0] * v2[1] - v1[1] * v2[0];

    dst[0] = x;
    dst[1] = y;
    dst[2] = z;
}

static inline void vec3_mul_num(double dst[3], const double v[3], double factor)
{
    dst[0] = v[0] * factor;
    dst[1] = v[1] * factor;
    dst[2] = v[2] * factor;
}

static inline void vec3_div_num(double dst[3], const double v[3], double divisor)
{
    dst[0] = v[0] / divisor;
    dst[1] = v[1] / divisor;
    dst[2] = v[2] / divisor;
}

static inline void vec3_sub(double dst[3], const double v1[3], const double v2[3]) {
    dst[0] = v1[0] - v2[0];
    dst[1] = v1[1] - v2[1];
    dst[2] = v1[2] - v2[2];
}

static inline void vec3_add(double dst[3], const double v1[3], const double v2[3])
{
    dst[0] = v1[0] + v2[0];
    dst[1] = v1[1] + v2[1];
    dst[2] = v1[2] + v2[2];
}

static inline void vec3_normalized(double dst[3], const double v[3])
{
    double len = v[0] * v[0] +
                 v[1] * v[1] +
                 v[2] * v[2];
    if (fuzzy_is_null(len - 1.0f)) {
        dst[0] = v[0];
        dst[1] = v[1];
        dst[2] = v[2];
    }
    else if (!fuzzy_is_null(len)) {
        double sqrtLen = sqrt(len);

        dst[0] = v[0] / sqrtLen;
        dst[1] = v[1] / sqrtLen;
        dst[2] = v[2] / sqrtLen;
    }
    else {
        dst[0] = 0;
        dst[1] = 0;
        dst[2] = 0;
    }
}

static inline double vec3_angle(const double a[3], const double b[3]) {
    return acos(vec3_dotproduct(a, b) / (vec3_len(a) * vec3_len(b)));
}

static inline void quat_mul(double dst[4], const double q1[4], const double q2[4]) {
    double yy = (q1[0] - q1[2]) * (q2[0] + q2[3]);
    double zz = (q1[0] + q1[2]) * (q2[0] - q2[3]);
    double ww = (q1[3] + q1[1]) * (q2[1] + q2[2]);
    double xx = ww + yy + zz;
    double qq = 0.5f * (xx + (q1[3] - q1[1]) * (q2[1] - q2[2]));

    double w = qq - ww + (q1[3] - q1[2]) * (q2[2] - q2[3]);
    double x = qq - xx + (q1[1] + q1[0]) * (q2[1] + q2[0]);
    double y = qq - yy + (q1[0] - q1[1]) * (q2[2] + q2[3]);
    double z = qq - zz + (q1[3] + q1[2]) * (q2[0] - q2[1]);

    dst[0] = w;
    dst[1] = x;
    dst[2] = y;
    dst[3] = z;
}

static inline void quat_conjugated(double dst[4], const double q[4]) {
    dst[0] = q[0];
    dst[1] = -q[1];
    dst[2] = -q[2];
    dst[3] = -q[3];
}

static inline void quat_from_vec3(double dst[4], double w, const double v[3]) {
    dst[0] = w;
    dst[1] = v[0];
    dst[2] = v[1];
    dst[3] = v[2];
}

static inline void quat_to_vec3(double dst[3], const double q[4]) {
    dst[0] = q[1];
    dst[1] = q[2];
    dst[2] = q[3];
}

static inline void quat_rotated_vec3(double dst[3], const double q[4], const double v[3]) {
    double qv[4];
    double q_mul_qv[4];
    double q_conjugated[4];
    double q_final[4];

    quat_from_vec3(qv, 0, v);
    quat_mul(q_mul_qv, q, qv);

    quat_conjugated(q_conjugated, q);
    quat_mul(q_final, q_mul_qv, q_conjugated);

    quat_to_vec3(dst, q_final);
}

static inline void quat_div_num(double dst[4], double q[4], double divisor) {
    dst[0] = q[0] / divisor;
    dst[1] = q[1] / divisor;
    dst[2] = q[2] / divisor;
    dst[3] = q[3] / divisor;
}

static inline void quat_normalized(double dst[4], double q[4]) {
    double len = q[1] * q[1] +
                 q[2] * q[2] +
                 q[3] * q[3] +
                 q[0] * q[0];

    if (fuzzy_is_null(len - 1.0f)) {
        dst[0] = q[0];
        dst[1] = q[1];
        dst[2] = q[2];
        dst[3] = q[3];
    }
    else if (!fuzzy_is_null(len)) {
        quat_div_num(dst, q, sqrt(len));
    }
    else {
        dst[0] = 0.0f;
        dst[1] = 0.0f;
        dst[2] = 0.0f;
        dst[3] = 0.0f;
    }
}

static inline double quat_lengthsquared(double q[4])
{
    return q[1] * q[1] + q[2] * q[2] + q[3] * q[3] + q[0] * q[0];
}

static inline void quat_from_rotationto(double dst[4], const double from[3], const double to[3])
{
    // Based on Stan Melax's article in Game Programming Gems

    double v0[3];
    double v1[3];

    vec3_normalized(v0, from);
    vec3_normalized(v1, to);

    double d = vec3_dotproduct(v0, v1) + 1.0f;

    // if dest vector is close to the inverse of source vector, ANY axis of rotation is valid
    if (fuzzy_is_null(d)) {
        double axis[3];
        double vtmp[3];

        vtmp[0] = 1.0f;
        vtmp[1] = 0.0f;
        vtmp[2] = 0.0f;
        vec3_crossproduct(axis, vtmp, v0);

        if (fuzzy_is_null(quat_lengthsquared(axis))) {
            vtmp[0] = 0.0f;
            vtmp[1] = 1.0f;
            vtmp[2] = 0.0f;

            vec3_crossproduct(axis, vtmp, v0);
        }
        vec3_normalized(axis, axis);

        // same as QQuaternion::fromAxisAndAngle(axis, 180.0f)
        dst[0] = 0.0f;
        dst[1] = axis[0];
        dst[2] = axis[1];
        dst[3] = axis[2];

        return;
    }

    d = sqrt(2.0f * d);
    double axis[3];

    vec3_crossproduct(axis, v0, v1);
    vec3_div_num(axis, axis, d);

    dst[0] = d * 0.5f;
    dst[1] = axis[0];
    dst[2] = axis[1];
    dst[3] = axis[2];

    quat_normalized(dst, dst);
}

static inline void quat_inverse(double dst[4], const double q[4]) {
    // Need some extra precision if the length is very small.
    double len = q[0] * q[0] +
                 q[1] * q[1] +
                 q[2] * q[2] +
                 q[3] * q[3];

    if (!fuzzy_is_null(len)) {
        dst[0] =  q[0] / len;
        dst[1] = -q[1] / len;
        dst[2] = -q[2] / len;
        dst[3] = -q[3] / len;
        return;
    }

    dst[0] = 0.0f;
    dst[1] = 0.0f;
    dst[2] = 0.0f;
    dst[3] = 0.0f;
}

static inline void quat_from_axis_and_angle(double dst[4], const double axis[3], double angle)
{
    double ax[3];

    // Algorithm from:
    // http://www.j3d.org/matrix_faq/matrfaq_latest.html#Q56
    // We normalize the result just in case the values are close
    // to zero, as suggested in the above FAQ.
    double a = deg2rad(angle / 2.0f);
    double s = sin(a);
    double c = cos(a);
    vec3_normalized(ax, axis);

    dst[0] = c;
    dst[1] = ax[0] * s;
    dst[2] = ax[1] * s;
    dst[3] = ax[2] * s;

    quat_normalized(dst, dst);
}

static inline double tri_opp2adj(double opp, double angle) {
    return opp / tan(angle);
}

static inline double tri_hyp2opp(double hyp, double angle) {
    return sin(angle) * hyp;
}

static inline double tri_hyp2adj(double hyp, double angle) {
    return cos(angle) * hyp;
}

double mean(const double *data, size_t len);
double stddev(const double *data, size_t len);

int ampd(const void *arr, size_t arrsz, size_t arrisz,
    size_t voff, size_t vnum, bool peak, int8_t *result);
#define AMPD(arr, arrsz, member, vnum, peak, result) ampd((arr), (arrsz), sizeof(*(arr)), \
    offsetof(typeof(*(arr)), member), (vnum), (peak), (result))

int lls(const void *arr, size_t arrsz, size_t arrisz,
    size_t xoff, size_t yoff, size_t ynum, double *a, double *b);
#define LLS(arr, arrsz, xmem, ymem, ynum, a, b) lls((arr), (arrsz), sizeof(*(arr)), \
    offsetof(typeof(*(arr)), xmem), offsetof(typeof(*(arr)), ymem), (ynum), (a), (b))

int thresholding(const void *arr, size_t arrsz, size_t arrisz,
    size_t yoff, size_t ynum, int8_t *_signals,
    size_t lag, double threshold, double influence);
#define THRESHOLDING(arr, arrsz, ymem, ynum, signals, lag, threshold, influence) \
    thresholding((arr), (arrsz), sizeof(*(arr)), \
    offsetof(typeof(*(arr)), ymem), (ynum), (signals), (lag), (threshold), (influence))

int pt_momentum(const void *arr, size_t arrsz, size_t arrisz,
    size_t xoff, size_t yoff, size_t ynum, int8_t *pt, double friction, double initial_min_momentum);
#define PT_MOMENTUM(arr, arrsz, xmem, ymem, ynum, pt, friction, imm) \
    pt_momentum((arr), (arrsz), sizeof(*(arr)), \
    offsetof(typeof(*(arr)), xmem), offsetof(typeof(*(arr)), ymem), (ynum), \
    (pt), (friction), (imm))

#endif /* SENSOREVAL_MATH_H */
