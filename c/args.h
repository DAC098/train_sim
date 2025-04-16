#ifndef ARGS_H
#define ARGS_H

#include <stdint.h>

enum ALGO {
    LEFT_RIEMANN = 0,
    MID_RIEMANN = 1,
    RIGHT_RIEMANN = 2,
    TRAPEZOIDAL = 3,
    SIMPSONS = 4
};

struct sim_args {
    int32_t threads;
    int32_t algo;
    int32_t step;
    int32_t iterations;
};

struct app_args {
    char* file_path;
    struct sim_args sim;
};

void print_help();
void print_full_help();

int32_t parse_threads_arg(const char* arg, int32_t* threads);
int32_t parse_algo_arg(const char* arg, int32_t* algo);
int32_t parse_step_arg(const char* arg, int32_t* step);
int32_t parse_iterations_arg(const char* arg, int32_t* iterations);

int32_t parse_l(const char* str, int64_t* value);

int32_t app_args_init(struct app_args* self, int argc, char** argv);

#endif
