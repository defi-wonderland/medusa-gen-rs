use clap::Parser;

#[derive(Parser)]
#[command(version, about = "Generate template for Medusa fuzzing campaigns")]
pub struct Args {
    /// Solidity version
    #[arg(short, long, default_value = "0.8.23")]
    pub solc: String,

    /// Number of handler to generate
    #[arg(short = 'n', long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..))]
    pub nb_handlers: u8,

    /// Number of properties contract to generate
    #[arg(short = 'p', long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..))]
    pub nb_properties: u8,

    /// Overwrite existing files
    #[arg(short, long, default_value_t = false)]
    pub overwrite: bool,
}
