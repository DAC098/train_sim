// NOTE: comments are the usual double slash but for documentation it will be a
// triple slash. some of the tools that rust provides use these doc blocks to
// generate documents that can be accessed outside of the code.

use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use clap::{Args, Parser, Subcommand, ValueEnum};

// indicates that there are nested modules that can contain code in a different
// namespace
mod summation;
mod time;

// once the mod is known we can access it similar to imported modules or the
// std namespace
use summation::InterpolateLookup;

/// an application for running "train" simulations of a given acceleration
/// profile that will calculate the final velocity and position of the train
#[derive(Debug, Parser)]
struct App {
    /// specifies the number of threads to use for calculations
    #[arg(short, long, default_value("1"))]
    threads: usize,

    #[command(flatten)]
    opts: SimOpts,

    #[command(subcommand)]
    sim: SimKind,
}

/// common options between simulations
#[derive(Debug, Clone, Args)]
struct SimOpts {
    /// specifies the summation algorithm to use for the simulation
    #[arg(short, long, default_value("left-riemann"))]
    algo: AppAlgo,

    /// determines the number of times to run the program, for benchmarking
    /// purposes
    #[arg(short, long, default_value("100"))]
    iterations: u32,

    /// specifies the amount of steps to take in between each summation
    /// calculation
    #[arg(short, long, default_value("100"))]
    step: u32,
}

/// the available summation algorithms that the simulation is capable of
/// running
#[derive(Debug, Clone, ValueEnum)]
enum AppAlgo {
    LeftRiemann,
    MidRiemann,
    RightRiemann,
    Trapezoidal,
    Simpsons,
}

/// the different kins of simulations available for the program to run
///
/// currently the only supported kind is loading data from a csv file
#[derive(Debug, Subcommand)]
enum SimKind {
    /// runs a simulation from a given acceleration profile
    Csv(CsvSim),
}

/// options for running a simulation from a specified csv file
#[derive(Debug, Args)]
struct CsvSim {
    /// loads acceleration data in a specific column from the csv file
    #[arg(long)]
    column: Option<String>,

    /// the csv file path to load
    path: PathBuf,
}

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

impl CsvSim {
    /// retrieves the path of the specified csv file
    ///
    /// if the given path is relative then it will be resolved using the
    /// current working directory
    fn get_path(&self) -> anyhow::Result<PathBuf> {
        if self.path.is_relative() {
            let cwd =
                std::env::current_dir().context("failed to retrieve current working directory")?;

            Ok(cwd.join(&self.path))
        } else {
            Ok(self.path.clone())
        }
    }

    /// builds the [`csv::Reader`] from the provided csv path
    fn get_csv_reader(&self) -> anyhow::Result<csv::Reader<std::fs::File>> {
        let path = self.get_path()?;

        let mut builder = csv::ReaderBuilder::new();

        if self.column.is_some() {
            builder.has_headers(true);
        } else {
            builder.has_headers(false);
        }

        builder.from_path(&path).context("failed to load csv file")
    }

    /// parses the given csv file into a lookup table that supports
    /// interpolation
    fn get_callable(self) -> anyhow::Result<summation::InterpolateLookup> {
        let mut rtn = Vec::new();
        let mut reader = self.get_csv_reader()?;

        let data_index = if let Some(column) = self.column {
            let mut maybe_index: Option<usize> = None;
            let headers = reader.headers().context("failed to retrieve csv headers")?;

            for (index, header) in headers.iter().enumerate() {
                if header == column {
                    maybe_index = Some(index);

                    break;
                }
            }

            maybe_index.context("failed to find the desired csv column")?
        } else {
            0
        };

        let records = reader.records();

        for (index, try_record) in records.enumerate() {
            let record = try_record
                .with_context(|| format!("failed to retrieve csv entry. {}", index + 1))?;

            let value = record
                .get(data_index)
                .with_context(|| format!("failed to retrieve csv entry column. {}", index + 1))?;

            rtn.push(f64::from_str(value).with_context(|| {
                format!("failed to convert csv entry into float. {}", index + 1)
            })?);
        }

        Ok(InterpolateLookup::from(rtn))
    }
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

    println!("time: {timer}");
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

    println!("time: {timer}");
}
