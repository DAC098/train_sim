#ifndef SUMMATION_H
#define SUMMATION_H

#include <stdint.h>

struct lut_info {
    int32_t len;
    double* lut;
};

typedef double (*summation_cb)(void*, double);
typedef double (*summation)(double, double, int32_t, void*, summation_cb);

double get_lut_index(struct lut_info* lut, int32_t index);
double calc_linear_interpolation(void* context, double x);

// summation functions

double left_riemann(
    double lower,
    double upper,
    int32_t iterations,
    void* context,
    summation_cb cb
);

double mid_riemann(
    double lower,
    double upper,
    int32_t iterations,
    void* context,
    summation_cb cb
);

double right_riemann(
    double lower,
    double upper,
    int32_t iterations,
    void* context,
    summation_cb cb
);

double trapezoidal(
    double lower,
    double upper,
    int32_t iterations,
    void* context,
    summation_cb cb
);

double simpsons(
    double lower,
    double upper,
    int32_t iterations,
    void* context,
    summation_cb cb
);

#endif
