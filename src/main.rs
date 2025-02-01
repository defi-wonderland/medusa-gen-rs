use anyhow::Result;
use clap::Parser;
use medusa_gen::cli::Args;

fn main() -> Result<()> {
    let args = Args::parse();

    medusa_gen::generate_test_suite(&args)?;

    Ok(())
}