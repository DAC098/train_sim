.PHONY: clean
.DEFAULT_GOAL: debug

CCFLAGS = -Wall -Wextra -fopenmp
objects = sim.o args.o ts.o summation.o
build_dir = build/

.all: debug release

debug: build_dir = build/debug/
debug: CCFLAGS += -ggdb
debug: init sim

release: build_dir = build/release/
release: CCFLAGS += -O3
release: init sim

init:
	mkdir -p $(build_dir)

sim: $(objects)
	gcc $(CCFLAGS) -o $(addprefix $(build_dir), $@) $(addprefix $(build_dir), $(objects)) -lm

sim.o: sim.c sim.h ts.o summation.o table_lookup.h
	gcc $(CCFLAGS) -o $(addprefix $(build_dir), $@) -c sim.c

ts.o: ts.c ts.h
	gcc $(CCFLAGS) -o $(addprefix $(build_dir), $@) -c ts.c

summation.o: summation.c summation.h
	gcc $(CCFLAGS) -o $(addprefix $(build_dir), $@) -c summation.c

args.o: args.c args.h
	gcc $(CCFLAGS) -o $(addprefix $(build_dir), $@) -c args.c

clean:
	rm -Rd build
