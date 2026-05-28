#![allow(dead_code)]

mod cli;
mod cmd;
mod git;
mod fs;
mod icon;
mod state;
mod cache;
mod build_status;
mod todo_scanner;
mod port_usage;
mod docker;
mod code_metrics;

use clap::Parser;
use anyhow::Result;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    cli.run()
}