use clap::{crate_authors, Parser};

#[derive(Parser)]
#[command(
    name = "youdusa",
    author = crate_authors!(",\n"),
    version,
    about = "Generate template for Medusa fuzzing campaigns",
    long_about = None,

    help_template = concat!(
include_str!("ascii_art.txt"),
"Made with ♥ by Wonderland (https://defi.sucks)\n
╔══════════════════════════════════╗\n\
║    \x1B[31mMedusa Template Generator\x1B[0m     ║\n\
╚══════════════════════════════════╝\n\
\n\
{about}\n\
\n\
{usage-heading} {usage}\n\
\n\
{all-args}\n\
\n\
Authors:{author-section}
Version: {version}
\n\
For more information, visit: https://github.com/defi-wonderland/medusa-gen-rs\n",
))]
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
