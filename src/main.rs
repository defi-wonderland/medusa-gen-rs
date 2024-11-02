mod cli;
mod gen;
mod types;

use anyhow::{Context, Result};
use clap::Parser;

use crate::cli::Args;
use crate::gen::{generate_contract, generate_family};
use crate::types::ContractType;

fn main() -> Result<()> {
    let args = Args::parse();

    generate_family(&args, ContractType::Handler)
        .context("Failed to generate handlers contracts")?;

    generate_family(&args, ContractType::Property)
        .context("Failed to generated properties contracts")?;

    generate_contract(&args, ContractType::EntryPoint)
        .context("Failed to generate entry point contract")?;

    generate_contract(&args, ContractType::Setup).context("Failed to generate setup contract")?;

    Ok(())
}
