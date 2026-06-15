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
    'a', // --hidden
    'c', // --compact
    'D', // --only-dirs
    'e', // --edit
    'f', // --filter
    'G', // --gitsort
    'L', // --level
    'm', // --max
    'o', // --oneline
    'r', // --reverse
    'R', // --recursive
    'S', // --sizesort
    't', // --timesort
    'U', // --no-sort
    'v', // --verbose
    'x', // --run
    'X', // --extensionsort
];

/// Lowercase aliases for uppercase flags. `f s` is equivalent to
/// `f S` (sort by size), `f g` to `f G` (sort by git), etc.
///
/// Only letters NOT already in `LAZY_FLAGS` can be aliased:
/// `r` is already `--reverse`, so it is NOT aliased to `R`
/// (--recursive). `x` and `X` are NOT aliased — they are
/// distinct flags (`x` = --run, `X` = --extensionsort).
const LOWERCASE_ALIASES: &[(char, char)] = &[
    ('s', 'S'), // --sizesort
    ('g', 'G'), // --gitsort
    ('d', 'D'), // --only-dirs
    ('l', 'L'), // --level
    ('u', 'U'), // --no-sort
];

fn is_lazy_flag(arg: &str) -> Option<char> {
    let mut chars = arg.chars();
    let c = chars.next()?;
    if chars.next().is_some() {
        return None; // not single char
    }
    // Canonical list first (preserves case for x vs X)
    if LAZY_FLAGS.contains(&c) {
        return Some(c);
    }
    // Lowercase aliases for uppercase flags (e.g. `s` → `S`)
    for &(from, to) in LOWERCASE_ALIASES {
        if c == from {
            return Some(to);
        }
    }
    None
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
        // file/dir called `t`. For chained flags, use explicit
        // `f -trc` instead of `f trc` to avoid ambiguity with paths.
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

    #[test]
    fn test_is_lazy_flag_case_insensitive_uppercase() {
        // Uppercase flags should also accept lowercase (where the
        // lowercase letter is not already a canonical flag).
        assert_eq!(is_lazy_flag("s"), Some('S')); // sizesort
        assert_eq!(is_lazy_flag("g"), Some('G')); // gitsort
        assert_eq!(is_lazy_flag("d"), Some('D')); // only-dirs
        assert_eq!(is_lazy_flag("l"), Some('L')); // level
        assert_eq!(is_lazy_flag("u"), Some('U')); // no-sort
                                                  // `r` is already canonical (--reverse), so it stays as `r`
        assert_eq!(is_lazy_flag("r"), Some('r')); // --reverse (not aliased to R)
    }

    #[test]
    fn test_is_lazy_flag_preserves_x_case() {
        // x and X are distinct — x is --run, X is --extensionsort
        assert_eq!(is_lazy_flag("x"), Some('x')); // --run
        assert_eq!(is_lazy_flag("X"), Some('X')); // --extensionsort
    }

    #[test]
    fn test_is_lazy_flag_lowercase_still_works() {
        // Lowercase flags should still work
        assert_eq!(is_lazy_flag("t"), Some('t')); // timesort
        assert_eq!(is_lazy_flag("a"), Some('a')); // hidden
        assert_eq!(is_lazy_flag("c"), Some('c')); // compact
    }
}
