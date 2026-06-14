use anyhow::Result;
use clap::Parser;
use folder_auto_banner::cli;
use std::env;
use std::time::Instant;

/// Known subcommands that clap should parse.
const KNOWN_SUBCOMMANDS: &[&str] = &["banner", "env", "install", "config", "daemon", "help"];

fn main() -> Result<()> {
    let _t_main = Instant::now();
    let args: Vec<String> = env::args().skip(1).collect();
    let _t_args = Instant::now();

    // Find the first non-flag argument (skip --debug, -d, etc.)
    let first_non_flag = args.iter().find(|a| !a.starts_with('-'));
    let _t_route = Instant::now();

    if let Some(arg) = first_non_flag {
        // If it's a number → navigate (route to banner subcommand)
        if arg.parse::<usize>().is_ok() {
            let mut new_args = vec!["f".to_string(), "banner".to_string()];
            new_args.extend(args);
            let cli = cli::Cli::parse_from(new_args);
            let _t_clap = Instant::now();
            let r = cli.run();
            if std::env::var("FAB_PROFILE").is_ok() {
                eprintln!("[FAB_PROFILE] main: args={:?} route={:?} clap={:?} run={:?} total={:?}",
                    _t_args - _t_main,
                    _t_route - _t_args,
                    _t_clap - _t_route,
                    _t_clap.elapsed(),
                    _t_main.elapsed());
            }
            return r;
        }

        // If it's a known subcommand → let clap handle it
        if KNOWN_SUBCOMMANDS.contains(&arg.as_str()) {
            let cli = cli::Cli::parse();
            let _t_clap = Instant::now();
            let r = cli.run();
            if std::env::var("FAB_PROFILE").is_ok() {
                eprintln!("[FAB_PROFILE] main: args={:?} route={:?} clap={:?} run={:?} total={:?}",
                    _t_args - _t_main,
                    _t_route - _t_args,
                    _t_clap - _t_route,
                    _t_clap.elapsed(),
                    _t_main.elapsed());
            }
            return r;
        }

        // Otherwise it's a path → route to banner subcommand
        let mut new_args = vec!["f".to_string(), "banner".to_string()];
        new_args.extend(args);
        let cli = cli::Cli::parse_from(new_args);
        let _t_clap = Instant::now();
        let r = cli.run();
        if std::env::var("FAB_PROFILE").is_ok() {
            eprintln!("[FAB_PROFILE] main: args={:?} route={:?} clap={:?} run={:?} total={:?}",
                _t_args - _t_main,
                _t_route - _t_args,
                _t_clap - _t_route,
                _t_clap.elapsed(),
                _t_main.elapsed());
        }
        return r;
    }

    // No args or only flags → let clap handle it
    let cli = cli::Cli::parse();
    let _t_clap = Instant::now();
    let r = cli.run();
    if std::env::var("FAB_PROFILE").is_ok() {
        eprintln!("[FAB_PROFILE] main: args={:?} route={:?} clap={:?} run={:?} total={:?}",
            _t_args - _t_main,
            _t_route - _t_args,
            _t_clap - _t_route,
            _t_clap.elapsed(),
            _t_main.elapsed());
    }
    r
}
