use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use clap::{Parser, ValueEnum, Subcommand, Args};

mod time;
mod summation;

use summation::InterpolateLookup;

#[derive(Debug, Parser)]
struct App {
    #[command(flatten)]
    opts: SimOpts,

    #[command(subcommand)]
    sim: SimKind,
}

#[derive(Debug, Clone, Args)]
struct SimOpts {
    #[arg(short, long, default_value("left-riemann"))]
    algo: AppAlgo,

    #[arg(short, long, default_value("100"))]
    iterations: u32,

    #[arg(short, long, default_value("100"))]
    step: u32,
}

#[derive(Debug, Clone, ValueEnum)]
enum AppAlgo {
    LeftRiemann,
    MidRiemann,
    RightRiemann,
    Trapezoidal,
}

#[derive(Debug, Subcommand)]
enum SimKind {
    Csv(CsvSim),
}

#[derive(Debug, Args)]
struct CsvSim {
    #[arg(long)]
    column: Option<String>,

    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = App::parse();

    match args.sim {
        SimKind::Csv(csv_args) => {
            let cb = csv_args.get_callable()?;
            let length = cb.len();

            run_sim(length, args.opts, cb);
        }
    };

    Ok(())
}

impl CsvSim {
    fn get_callable(self) -> anyhow::Result<summation::InterpolateLookup> {
        let full_path = if self.path.is_relative() {
            let cwd = std::env::current_dir()
                .context("failed to retrieve current working directory")?;

            cwd.join(self.path)
        } else {
            self.path
        };

        let mut rtn = Vec::new();
        let mut builder = csv::ReaderBuilder::new();

        if self.column.is_some() {
            builder.has_headers(true);
        } else {
            builder.has_headers(false);
        }

        let mut reader = builder.from_path(&full_path)
            .context("failed to load csv file")?;

        let data_index = if let Some(column) = self.column {
            let mut maybe_index: Option<usize> = None;
            let headers = reader.headers()
                .context("failed to retrieve csv headers")?;

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
            let record = try_record.with_context(|| format!(
                "failed to retrieve csv entry. {}", index + 1
            ))?;

            let value = record.get(data_index).with_context(|| format!(
                "failed to retrieve csv entry column. {}", index + 1
            ))?;

            rtn.push(f64::from_str(value).with_context(|| format!(
                "failed to convert csv entry into float. {}", index + 1
            ))?);
        }

        Ok(InterpolateLookup::from(rtn))
    }
}

fn run_sim<T>(length: usize, opts: SimOpts, accel_lookup: T)
where
    T: summation::Callable<f64>
{
    println!("lenth: {length} step: {} iterations: {}", opts.step, opts.iterations);

    /*
    let sum_cb: summation::Summation<f64> = match opts.algo {
        AppAlgo::LeftRiemann => summation::left_riemann,
        AppAlgo::MidRiemann => summation::mid_riemann,
        AppAlgo::RightRiemann => summation::right_riemann,
        AppAlgo::Trapezoidal => summation::trapezoidal,
    };
    */

    let mut timer = time::Timing::default();

    for iter in 0..(opts.iterations) {
        let mut running_vel = 0.0;
        let mut running_pos = 0.0;
        let mut vel_table = Vec::with_capacity(length);
        let mut pos_table = Vec::with_capacity(length);
        vel_table.push(0.0);
        pos_table.push(0.0);

        let start = std::time::Instant::now();

        for sec in 1..length {
            let result = summation::trapezoidal(
                (sec - 1) as f64,
                sec as f64,
                opts.step,
                &accel_lookup,
            );

            running_vel += result;

            vel_table.push(running_vel);
        }

        let vel_lookup = InterpolateLookup::from(vel_table);

        for sec in 1..length {
            let result = summation::trapezoidal(
                (sec - 1) as f64,
                sec as f64,
                opts.step,
                &vel_lookup,
            );

            running_pos += result;

            pos_table.push(running_pos);
        }

        timer.update(start.elapsed());

        if iter == opts.iterations - 1 {
            if let Some(last) = vel_lookup.inner().last() {
                println!("final velocity: {last:+}");
            }

            if let Some(last) = pos_table.last() {
                println!("final position: {last:+}");
            }
        }
    }

    println!("time: {timer}");
}
