use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use clap::{Args, Parser, Subcommand, ValueEnum};

mod summation;
mod time;

use summation::InterpolateLookup;

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

#[derive(Debug, Clone, ValueEnum)]
enum AppAlgo {
    LeftRiemann,
    MidRiemann,
    RightRiemann,
    Trapezoidal,
    Simpsons,
}

#[derive(Debug, Subcommand)]
enum SimKind {
    /// runs a simulation from a given acceleration profile
    Csv(CsvSim),
}

#[derive(Debug, Args)]
struct CsvSim {
    /// loads acceleration data in a specific column from the csv file
    #[arg(long)]
    column: Option<String>,

    /// the csv file path to load
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = App::parse();

    match args.sim {
        SimKind::Csv(csv_args) => {
            let cb = csv_args.get_callable()?;
            let length = cb.len();

            if args.threads == 1 {
                run_sim(length, args.opts, cb);
            } else {
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
    fn get_path(&self) -> anyhow::Result<PathBuf> {
        if self.path.is_relative() {
            let cwd =
                std::env::current_dir().context("failed to retrieve current working directory")?;

            Ok(cwd.join(&self.path))
        } else {
            Ok(self.path.clone())
        }
    }

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

fn run_sim(length: usize, opts: SimOpts, accel_lookup: InterpolateLookup) {
    println!(
        "lenth: {length} step: {} iterations: {}",
        opts.step, opts.iterations
    );

    let sum_cb = match opts.algo {
        //_ => summation::left_riemann as fn(f64, f64, u32, &InterpolateLookup) -> f64,
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
        let mut pos_lookup = InterpolateLookup::from(Vec::with_capacity(length));
        vel_lookup.push(0.0);
        pos_lookup.push(0.0);

        let start = std::time::Instant::now();

        let vel_final = calc_range(length, opts.step, &accel_lookup, sum_cb, &mut vel_lookup);
        let pos_final = calc_range(length, opts.step, &vel_lookup, sum_cb, &mut pos_lookup);

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

fn calc_range(
    length: usize,
    step: u32,
    calling: &InterpolateLookup,
    sum_cb: fn(f64, f64, u32, &InterpolateLookup) -> f64,
    updating: &mut InterpolateLookup,
) -> f64 {
    let mut rolling = 0.0;

    for sec in 1..length {
        let result = sum_cb((sec - 1) as f64, sec as f64, step, calling);

        rolling += result;

        updating.push(rolling);
    }

    rolling
}

fn run_sim_rayon(length: usize, opts: SimOpts, accel_lookup: InterpolateLookup) {
    use rayon::prelude::*;

    println!(
        "lenth: {length} step: {} iterations: {}",
        opts.step, opts.iterations
    );

    let sum_cb = match opts.algo {
        //_ => summation::left_riemann as fn(f64, f64, u32, &InterpolateLookup) -> f64,
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

        // parallel
        let vel_diffs = (1..length)
            .into_par_iter()
            .map(|sec| sum_cb((sec - 1) as f64, sec as f64, opts.step, &accel_lookup))
            .collect::<Vec<f64>>();

        let mut vel_rolling = 0.0f64;

        for v in vel_diffs {
            vel_rolling += v;

            vel_lookup.push(vel_rolling);
        }

        // parallel
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
