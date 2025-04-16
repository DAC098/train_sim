#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include "summation.h"

double get_lut_index(struct lut_info* lut, int32_t index) {
    // if the given lut does not cover the range because it is only a small
    // subset of the total then we can use the offset to get the appropriate
    // index with the specified offset
    if (index < 0 || index >= lut->len) {
        fprintf(
            stderr,
            "attempted to access lut index that is out of bounds. index: %d len: %d\n",
            index,
            lut->len
        );

        exit(EXIT_FAILURE);
    }

    return lut->lut[index];
}

// linear interpolation using this equation:
// y = y0 + ((x - x0) * ((y1 - y0) / (x1 - x0)))
double calc_linear_interpolation(void* context, double x) {
    struct lut_info* lut = (struct lut_info*)context;

    if (floor(x) == x) {
        // the x value we are given will just be an index which we know so just
        // go get it
        return get_lut_index(lut, (int32_t)x);
    }

    // cast the given doule to an int which will truncate all decimals and give
    // us the lower x value (x0).
    int32_t x0_index = (int32_t)x;
    // increment from x0_index to get x1;
    int32_t x1_index = x0_index + 1;

    double x0 = (double)x0_index;
    // since x1 will always be 1 greater than x0
    //double x1 = (double)x1_index;
    double y0 = get_lut_index(lut, x0_index);
    double y1 = get_lut_index(lut, x1_index);

    // if x1 is always 1 greather than x0 then it can be removed and just 1;
    //return y0 + (x - x0) * ((y1 - y0) / (x1 - x0));
    return y0 + (x - x0) * (y1 - y0);
}

// summation functions

double left_riemann(
    double lower,
    double upper,
    int32_t iterations,
    void* context,
    summation_cb cb
) {
    double step = (upper - lower) / (double)iterations;
    double sum = 0.0;

    for (int32_t iter = 0; iter < iterations; iter += 1) {
        double x = lower + (double)iter * step;

        // Add the value of the function at the left endpoint of each subinterval.
        sum += cb(context, x);
    }

    return step * sum;
}

double mid_riemann(
    double lower,
    double upper,
    int32_t iterations,
    void* context,
    summation_cb cb
) {
    double step = (upper - lower) / (double)iterations;
    double half_step = step / 2.0;
    double sum = 0.0;

    for (int32_t iter = 0; iter < iterations; iter += 1) {
        double x = (lower + (double)iter * step) + half_step;

        // Add the value of the function at the left endpoint of each subinterval.
        sum += cb(context, x);
    }

    return step * sum;
}

double right_riemann(
    double lower,
    double upper,
    int32_t iterations,
    void* context,
    summation_cb cb
) {
    double step = (upper - lower) / (double)iterations;
    double sum = 0.0;

    for (int32_t iter = 0; iter < iterations; iter += 1) {
        double x = lower + (double)(iter + 1) * step;

        sum += cb(context, x);
    }

    return step * sum;
}

double trapezoidal(
    double lower,
    double upper,
    int32_t iterations,
    void* context,
    summation_cb cb
) {
    double step = (upper - lower) / (double)iterations;
    double sum = (cb(context, lower) + cb(context, upper)) / 2.0;

    for (int32_t iter = 1; iter < iterations; iter += 1) {
        double x = lower + (double)iter * step;

        sum += cb(context, x);
    }

    return step * sum;
}

double simpsons(
    double lower,
    double upper,
    int32_t iterations,
    void* context,
    summation_cb cb
) {
    double step = (upper - lower) / (double)iterations;
    double sum = 0.0;

    for (int32_t iter = 0; iter <= iterations; iter += 1) {
        double x = lower + (double)iter * step;
        double res = cb(context, x);

        if (iter == 0 || iter == iterations) {
            sum += res;
        } else if (iter % 2 == 1) {
            sum += 4.0 * res;
        } else {
            sum += 2.0 * res;
        }
    }

    return step * sum / 3.0;
}
