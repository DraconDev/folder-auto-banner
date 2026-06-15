use anyhow::Result;
use clap::Parser;
use folder_auto_banner::cli;
use std::env;

/// Known subcommands that clap should parse.
const KNOWN_SUBCOMMANDS: &[&str] = &["banner", "env", "install", "config", "daemon", "help"];

/// Single-character short flags that can be used as lazy flags
/// (e.g. `f t` is equivalent to `f -t`).
///
/// Rule: no fallback. `f t` ALWAYS means `-t`. To show a banner
/// for a file called `t`, use `./t` or an absolute path.
const LAZY_FLAGS: &[char] = &[
    '1', // --oneline
    'a', // --hidden
    'c', // --compact
    'D', // --only-dirs
    'e', // --edit
    'f', // --filter
    'G', // --gitsort
    'L', // --level
    'm', // --max
    'r', // --reverse
    'R', // --recursive
    'S', // --sizesort
    't', // --timesort
    'U', // --no-sort
    'v', // --verbose
    'x', // --run
    'X', // --extensionsort
];

fn is_lazy_flag(arg: &str) -> Option<char> {
    let mut chars = arg.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None; // not single char
    }
    if LAZY_FLAGS.contains(&c) {
        Some(c)
    } else {
        None
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    // Find the first non-flag argument (skip --debug, -d, etc.)
    let first_non_flag = args.iter().find(|a| !a.starts_with('-'));

    if let Some(arg) = first_non_flag {
        // If it's a number → navigate (route to banner subcommand)
        // NOTE: numbers take precedence over lazy flags, so `f 1` navigates
        // to item 1, not --oneline.
        if arg.parse::<usize>().is_ok() {
            let mut new_args = vec!["f".to_string(), "banner".to_string()];
            new_args.extend(args);
            let cli = cli::Cli::parse_from(new_args);
            return cli.run();
        }

        // If it's a known subcommand → let clap handle it
        if KNOWN_SUBCOMMANDS.contains(&arg.as_str()) {
            let cli = cli::Cli::parse();
            return cli.run();
        }

        // If it's a single-char lazy flag (e.g. `t` → `-t`) → expand it.
        // No fallback: `f t` ALWAYS means sort by time. Use `./t` for a
        // file/dir called `t`.
        if let Some(c) = is_lazy_flag(arg) {
            let mut new_args: Vec<String> = vec!["f".to_string(), "banner".to_string()];
            for a in &args {
                if a == arg {
                    new_args.push(format!("-{}", c));
                } else {
                    new_args.push(a.clone());
                }
            }
            let cli = cli::Cli::parse_from(new_args);
            return cli.run();
        }

        // Otherwise it's a path → route to banner subcommand
        let mut new_args = vec!["f".to_string(), "banner".to_string()];
        new_args.extend(args);
        let cli = cli::Cli::parse_from(new_args);
        return cli.run();
    }

    // No args or only flags → let clap handle it
    let cli = cli::Cli::parse();
    cli.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_lazy_flag_single_char() {
        assert_eq!(is_lazy_flag("t"), Some('t'));
        assert_eq!(is_lazy_flag("S"), Some('S'));
        assert_eq!(is_lazy_flag("X"), Some('X'));
        assert_eq!(is_lazy_flag("a"), Some('a'));
    }

    #[test]
    fn test_is_lazy_flag_rejects_multi_char() {
        assert_eq!(is_lazy_flag("tt"), None);
        assert_eq!(is_lazy_flag("abc"), None);
    }

    #[test]
    fn test_is_lazy_flag_rejects_unknown() {
        // 'z' is not a known flag
        assert_eq!(is_lazy_flag("z"), None);
        // 'Q' is not a known flag
        assert_eq!(is_lazy_flag("Q"), None);
    }

    #[test]
    fn test_is_lazy_flag_rejects_empty() {
        assert_eq!(is_lazy_flag(""), None);
    }
}
