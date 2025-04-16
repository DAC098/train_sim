#include <stdio.h>

#include "ts.h"

void timing_init(struct timing* init) {
    init->min.tv_sec = 1000000000;
    init->min.tv_nsec = 0;
    init->max.tv_sec = 0;
    init->max.tv_nsec = 0;
    init->total.tv_sec = 0;
    init->total.tv_nsec = 0;
    init->count = 0;
}

void timing_update(struct timing* self, struct timespec* given) {
    time_min(&self->min, given, &self->min);
    time_max(&self->max, given, &self->max);

    time_add(&self->total, given, &self->total);

    self->count += 1;
}

void timing_print(struct timing* self) {
    if (self->count > 1) {
        struct timespec avg;

        if (time_div(&self->total, self->count, &avg) != 0) {
            printf("bad nanos calculated\n");

            return;
        }

        printf(
            "min: %ld.%.9ld\nmax: %ld.%.9ld\navg: %ld.%.9ld\ntot: %ld.%.9ld\n",
            self->min.tv_sec,
            self->min.tv_nsec,
            self->max.tv_sec,
            self->max.tv_nsec,
            avg.tv_sec,
            avg.tv_nsec,
            self->total.tv_sec,
            self->total.tv_nsec
        );
    } else {
        printf("total: %ld.%.9ld\n", self->total.tv_sec, self->total.tv_nsec);
    }
}

int32_t log_timer_init(struct log_timer* self) {
    if (clock_gettime(CLOCK_MONOTONIC, &self->prev) != 0) {
        return 1;
    }

    self->dur.tv_sec = 10;
    self->dur.tv_nsec = 0;

    return 0;
}

int32_t log_timer_update(struct log_timer* self) {
    struct timespec now;
    struct timespec diff;

    if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) {
        return -1;
    }

    time_diff(&self->prev, &now, &diff);

    if (time_ge(&diff, &self->dur)) {
        self->prev.tv_sec = now.tv_sec;
        self->prev.tv_nsec = now.tv_nsec;

        return 1;
    } else {
        return 0;
    }
}

void time_diff(struct timespec* start, struct timespec* end, struct timespec* diff) {
    time_t tv_sec = end->tv_sec - start->tv_sec;
    long tv_nsec = end->tv_nsec - start->tv_nsec;

    if (tv_nsec < 0) {
        tv_sec -= 1;
        tv_nsec += 1000000000;
    }

    diff->tv_sec = tv_sec;
    diff->tv_nsec = tv_nsec;
}

void time_min(struct timespec* l, struct timespec* r, struct timespec* min) {
    if (l->tv_sec > r->tv_sec) {
        min->tv_sec = r->tv_sec;
        min->tv_nsec = r->tv_nsec;
    } else if (l->tv_sec < r->tv_sec) {
        min->tv_sec = l->tv_sec;
        min->tv_nsec = l->tv_nsec;
    } else {
        if (l->tv_nsec > r->tv_nsec) {
            min->tv_sec = r->tv_sec;
            min->tv_nsec = r->tv_nsec;
        } else if (l->tv_nsec < r->tv_nsec) {
            min->tv_sec = l->tv_sec;
            min->tv_nsec = l->tv_nsec;
        }
    }
}

void time_max(struct timespec* l, struct timespec* r, struct timespec* max) {
    if (l->tv_sec < r->tv_sec) {
        max->tv_sec = r->tv_sec;
        max->tv_nsec = r->tv_nsec;
    } else if (l->tv_sec > r->tv_sec) {
        max->tv_sec = l->tv_sec;
        max->tv_nsec = l->tv_nsec;
    } else {
        if (l->tv_nsec < r->tv_nsec) {
            max->tv_sec = r->tv_sec;
            max->tv_nsec = r->tv_nsec;
        } else if (l->tv_nsec > r->tv_nsec) {
            max->tv_sec = l->tv_sec;
            max->tv_nsec = l->tv_nsec;
        }
    }
}

void time_add(struct timespec* l, struct timespec* r, struct timespec* add) {
    time_t tv_sec = l->tv_sec + r->tv_sec;
    long tv_nsec = l->tv_nsec + r->tv_nsec;

    if (tv_nsec >= 1000000000) {
        tv_sec += tv_nsec / 1000000000;
        tv_nsec %= 1000000000;
    }

    add->tv_sec = tv_sec;
    add->tv_nsec = tv_nsec;
}

int32_t time_div(struct timespec* l, uint32_t count, struct timespec* div) {
    if (count != 0) {
        time_t secs = l->tv_sec / count;
        time_t secs_extra = l->tv_sec % count;
        int32_t nanos = l->tv_nsec / count;
        int32_t nanos_extra = l->tv_nsec % count;

        nanos += (secs_extra * 1000000000 + nanos_extra) / count;

        if (nanos >= 1000000000) {
            return 2;
        }

        div->tv_sec = secs;
        div->tv_nsec = nanos;

        return 0;
    } else {
        return 1;
    }
}

int32_t time_eq(struct timespec* l, struct timespec* r) {
    if (l->tv_sec == r->tv_sec) {
        return l->tv_nsec == r->tv_nsec;
    } else {
        return 0;
    }
}

int32_t time_ge(struct timespec* l, struct timespec* r) {
    if (l->tv_sec > r->tv_sec) {
        return 1;
    } else if (l->tv_sec == r->tv_sec) {
        return l->tv_nsec >= r->tv_nsec;
    } else {
        return 0;
    }
}
