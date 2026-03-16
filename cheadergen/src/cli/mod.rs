mod generate;
mod input;
mod warm_cache;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
/// Generate C/C++ headers from a Rust crate using rustdoc-json.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Generate C/C++ headers from a Rust crate.
    Generate(generate::GenerateArgs),
    /// Pre-warm the rustdoc JSON cache for all workspace members.
    WarmCache(warm_cache::WarmCacheArgs),
    /// Configuration utilities.
    Config(ConfigArgs),
}

#[derive(Debug, Parser)]
struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommand,
}

#[derive(Debug, clap::Subcommand)]
enum ConfigCommand {
    /// Translate a cbindgen config file into cheadergen format.
    Translate(TranslateArgs),
}

#[derive(Debug, Parser)]
struct TranslateArgs {
    /// Path to the cbindgen config file.
    #[arg(long)]
    from: PathBuf,
    /// Path to write the cheadergen config file.
    #[arg(long)]
    to: PathBuf,
}

/// Parses CLI arguments and dispatches to the appropriate subcommand handler.
pub fn run() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Generate(args) => {
            if let Err(e) = generate::generate(&args) {
                eprintln!("Error: {e:?}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Command::WarmCache(args) => {
            if let Err(e) = warm_cache::warm_cache(&args) {
                eprintln!("Error: {e:?}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Command::Config(args) => match args.command {
            ConfigCommand::Translate(args) => {
                if let Err(e) = crate::config::cbindgen::translate(&args.from, &args.to) {
                    eprintln!("Error: {e}");
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            }
        },
    }
}
