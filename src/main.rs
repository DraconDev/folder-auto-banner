#![allow(dead_code)]

mod cli;
mod cmd;
mod git;
mod fs;
mod icon;
mod state;

use clap::Parser;
use anyhow::Result;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    cli.run()
}