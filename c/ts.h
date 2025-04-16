#ifndef TS_H
#define TS_H

#include <stdint.h>
#include <time.h>

struct timing {
    struct timespec min;
    struct timespec max;
    struct timespec total;
    uint32_t count;
};

struct log_timer {
    struct timespec prev;
    struct timespec dur;
};

void timing_init(struct timing* init);
void timing_update(struct timing* self, struct timespec* given);
void timing_print(struct timing* self);

int32_t log_timer_init(struct log_timer* self);
int32_t log_timer_update(struct log_timer* self);

void time_diff(struct timespec* start, struct timespec* end, struct timespec* diff);
void time_min(struct timespec* l, struct timespec* r, struct timespec* min);
void time_max(struct timespec* l, struct timespec* r, struct timespec* max);
void time_add(struct timespec* l, struct timespec* r, struct timespec* add);
int32_t time_div(struct timespec* l, uint32_t count, struct timespec* div);
int32_t time_eq(struct timespec* l, struct timespec* r);
int32_t time_ge(struct timespec* l, struct timespec* r);

#endif
