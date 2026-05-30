mod build_status;
mod cache;
mod cli;
mod cmd;
mod code_metrics;
mod daemon_client;
mod daemon_types;
mod docker;
mod fs;
mod git;
mod icon;
mod port_usage;
mod state;
mod test_cache;
mod todo_scanner;
mod utils;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    cli.run()
}
