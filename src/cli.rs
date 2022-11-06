use clap::{Parser, ValueEnum};
use std::path::PathBuf;

lazy_static! {
    pub static ref ARGS: Cli = Cli::parse();
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, ValueEnum)]
pub enum Algorithm {
    SuffixArray,
    BWT,
    BiBWT,
}

#[derive(Parser)]
#[clap(author, about)]
pub struct Cli {
    /// Algorithm to run
    #[clap(value_enum)]
    pub algorithm: Algorithm,

    /// String to run on.
    #[clap()]
    pub input: Option<String>,

    /// Query string for BWT.
    #[clap(short, long)]
    pub query: Option<String>,

    /// Where to optionally save image files.
    #[clap(short, long, parse(from_os_str))]
    pub save: Option<PathBuf>,
}
