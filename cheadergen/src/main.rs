mod analysis;
mod cli;
mod codegen;
mod config;
mod constant_item;
mod metadata;
mod static_item;
mod topological_sort;

use std::process::ExitCode;

use rustdoc_processor::CrateCollection;
use rustdoc_processor::indexing::NoAnnotations;

type Collection = CrateCollection<NoAnnotations>;

fn main() -> ExitCode {
    cli::run()
}
