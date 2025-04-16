// this code was pulled from CSCI-440 for the final project of testing
// different languages. it will be modified to adhere to the requuirements of
// assignment for this class

#include <errno.h>
#include <limits.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/sysinfo.h>
#include <time.h>
#include <omp.h>

#include "sim.h"
#include "ts.h"
#include "args.h"
#include "summation.h"
#include "table_lookup.h"

int main(int argc, char** argv) {
    struct app_args args;

    if (app_args_init(&args, argc, argv) != 0) {
        return 1;
    }

    /*
    struct lut_info accel_lut;
    accel_lut.len = 0;
    accel_lut.lut = (double*)malloc(accel_lut.len * sizeof(double));
    */
    struct lut_info accel_lut;
    accel_lut.len = TABLE_SIZE;
    accel_lut.lut = ACCELERATION_DATA;

    if (args.sim.threads == 1) {
        run_sim(&args.sim, &accel_lut);
    } else {
        run_sim_openmp(&args.sim, &accel_lut);
    }

    return 0;
}

void run_sim(struct sim_args* args, struct lut_info* lut) {
    struct timing time_data;
    struct log_timer log_time;

    timing_init(&time_data);
    log_timer_init(&log_time);

    struct lut_info vel_lut;
    vel_lut.len = lut->len;
    vel_lut.lut = (double*)malloc(vel_lut.len * sizeof(double));

    if (vel_lut.lut == NULL) {
        fprintf(stderr, "failed allocating vel lookup table. %s\n", strerror(errno));

        return;
    }

    vel_lut.lut[0] = 0.0;

    summation sum_cb = left_riemann;

    switch (args->algo) {
    case LEFT_RIEMANN:
        break;
    case MID_RIEMANN:
        sum_cb = mid_riemann;
        break;
    case RIGHT_RIEMANN:
        sum_cb = right_riemann;
        break;
    case TRAPEZOIDAL:
        sum_cb = trapezoidal;
        break;
    case SIMPSONS:
        sum_cb = simpsons;
        break;
    }

    for (int c = 0; c < args->iterations; c += 1) {
        struct timespec start;
        struct timespec end;
        start.tv_sec = 0;
        start.tv_nsec = 0;
        end.tv_sec = 0;
        end.tv_nsec = 0;

        if (clock_gettime(CLOCK_MONOTONIC, &start) != 0) {
            fprintf(stderr, "failed to retrieve start time\n");

            continue;
        }

        double vel_final = 0.0;

        for (int32_t sec = 1; sec < lut->len; sec += 1) {
            vel_final += sum_cb(
                (double)(sec - 1),
                (double)sec,
                args->step,
                (void*)lut,
                calc_linear_interpolation
            );

            vel_lut.lut[sec] = vel_final;
        }

        double pos_final = 0.0;

        for (int32_t sec = 1; sec < vel_lut.len; sec += 1) {
            pos_final += sum_cb(
                (double)(sec - 1),
                (double)sec,
                args->step,
                (void*)&vel_lut,
                calc_linear_interpolation
            );
        }

        if (clock_gettime(CLOCK_MONOTONIC, &end) == 0) {
            struct timespec diff;

            time_diff(&start, &end, &diff);

            timing_update(&time_data, &diff);

            switch (log_timer_update(&log_time)) {
            case 1:
                printf("iteration: %d\n", c);

                timing_print(&time_data);
                break;
            case 0:
                // all good
                break;
            case -1:
                fprintf(stderr, "error when updating log_timer\n");
                break;
            }
        } else {
            fprintf(stderr, "failed failed to retrieve end time. %s\n", strerror(errno));

            break;
        }

        if (c == args->iterations - 1) {
            printf("velocity: %.15lf\n", vel_final);
            printf("position: %.15lf\n", pos_final);
        }
    }

    timing_print(&time_data);

    free(vel_lut.lut);
}

void run_sim_openmp(struct sim_args* args, struct lut_info* lut) {
    struct timing time_data;
    struct log_timer log_time;

    timing_init(&time_data);
    log_timer_init(&log_time);

    struct lut_info vel_lut;
    vel_lut.len = lut->len;
    vel_lut.lut = (double*)malloc(vel_lut.len * sizeof(double));

    if (vel_lut.lut == NULL) {
        fprintf(stderr, "failed allocating vel lookup table. %s\n", strerror(errno));

        return;
    }

    vel_lut.lut[0] = 0.0;

    summation sum_cb = left_riemann;

    switch (args->algo) {
    case LEFT_RIEMANN:
        break;
    case MID_RIEMANN:
        sum_cb = mid_riemann;
        break;
    case RIGHT_RIEMANN:
        sum_cb = right_riemann;
        break;
    case TRAPEZOIDAL:
        sum_cb = trapezoidal;
        break;
    case SIMPSONS:
        sum_cb = simpsons;
        break;
    }

    for (int c = 0; c < args->iterations; c += 1) {
        struct timespec start;
        struct timespec end;
        start.tv_sec = 0;
        start.tv_nsec = 0;
        end.tv_sec = 0;
        end.tv_nsec = 0;

        if (clock_gettime(CLOCK_MONOTONIC, &start) != 0) {
            fprintf(stderr, "failed to retrieve start time\n");

            continue;
        }

#pragma omp parallel for num_threads(args->threads)
        for (int32_t sec = 1; sec < lut->len; sec += 1) {
            vel_lut.lut[sec] = sum_cb(
                (double)(sec - 1),
                (double)sec,
                args->step,
                (void*)lut,
                calc_linear_interpolation
            );
        }

        double vel_final = 0.0;

        for (int32_t index = 0; index < vel_lut.len; index += 1) {
            vel_final += vel_lut.lut[index];

            vel_lut.lut[index] = vel_final;
        }

        double pos_final = 0.0;

#pragma omp parallel for num_threads(args->threads) reduction(+:pos_final)
        for (int32_t sec = 1; sec < vel_lut.len; sec += 1) {
            pos_final += sum_cb(
                (double)(sec - 1),
                (double)sec,
                args->step,
                (void*)&vel_lut,
                calc_linear_interpolation
            );
        }

        if (clock_gettime(CLOCK_MONOTONIC, &end) == 0) {
            struct timespec diff;

            time_diff(&start, &end, &diff);

            timing_update(&time_data, &diff);

            switch (log_timer_update(&log_time)) {
            case 1:
                printf("iteration: %d\n", c);

                timing_print(&time_data);
                break;
            case 0:
                // all good
                break;
            case -1:
                fprintf(stderr, "error when updating log_timer\n");
                break;
            }
        } else {
            fprintf(stderr, "failed failed to retrieve end time. %s\n",strerror(errno));

            break;
        }

        if (c == args->iterations - 1) {
            printf("velocity: %.15lf\n", vel_final);
            printf("position: %.15lf\n", pos_final);
        }
    }

    timing_print(&time_data);

    free(vel_lut.lut);
}
