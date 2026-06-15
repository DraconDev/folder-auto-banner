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

/// Resolve a single character to its canonical lazy-flag form
/// (e.g. `s` → `S`). Returns `None` if the char is not a lazy flag.
fn resolve_lazy_flag_char(c: char) -> Option<char> {
    if LAZY_FLAGS.contains(&c) {
        return Some(c);
    }
    for &(from, to) in LOWERCASE_ALIASES {
        if c == from {
            return Some(to);
        }
    }
    None
}

/// Expand a multi-character arg into a list of canonical lazy flags.
/// Returns `Some(Vec<char>)` if EVERY character in `arg` resolves to
/// a lazy flag, `None` otherwise.
///
/// No fallback: `f trc` ALWAYS means `-t -r -c`. To show a banner
/// for a path, the path must start with `./`, `/`, or `~` (explicit
/// path indicators). Bare words are always lazy-flag chains.
fn expand_lazy_flags(arg: &str) -> Option<Vec<char>> {
    if arg.is_empty() {
        return None;
    }
    let mut result = Vec::with_capacity(arg.len());
    for c in arg.chars() {
        result.push(resolve_lazy_flag_char(c)?);
    }
    Some(result)
}

/// Returns true if the arg looks like an explicit path (starts with
/// `.`, `/`, or `~`). Used to disambiguate paths from lazy-flag chains
/// in the routing logic.
fn is_explicit_path(arg: &str) -> bool {
    if arg.is_empty() {
        return false;
    }
    let first = arg.chars().next().unwrap();
    first == '.' || first == '/' || first == '~'
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

        // If it's an explicit path (starts with `.`, `/`, or `~`) → path
        // No fallback: bare words without explicit path indicators are
        // always lazy-flag chains, never paths.
        if is_explicit_path(arg) {
            let mut new_args = vec!["f".to_string(), "banner".to_string()];
            new_args.extend(args);
            let cli = cli::Cli::parse_from(new_args);
            return cli.run();
        }

        // If it's a single-char lazy flag (e.g. `t` → `-t`) or a chain
        // of lazy flags (e.g. `trc` → `-t -r -c`) → expand it.
        // No fallback: `f trc` ALWAYS means time+reverse+compact. Use
        // `./trc` for a file/dir called `trc`.
        if let Some(flags) = expand_lazy_flags(arg) {
            let mut new_args: Vec<String> = vec!["f".to_string(), "banner".to_string()];
            for a in &args {
                if a == arg {
                    for c in &flags {
                        new_args.push(format!("-{}", c));
                    }
                } else {
                    new_args.push(a.clone());
                }
            }
            let cli = cli::Cli::parse_from(new_args);
            return cli.run();
        }

        // Bare word that's not a lazy flag chain (e.g. contains a digit
        // or non-flag char). This is an unusual case — we treat it as
        // a path. The user should use `./` or `/` for explicit paths.
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
    fn test_expand_lazy_flags_single_char() {
        // Single chars work (backward compat with single-char lazy flags)
        assert_eq!(expand_lazy_flags("t"), Some(vec!['t']));
        assert_eq!(expand_lazy_flags("S"), Some(vec!['S']));
        assert_eq!(expand_lazy_flags("X"), Some(vec!['X']));
        assert_eq!(expand_lazy_flags("a"), Some(vec!['a']));
    }

    #[test]
    fn test_expand_lazy_flags_all_flags() {
        // All characters are lazy flags → expand each
        assert_eq!(expand_lazy_flags("trc"), Some(vec!['t', 'r', 'c']));
        assert_eq!(expand_lazy_flags("tS"), Some(vec!['t', 'S']));
        assert_eq!(expand_lazy_flags("aG"), Some(vec!['a', 'G']));
    }

    #[test]
    fn test_expand_lazy_flags_rejects_non_flag() {
        // Any non-flag character → None
        assert_eq!(expand_lazy_flags("z"), None); // z is not a flag
        assert_eq!(expand_lazy_flags("tz"), None);
        assert_eq!(expand_lazy_flags("trz"), None);
        assert_eq!(expand_lazy_flags("Downloads"), None); // path
        assert_eq!(expand_lazy_flags("Q"), None); // uppercase Q not a flag
        assert_eq!(expand_lazy_flags(""), None); // empty
    }

    #[test]
    fn test_expand_lazy_flags_case_insensitive_uppercase() {
        // Uppercase flags should also accept lowercase (where the
        // lowercase letter is not already a canonical flag).
        assert_eq!(expand_lazy_flags("s"), Some(vec!['S'])); // sizesort
        assert_eq!(expand_lazy_flags("g"), Some(vec!['G'])); // gitsort
        assert_eq!(expand_lazy_flags("d"), Some(vec!['D'])); // only-dirs
        assert_eq!(expand_lazy_flags("l"), Some(vec!['L'])); // level
        assert_eq!(expand_lazy_flags("u"), Some(vec!['U'])); // no-sort
                                                             // `r` is already canonical (--reverse), so it stays as `r`
        assert_eq!(expand_lazy_flags("r"), Some(vec!['r'])); // --reverse (not aliased to R)
    }

    #[test]
    fn test_expand_lazy_flags_preserves_x_case() {
        // x and X are distinct — x is --run, X is --extensionsort
        assert_eq!(expand_lazy_flags("x"), Some(vec!['x'])); // --run
        assert_eq!(expand_lazy_flags("X"), Some(vec!['X'])); // --extensionsort
    }

    #[test]
    fn test_is_explicit_path() {
        // Paths starting with `.`, `/`, or `~` are explicit
        assert!(is_explicit_path("./Downloads"));
        assert!(is_explicit_path("/home/user"));
        assert!(is_explicit_path("~/Downloads"));
        assert!(is_explicit_path("../sibling"));
        // Bare words are NOT explicit paths — they're lazy flag chains
        assert!(!is_explicit_path("Downloads"));
        assert!(!is_explicit_path("trc"));
        assert!(!is_explicit_path("t"));
        assert!(!is_explicit_path(""));
    }
}
