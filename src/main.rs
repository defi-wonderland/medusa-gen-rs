mod cli;
mod gen;
mod types;

use anyhow::Result;
use clap::Parser;

use crate::cli::Args;
use crate::gen::generate_test_suite;
use crate::types::ContractType;

fn main() -> Result<()> {
    let args = Args::parse();

    generate_test_suite(&args)?;

    Ok(())
}
