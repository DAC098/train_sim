// NOTE: comments are the usual double slash but for documentation it will be a
// triple slash. some of the tools that rust provides use these doc blocks to
// generate documents that can be accessed outside of the code.

use anyhow::Context;
use clap::Parser;

// indicates that there are nested modules that can contain code in a different
// namespace
mod args;
mod summation;
mod time;

use args::{App, AppAlgo, SimKind, SimOpts};

// once the mod is known we can access it similar to imported modules or the
// std namespace
use summation::InterpolateLookup;

fn main() -> anyhow::Result<()> {
    // pull in the command line arguments provided at runtime and parse into
    // the App struct
    let args = App::parse();

    match args.sim {
        SimKind::Csv(csv_args) => {
            let cb = csv_args.get_callable()?;
            let length = cb.len();

            if args.threads == 1 {
                run_sim(length, args.opts, cb);
            } else {
                // construct the rayon thread pool with the specified number of
                // threads and make it globaly available
                rayon::ThreadPoolBuilder::new()
                    .num_threads(args.threads)
                    .build_global()
                    .context("failed to create global thread pool")?;

                run_sim_rayon(length, args.opts, cb);
            }
        }
    };

    Ok(())
}

/// runs the non multi-threaded train sim with the provided lookup table
fn run_sim(length: usize, opts: SimOpts, accel_lookup: InterpolateLookup) {
    println!(
        "lenth: {length} step: {} iterations: {}",
        opts.step, opts.iterations
    );

    // let the type system decide what this is supposed to be as I was having
    // trouble with getting it to behave
    let sum_cb = match opts.algo {
        AppAlgo::LeftRiemann => summation::left_riemann,
        AppAlgo::MidRiemann => summation::mid_riemann,
        AppAlgo::RightRiemann => summation::right_riemann,
        AppAlgo::Trapezoidal => summation::trapezoidal,
        AppAlgo::Simpsons => summation::simpsons,
    };

    let mut log_timer = time::LogTimer::default();
    let mut timer = time::Timing::default();

    for iter in 0..(opts.iterations) {
        // pre-allocate the lookup table before starting the timer
        let mut vel_lookup = InterpolateLookup::from(Vec::with_capacity(length));
        vel_lookup.push(0.0);

        let start = std::time::Instant::now();

        let mut vel_final = 0.0f64;

        for sec in 1..length {
            let result = sum_cb((sec - 1) as f64, sec as f64, opts.step, &accel_lookup);

            vel_final += result;

            vel_lookup.push(vel_final);
        }

        let pos_final = (1..length)
            .map(|sec| sum_cb((sec - 1) as f64, sec as f64, opts.step, &vel_lookup))
            .sum::<f64>();

        timer.update(start.elapsed());

        if log_timer.update() {
            println!("iteration: {iter} {timer}");
        }

        if iter == opts.iterations - 1 {
            println!("final velocity: {vel_final:+}");
            println!("final position: {pos_final:+}");
        }
    }

    println!("{timer}");
}

/// runs the multi-threaded train sim with the provided lookup table
fn run_sim_rayon(length: usize, opts: SimOpts, accel_lookup: InterpolateLookup) {
    // since this is the only spot that will use the rayon module we can just
    // import it here.
    use rayon::prelude::*;

    println!(
        "lenth: {length} step: {} iterations: {}",
        opts.step, opts.iterations
    );

    let sum_cb = match opts.algo {
        AppAlgo::LeftRiemann => summation::left_riemann,
        AppAlgo::MidRiemann => summation::mid_riemann,
        AppAlgo::RightRiemann => summation::right_riemann,
        AppAlgo::Trapezoidal => summation::trapezoidal,
        AppAlgo::Simpsons => summation::simpsons,
    };

    let mut log_timer = time::LogTimer::default();
    let mut timer = time::Timing::default();

    for iter in 0..(opts.iterations) {
        let mut vel_lookup = InterpolateLookup::from(Vec::with_capacity(length));
        vel_lookup.push(0.0);

        let start = std::time::Instant::now();

        // we are going to calculate all of the differences between the
        // acceleration values and then sum them together after they have been
        // calculated. once everything has been calculated we will collected
        // them into a vec of f64's and the ordering will be preserved.
        let vel_diffs = (1..length)
            .into_par_iter()
            .map(|sec| sum_cb((sec - 1) as f64, sec as f64, opts.step, &accel_lookup))
            .collect::<Vec<f64>>();

        let mut vel_rolling = 0.0f64;

        for v in vel_diffs {
            vel_rolling += v;

            vel_lookup.push(vel_rolling);
        }

        let pos_final = (1..length)
            .into_par_iter()
            .map(|sec| sum_cb((sec - 1) as f64, sec as f64, opts.step, &vel_lookup))
            .sum::<f64>();

        timer.update(start.elapsed());

        if log_timer.update() {
            println!("iteration: {iter} {timer}");
        }

        if iter == opts.iterations - 1 {
            println!("final velocity: {vel_rolling:+}");
            println!("final position: {pos_final:+}");
        }
    }

    println!("{timer}");
}
