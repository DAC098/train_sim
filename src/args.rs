use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::summation::{self, InterpolateLookup};

/// an application for running "train" simulations of a given acceleration
/// profile that will calculate the final velocity and position of the train
#[derive(Debug, Parser)]
pub struct App {
    /// specifies the number of threads to use for calculations
    #[arg(short, long, default_value("1"))]
    pub threads: usize,

    #[command(flatten)]
    pub opts: SimOpts,

    #[command(subcommand)]
    pub sim: SimKind,
}

/// common options between simulations
#[derive(Debug, Clone, Args)]
pub struct SimOpts {
    /// specifies the summation algorithm to use for the simulation
    #[arg(short, long, default_value("left-riemann"))]
    pub algo: AppAlgo,

    /// determines the number of times to run the program, for benchmarking
    /// purposes
    #[arg(short, long, default_value("100"))]
    pub iterations: u32,

    /// specifies the amount of steps to take in between each summation
    /// calculation
    #[arg(short, long, default_value("100"))]
    pub step: u32,
}

/// the available summation algorithms that the simulation is capable of
/// running
#[derive(Debug, Clone, ValueEnum)]
pub enum AppAlgo {
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
pub enum SimKind {
    /// runs a simulation from a given acceleration profile
    Csv(CsvSim),
}

/// options for running a simulation from a specified csv file
#[derive(Debug, Args)]
pub struct CsvSim {
    /// loads acceleration data in a specific column from the csv file
    #[arg(long)]
    pub column: Option<String>,

    /// the csv file path to load
    pub path: PathBuf,
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
    pub fn get_callable(self) -> anyhow::Result<summation::InterpolateLookup> {
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
