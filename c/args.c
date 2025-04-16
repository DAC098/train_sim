#include <errno.h>
#include <getopt.h>
#include <limits.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "args.h"

void print_help() {
    printf(
"an application for running \"train\" simulations of a givne acceleration\n"
"profile that will calculate the final velocity and position of the train\n"
"\n"
"Usage: sim [OPTIONS] <PATH>\n"
"\n"
"Arguments:\n"
"  <PATH> the input file to load acceleration data from\n"
"\n"
"Options:\n"
"  -t, --threads <THREADS> specifies the number of threads to use for\n"
"                          simulation\n"
"  -a, --algo <ALGO>       specifies the summation algorithm to use for the\n"
"                          simulation [default: left-riemann]\n"
"  -i, --iterations <ITER> specifies the number times to run the program, for\n"
"                          benchmarking purposes\n"
"  -s, --step <STEP>       specifies the number of steps to take in bewteen each\n"
"                          summation calculation [default: 10]\n"
    );

}

void print_full_help() {
    printf(
"an application for running \"train\" simulations of a givne acceleration\n"
"profile that will calculate the final velocity and position of the train\n"
"\n"
"Usage: sim [OPTIONS] <PATH>\n"
"\n"
"Arguments:\n"
"  <PATH> the input file to load acceleration data from\n"
"\n"
"Options:\n"
"\n"
"  -t, --threads <THREADS>\n"
"        specifies the number of threads to use for simulation\n"
"\n"
"  -a, --algo <ALGO>\n"
"        specifies the summation algorithm to use for the simulation\n"
"        [default: left-riemann] [possible-values: left-riemann, mid-riemann,\n"
"        right-riemann, trapezoidal, simpsons]\n"
"\n"
"  -i, --iterations <ITER>\n"
"        specifies the number times to run the program, for benchmarking purposes\n"
"\n"
"  -s, --step <STEP>\n"
"        specifies the number of steps to take in bewteen each summation\n"
"        calculation [default: 10]\n"
    );
}

int32_t app_args_init(struct app_args* self, int argc, char** argv) {
    self->file_path = NULL;
    self->sim.threads = 1;
    self->sim.algo = 0;
    self->sim.step = 10;
    self->sim.iterations = 1;

    struct option long_options[] = {
        {"threads", required_argument, 0, 0 },
        {"step", required_argument, 0, 0 },
        {"iterations", required_argument, 0, 0 },
        {"algo", required_argument, 0, 0 },
        {"help", no_argument, 0, 0}
    };

    int32_t option_index = 0;

    while (1) {
        int32_t c = getopt_long(argc, argv, "t:s:i:a:h:", long_options, &option_index);

        if (c == -1) {
            return 0;
        }

        switch (c) {
        case 0:
            switch (option_index) {
            case 0:
                if (parse_threads_arg(optarg, &self->sim.threads) != 0) {
                    return 1;
                }
                break;
            case 1:
                if (parse_step_arg(optarg, &self->sim.step) != 0) {
                    return 1;
                }
                break;
            case 2:
                if (parse_iterations_arg(optarg, &self->sim.iterations) != 0) {
                    return 1;
                }
                break;
            case 3:
                if (parse_algo_arg(optarg, &self->sim.algo) != 0) {
                    return 1;
                }
                break;
            case 4:
                print_full_help();

                return 1;
            }
            break;
        case 't':
            if (parse_threads_arg(optarg, &self->sim.threads) != 0) {
                return 1;
            }
            break;
        case 's':
            if (parse_step_arg(optarg, &self->sim.step) != 0) {
                return 1;
            }
            break;
        case 'i':
            if (parse_iterations_arg(optarg, &self->sim.iterations) != 0) {
                return 1;
            }
            break;
        case 'a':
            if (parse_algo_arg(optarg, &self->sim.algo) != 0) {
                return 1;
            }
            break;
        case 'h':
            print_help();

            return 1;
        case '?':
            break;
        default:
            break;
        }
    }

    return 0;
}

int32_t parse_threads_arg(const char* arg, int32_t* threads) {
    long parsed = 0;

    if (parse_l(arg, &parsed) != 0) {
        fprintf(stderr, "invalid thread size provided\n");

        return 1;
    }

    if (parsed > INT32_MAX || parsed < 1) {
        fprintf(stderr, "invalid thread size provided\n");

        return 1;
    }

    *threads = parsed;

    return 0;
}

int32_t parse_algo_arg(const char* arg, int32_t* algo) {
    if (strncmp(arg, "left-riemann", 13) == 0) {
        *algo = LEFT_RIEMANN;
    } else if (strncmp(arg, "mid-riemann", 12) == 0) {
        *algo = MID_RIEMANN;
    } else if (strncmp(arg, "right-riemann", 14) == 0) {
        *algo = RIGHT_RIEMANN;
    } else if (strncmp(arg, "trapezoidal", 12) == 0) {
        *algo = TRAPEZOIDAL;
    } else if (strncmp(arg, "simpsons", 9) == 0) {
        *algo = SIMPSONS;
    } else {
        fprintf(stderr, "invalid algo provided\n");

        return 1;
    }

    return 0;
}

int32_t parse_step_arg(const char* arg, int32_t* step) {
    long parsed = 0;

    if (parse_l(arg, &parsed) != 0) {
        fprintf(stderr, "invalid step provided\n");

        return 1;
    }

    if (parsed > INT32_MAX || parsed < 1) {
        fprintf(stderr, "invalid step provided\n");

        return 1;
    }

    *step = parsed;

    return 0;
}

int32_t parse_iterations_arg(const char* arg, int32_t* iterations) {
    long parsed = 0;

    if (parse_l(arg, &parsed) != 0) {
        fprintf(stderr, "invalid iterations provided\n");

        return 1;
    }

    if (parsed > INT32_MAX || parsed < 1) {
        fprintf(stderr, "invalid iterations provided\n");

        return 1;
    }

    *iterations = parsed;

    return 0;

}

int parse_l(const char* str, long* value) {
    char* endptr;
    *value = strtol(str, &endptr, 10);

    if (*endptr != '\0') {
        return -1;
    }

    if ((*value == LONG_MAX && errno == ERANGE) || errno == EINVAL) {
        return -1;
    }

    return 0;
}
