# Acceleration, Velocity, Position Profiler

Author: David Cathers, CSU Chico

This is an application for profiling simple velocity and position from a given
acceleration profile. It is based around the idea of Assignment 3 / 4 when
modeling a "train" on a track of railroad. The repository contains 2 different
implementations of the application with one being written in Rust while the
other is written in C which is the base implementation.

The Rust version uses the Rayon library to assist with parallelizing the
algorithm by using a thread pool of a given size. The C version uses OpenMP
compiler directives to achive the same goal. The Rust version also includes
additional libraries to assist with execution of the program and loading data
but not for running the algorithm. Both implementations provide a non-parallel
implementation to use as a base line when comparing speed up of the application.

The C version is currently limited in scope as the acceleration data is being
loaded from an static table that is hard coded to the executable. Further work
could be done to include a file loading mechanism to be more on par with the
Rust version.

Both applications include additional code for timing and benchmarking purposes
to assist with collection information for this report. All timing is around the
critical portions of the application which is only the implementation of the
algorithm. When the algorithm is run over multiple iterations the programs will
aggregate the times together to display the minimum, maximum, average, and total
times.

## Algorithm Implementation

The base algorithm for both implementations follows this process.
1. Load all acceleration data into memory
2. Calculate velocity data from acceleration
  1. The non-parallel version calculates the velocity data in one loop since we
     will not have to worry about sharing the rolling velocity between multiple
     threads
  2. The parallel version calculates the velocity differences in one iteration
     and then a second iteration is used to calculate the rolling velocity in
     order to have a valid velocity table.
3. Calculate position data from the velocity
  1. The non-parallel version calculates the position in a similar fashion as
     the velocity but only keeps the final position.
  2. The parallel version calculates the position using reduction between all
     the threads since we only care about the final position.

Calculations for each step of the data use summations and you can choose between
Riemann sums (left, mid, right), Trapezoidal rule, or Simpsons rule. The
implementations do not use any parallelization for additional speed up but
further investigation can be done to see if it would benefit with out contesting
the current threading model. Interpolated lookup tables are used with the
summations in order to calculate sub steps between data points.

## Executing

### Rust

The latest Rust compiler will be able to build and run executable. Testing was
done on a linux system and should be possible to run on other operating systems
but has not been tested. Cargo will handle all the dependencies so no further
work will be necessary. The following command will run the Rust version with a
single thread, a step of 100 (1/100 th of a second), a single iteration, use
the Left Riemann summation, and load the acceleration data from the "accel.csv":

```
cargo run -- -t 1 -s 100 -i 1 -a left-riemann csv ./accel.csv
```

This command will run the executable with compiler optimizations enabled and the
same arguments:

```
cargo run --release -- -t 1 -s 100 -i 1 -a left-riemann csv ./accel.csv
```

### C

Testing was done using GCC that has OpenMP and the Math library. The executable
is built for a Linux operating system as it uses specific libraries from the
operating system. There are no additional dependencies outside of the OS as most
of the code was custom made. The following commands will build and make the c
version (you must be in the `c` directory as build will be relative to that
directory):

```
make debug
```

This will create a *release* version of the executable:

```
make release
```

Once built the following command can be run to perform a similar execution as
the Rust version with the staticly defined acceleration data:

```
./build/debug/sim -t 1 -s 100 -i 1 -a left-riemann
```

You can change the `debug` to `release` if you want to run the *release* version
of the program.

## Timing Executables

The following results are run from a dedicated server that the school provides
that is running a native Linux distribution with a 16 core CPU. All tests are
executed with 1000 iterations and time values are in nanoseconds.

### Left Riemann - 1/100th Second

#### Rust

|threads|minimum|maximum|average|total|
|-------|-------|-------|-------|-----|
|1|0.003213292|0.035432495|0.007241681|7.241681209|
|2|0.001639495|0.066749518|0.008686383|8.686383849|
|4|0.000833231|0.058341602|0.005805893|5.805893254|
|8|0.000471954|0.039031958|0.003836672|3.836672916|
|12|0.000378842|0.033974216|0.003009247|3.009247389|

#### C

|threads|minimum|maximum|average|total|
|-------|-------|-------|-------|-----|
|1|0.002276755|0.055704111|0.005973172|5.973172412|
|2|0.001165612|0.066871359|0.006497647|6.497647946|
|4|0.000585604|0.075883762|0.009214488|9.214488758|
|8|0.000297638|0.078769623|0.017638049|17.638049707|
|12|0.000202818|0.083641884|0.030317523|30.317523140|

### Left Riemann - 1/1000th Second

#### Rust

|threads|minimum|maximum|average|total|
|-------|-------|-------|-------|-----|
|1|0.031327088|0.149941795|0.050102533|50.102533333|
|2|0.016127321|0.159415059|0.060624244|60.624244736|
|4|0.009475283|0.112430939|0.041852318|41.852318584|
|8|0.005029760|0.098162725|0.027322942|27.322942312|
|12|0.003834157|0.102013888|0.022175896|22.175896394|

#### C

|threads|minimum|maximum|average|total|
|-------|-------|-------|-------|-----|
|1|0.022480036|0.142639903|0.050446065|50.446065557|
|2|0.011582219|0.122996655|0.048830783|48.830783223|
|4|0.005809150|0.099402851|0.041619933|41.619933592|
|8|0.002914228|0.093985994|0.039317892|39.317892056|
|12|0.010956094|0.095700695|0.042838571|42.838571250|

