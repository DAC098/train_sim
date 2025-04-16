#ifndef SIM_H
#define SIM_H

#include <stdint.h>
#include <time.h>

#include "summation.h"
#include "args.h"

void run_sim(struct sim_args* args, struct lut_info* lut);
void run_sim_openmp(struct sim_args* args, struct lut_info* lut);

int32_t load_data(const char* file_path);

#endif
