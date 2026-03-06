use std::path::PathBuf;
use std::process::ExitCode;

use clap::{ArgAction, Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
enum Language {
    #[value(name = "c", alias = "C")]
    C,
    #[value(name = "c++", alias = "C++", alias = "cpp")]
    Cxx,
    #[value(name = "cython", alias = "Cython")]
    Cython,
}

#[derive(Debug, Clone, ValueEnum)]
enum Style {
    #[value(name = "both", alias = "Both")]
    Both,
    #[value(name = "tag", alias = "Tag")]
    Tag,
    #[value(name = "type", alias = "Type")]
    Type,
}

/// Generate C/C++ headers from a Rust crate using rustdoc-json.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Path to the Rust crate directory (defaults to current directory).
    input: Option<PathBuf>,

    /// Increase verbosity (can be repeated: -v, -vv, -vvv).
    #[arg(short, action = ArgAction::Count)]
    verbose: u8,

    /// Suppress all output.
    #[arg(short, long)]
    quiet: bool,

    /// Verify that the generated bindings match the existing output file.
    #[arg(long)]
    verify: bool,

    /// Path to a TOML configuration file.
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Target language for the generated bindings.
    #[arg(short, long)]
    lang: Option<Language>,

    /// Add C++ compatibility features to the generated C header.
    #[arg(long)]
    cpp_compat: bool,

    /// Declaration style for generated types.
    #[arg(short, long)]
    style: Option<Style>,

    /// Output file path (defaults to stdout).
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Path to a pre-generated `cargo metadata` JSON file.
    #[arg(long)]
    metadata: Option<PathBuf>,
}

fn main() -> ExitCode {
    let _cli = Cli::parse();
    eprintln!("TODO: cheadergen is not yet implemented");
    ExitCode::SUCCESS
}
