mod analysis;
mod cli;
mod codegen;
mod config;
mod constant_item;
mod indexing;
mod metadata;
mod static_item;
mod topological_sort;

use std::process::ExitCode;

use rustdoc_processor::CrateCollection;

use crate::indexing::CheadergenIndexer;

type Collection = CrateCollection<CheadergenIndexer>;

fn main() -> ExitCode {
    cli::run()
}
