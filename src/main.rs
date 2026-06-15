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

/// Flags that take a value (the value is the next argument).
/// Used for smart expansion of chained lazy flags.
const VALUE_TAKING_FLAGS: &[char] = &[
    'm', // --max <MAX> (usize)
    'f', // --filter <PATTERN> (String)
    'L', // --level <LEVEL> (usize)
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

    // If the user passed any explicit flags (starting with `-`), let clap
    // handle the parsing directly. The lazy flag chain system is only for
    // bare-word invocations like `f t`, `f trc`, `f m 10`.
    let has_explicit_flag = args.iter().any(|a| a.starts_with('-'));
    if has_explicit_flag {
        let cli = cli::Cli::parse();
        return cli.run();
    }

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
        //
        // Value-taking flags in the chain consume the next arg as their
        // value. E.g. `f mL 10 2 path` → `-m 10 -L 2 path`. The values
        // are assigned in chain order to value-taking flags.
        if let Some(flags) = expand_lazy_flags(arg) {
            let arg_pos = args.iter().position(|a| a == arg).unwrap();
            let mut new_args: Vec<String> = vec!["f".to_string(), "banner".to_string()];

            // Args before the chain
            for a in &args[..arg_pos] {
                new_args.push(a.clone());
            }

            // Expand the chain, consuming values for value-taking flags
            let mut value_idx = arg_pos + 1;
            for c in &flags {
                new_args.push(format!("-{}", c));
                if VALUE_TAKING_FLAGS.contains(c) {
                    // Consume the next arg as the value
                    if value_idx < args.len() {
                        new_args.push(args[value_idx].clone());
                        value_idx += 1;
                    }
                }
            }

            // Args after the consumed values (the path and any remaining args)
            for a in &args[value_idx..] {
                new_args.push(a.clone());
            }

            let cli = cli::Cli::parse_from(new_args);
            return cli.run();
        }

        // Bare word that's not a lazy flag chain (e.g. contains a digit
        // or non-flag char). This is an unusual case — we treat it as
        // a path. The user should use `./` or `/` for explicit paths.
        // If the arg looks like it MIGHT have been intended as a lazy
        // chain (only alpha chars, but with at least one valid flag),
        // give a helpful error explaining the lazy flag system.
        if arg.chars().all(|c| c.is_ascii_alphabetic()) {
            if arg.chars().any(|c| resolve_lazy_flag_char(c).is_some()) {
                eprintln!(
                    "error: '{}' is not a valid lazy flag chain. \
                     Valid flags: a, c, D, e, f, G, L, m, o, r, R, S, t, U, v, x, X. \
                     Use './{}' to treat it as a path.",
                    arg, arg
                );
            } else {
                eprintln!(
                    "error: '{}' is not a valid lazy flag. \
                     Valid flags: a, c, D, e, f, G, L, m, o, r, R, S, t, U, v, x, X. \
                     Use './{}' to treat it as a path.",
                    arg, arg
                );
            }
            std::process::exit(2);
        }
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

    // ===== resolve_lazy_flag_char tests =====

    #[test]
    fn test_resolve_all_17_canonical_flags() {
        // Every entry in LAZY_FLAGS must resolve to itself
        for &c in LAZY_FLAGS {
            assert_eq!(
                resolve_lazy_flag_char(c),
                Some(c),
                "flag {:?} should resolve to itself",
                c
            );
        }
    }

    #[test]
    fn test_resolve_all_5_lowercase_aliases() {
        // Every lowercase alias must resolve to its canonical uppercase
        for &(from, to) in LOWERCASE_ALIASES {
            assert_eq!(
                resolve_lazy_flag_char(from),
                Some(to),
                "alias {:?} should resolve to {:?}",
                from,
                to
            );
        }
    }

    #[test]
    fn test_resolve_rejects_non_flags() {
        // Letters that are not in LAZY_FLAGS and not in LOWERCASE_ALIASES
        // must resolve to None. Check all 26 letters.
        let mut valid_chars: std::collections::HashSet<char> = LAZY_FLAGS.iter().copied().collect();
        for &(from, to) in LOWERCASE_ALIASES {
            valid_chars.insert(from);
            valid_chars.insert(to);
        }
        for c in 'a'..='z' {
            if !valid_chars.contains(&c) {
                assert_eq!(
                    resolve_lazy_flag_char(c),
                    None,
                    "char {:?} should not resolve",
                    c
                );
            }
        }
        for c in 'A'..='Z' {
            if !valid_chars.contains(&c) {
                assert_eq!(
                    resolve_lazy_flag_char(c),
                    None,
                    "char {:?} should not resolve",
                    c
                );
            }
        }
    }

    #[test]
    fn test_resolve_non_ascii_returns_none() {
        // Non-ASCII characters must not resolve
        assert_eq!(resolve_lazy_flag_char('é'), None);
        assert_eq!(resolve_lazy_flag_char('ñ'), None);
        assert_eq!(resolve_lazy_flag_char('中'), None);
        assert_eq!(resolve_lazy_flag_char('🦀'), None);
    }

    #[test]
    fn test_resolve_digits_and_symbols() {
        // Digits and symbols must not resolve
        assert_eq!(resolve_lazy_flag_char('0'), None);
        assert_eq!(resolve_lazy_flag_char('9'), None);
        assert_eq!(resolve_lazy_flag_char('-'), None);
        assert_eq!(resolve_lazy_flag_char('_'), None);
    }

    // ===== expand_lazy_flags tests =====

    #[test]
    fn test_expand_empty_string() {
        assert_eq!(expand_lazy_flags(""), None);
    }

    #[test]
    fn test_expand_single_char_each_canonical() {
        for &c in LAZY_FLAGS {
            assert_eq!(
                expand_lazy_flags(&c.to_string()),
                Some(vec![c]),
                "single char {:?} failed",
                c
            );
        }
    }

    #[test]
    fn test_expand_single_char_each_alias() {
        for &(from, to) in LOWERCASE_ALIASES {
            assert_eq!(
                expand_lazy_flags(&from.to_string()),
                Some(vec![to]),
                "alias {:?}→{:?} failed",
                from,
                to
            );
        }
    }

    #[test]
    fn test_expand_two_char_chains() {
        // Test all valid 2-char combinations
        assert_eq!(expand_lazy_flags("tr"), Some(vec!['t', 'r']));
        assert_eq!(expand_lazy_flags("tS"), Some(vec!['t', 'S']));
        assert_eq!(expand_lazy_flags("ta"), Some(vec!['t', 'a']));
        assert_eq!(expand_lazy_flags("tc"), Some(vec!['t', 'c']));
        assert_eq!(expand_lazy_flags("rS"), Some(vec!['r', 'S']));
        assert_eq!(expand_lazy_flags("aR"), Some(vec!['a', 'R']));
        assert_eq!(expand_lazy_flags("GS"), Some(vec!['G', 'S']));
        assert_eq!(expand_lazy_flags("oR"), Some(vec!['o', 'R']));
        assert_eq!(expand_lazy_flags("Dt"), Some(vec!['D', 't']));
        assert_eq!(expand_lazy_flags("Rc"), Some(vec!['R', 'c']));
    }

    #[test]
    fn test_expand_three_char_chains() {
        assert_eq!(expand_lazy_flags("trc"), Some(vec!['t', 'r', 'c']));
        assert_eq!(expand_lazy_flags("tSr"), Some(vec!['t', 'S', 'r']));
        assert_eq!(expand_lazy_flags("aRc"), Some(vec!['a', 'R', 'c']));
        assert_eq!(expand_lazy_flags("GSr"), Some(vec!['G', 'S', 'r']));
        assert_eq!(expand_lazy_flags("Dta"), Some(vec!['D', 't', 'a']));
    }

    #[test]
    fn test_expand_four_char_chains() {
        assert_eq!(expand_lazy_flags("trca"), Some(vec!['t', 'r', 'c', 'a']));
        assert_eq!(expand_lazy_flags("tSra"), Some(vec!['t', 'S', 'r', 'a']));
        assert_eq!(expand_lazy_flags("trcS"), Some(vec!['t', 'r', 'c', 'S']));
    }

    #[test]
    fn test_expand_value_taking_chains() {
        // Value-taking flags mixed with boolean flags
        assert_eq!(expand_lazy_flags("mL"), Some(vec!['m', 'L']));
        assert_eq!(expand_lazy_flags("Lm"), Some(vec!['L', 'm']));
        assert_eq!(expand_lazy_flags("tSm"), Some(vec!['t', 'S', 'm']));
        assert_eq!(expand_lazy_flags("mLf"), Some(vec!['m', 'L', 'f']));
        assert_eq!(expand_lazy_flags("tSmL"), Some(vec!['t', 'S', 'm', 'L']));
    }

    #[test]
    fn test_expand_mixed_case_aliases() {
        // Mixed case: some canonical, some alias
        assert_eq!(expand_lazy_flags("sG"), Some(vec!['S', 'G']));
        assert_eq!(expand_lazy_flags("sd"), Some(vec!['S', 'D']));
        assert_eq!(expand_lazy_flags("gl"), Some(vec!['G', 'L']));
        assert_eq!(expand_lazy_flags("ud"), Some(vec!['U', 'D']));
    }

    #[test]
    fn test_expand_rejects_single_non_flag() {
        // Each non-flag letter should reject
        for c in 'a'..='z' {
            if !LAZY_FLAGS.contains(&c) && !LOWERCASE_ALIASES.iter().any(|&(f, _)| f == c) {
                assert_eq!(
                    expand_lazy_flags(&c.to_string()),
                    None,
                    "char {:?} should reject",
                    c
                );
            }
        }
    }

    #[test]
    fn test_expand_rejects_mixed_valid_invalid() {
        // Chain with at least one invalid char must reject
        assert_eq!(expand_lazy_flags("tz"), None);
        assert_eq!(expand_lazy_flags("trz"), None);
        assert_eq!(expand_lazy_flags("ztr"), None);
        assert_eq!(expand_lazy_flags("tqr"), None);
        assert_eq!(expand_lazy_flags("tn"), None);
        assert_eq!(expand_lazy_flags("tp"), None);
        assert_eq!(expand_lazy_flags("tw"), None);
        assert_eq!(expand_lazy_flags("tb"), None);
    }

    #[test]
    fn test_expand_rejects_digits() {
        assert_eq!(expand_lazy_flags("1"), None);
        assert_eq!(expand_lazy_flags("t1"), None);
        assert_eq!(expand_lazy_flags("123"), None);
    }

    #[test]
    fn test_expand_rejects_special_chars() {
        assert_eq!(expand_lazy_flags("t-"), None);
        assert_eq!(expand_lazy_flags("t."), None);
        assert_eq!(expand_lazy_flags("t/"), None);
        assert_eq!(expand_lazy_flags("t~"), None);
        assert_eq!(expand_lazy_flags("t "), None); // space
    }

    #[test]
    fn test_expand_rejects_unicode() {
        assert_eq!(expand_lazy_flags("t\u{00e9}"), None); // é
        assert_eq!(expand_lazy_flags("\u{4e2d}"), None); // 中
    }

    #[test]
    fn test_expand_x_vs_upper_x_distinct() {
        // x and X must remain distinct (x=run, X=extensionsort)
        assert_eq!(expand_lazy_flags("x"), Some(vec!['x']));
        assert_eq!(expand_lazy_flags("X"), Some(vec!['X']));
        assert_eq!(expand_lazy_flags("xX"), Some(vec!['x', 'X']));
        assert_eq!(expand_lazy_flags("Xx"), Some(vec!['X', 'x']));
    }

    #[test]
    fn test_expand_r_not_aliased_to_r() {
        // r is canonical for --reverse, NOT aliased to R (which doesn't exist)
        assert_eq!(expand_lazy_flags("r"), Some(vec!['r']));
        // R is canonical for --recursive
        assert_eq!(expand_lazy_flags("R"), Some(vec!['R']));
    }

    #[test]
    fn test_expand_long_chain() {
        // 10-char chain of unique flags
        assert_eq!(
            expand_lazy_flags("tacSDGvRrx"),
            Some(vec!['t', 'a', 'c', 'S', 'D', 'G', 'v', 'R', 'r', 'x'])
        );
    }

    // ===== is_explicit_path tests =====

    #[test]
    fn test_explicit_path_dot_prefix() {
        assert!(is_explicit_path("."));
        assert!(is_explicit_path(".."));
        assert!(is_explicit_path("./Downloads"));
        assert!(is_explicit_path("../sibling"));
        assert!(is_explicit_path(".hidden"));
    }

    #[test]
    fn test_explicit_path_slash_prefix() {
        assert!(is_explicit_path("/"));
        assert!(is_explicit_path("/tmp"));
        assert!(is_explicit_path("/home/user"));
        assert!(is_explicit_path("/usr/local/bin"));
    }

    #[test]
    fn test_explicit_path_tilde_prefix() {
        assert!(is_explicit_path("~"));
        assert!(is_explicit_path("~/"));
        assert!(is_explicit_path("~/Downloads"));
        assert!(is_explicit_path("~/.config"));
    }

    #[test]
    fn test_explicit_path_bare_words() {
        // Bare words without prefix are NOT explicit paths
        assert!(!is_explicit_path("Downloads"));
        assert!(!is_explicit_path("Documents"));
        assert!(!is_explicit_path("trc"));
        assert!(!is_explicit_path("src"));
        assert!(!is_explicit_path("home"));
    }

    #[test]
    fn test_explicit_path_empty() {
        assert!(!is_explicit_path(""));
    }

    #[test]
    fn test_explicit_path_unicode() {
        // Unicode chars that aren't `.`, `/`, or `~` are not explicit paths
        assert!(!is_explicit_path("é"));
        assert!(!is_explicit_path("中"));
        assert!(!is_explicit_path("🦀"));
    }

    #[test]
    fn test_explicit_path_dollar_env_var() {
        // $ is not a recognized explicit path prefix
        // (shell would expand $VAR before f sees it)
        assert!(!is_explicit_path("$HOME"));
        assert!(!is_explicit_path("$VAR/path"));
    }

    // ===== Constants integrity tests =====

    #[test]
    fn test_lazy_flags_count_is_17() {
        assert_eq!(LAZY_FLAGS.len(), 17, "LAZY_FLAGS should have 17 entries");
    }

    #[test]
    fn test_lowercase_aliases_count_is_5() {
        assert_eq!(
            LOWERCASE_ALIASES.len(),
            5,
            "LOWERCASE_ALIASES should have 5 entries"
        );
    }

    #[test]
    fn test_value_taking_flags_count_is_3() {
        assert_eq!(
            VALUE_TAKING_FLAGS.len(),
            3,
            "VALUE_TAKING_FLAGS should have 3 entries (m, f, L)"
        );
    }

    #[test]
    fn test_value_taking_flags_are_m_f_l() {
        let mut v: Vec<char> = VALUE_TAKING_FLAGS.to_vec();
        v.sort();
        assert_eq!(
            v,
            vec!['L', 'f', 'm'],
            "VALUE_TAKING_FLAGS should be {{m, f, L}}"
        );
    }

    #[test]
    fn test_value_taking_flags_are_in_lazy_flags() {
        // Every value-taking flag must also be a canonical lazy flag
        for &c in VALUE_TAKING_FLAGS {
            assert!(
                LAZY_FLAGS.contains(&c),
                "value-taking flag {:?} must be in LAZY_FLAGS",
                c
            );
        }
    }

    #[test]
    fn test_no_duplicate_lazy_flags() {
        let mut sorted: Vec<char> = LAZY_FLAGS.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            LAZY_FLAGS.len(),
            "LAZY_FLAGS must have no duplicates"
        );
    }

    #[test]
    fn test_no_duplicate_value_taking_flags() {
        let mut sorted: Vec<char> = VALUE_TAKING_FLAGS.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            VALUE_TAKING_FLAGS.len(),
            "VALUE_TAKING_FLAGS must have no duplicates"
        );
    }

    #[test]
    fn test_aliases_dont_override_canonical() {
        // A lowercase alias should never map to a char that's already
        // a canonical lazy flag with a different meaning.
        // r is canonical for --reverse; if we aliased r→R, that would
        // be wrong because R is --recursive.
        for &(from, to) in LOWERCASE_ALIASES {
            // The 'from' char must NOT be in LAZY_FLAGS
            assert!(
                !LAZY_FLAGS.contains(&from),
                "alias source {:?} should not be a canonical flag",
                from
            );
            // The 'to' char must BE in LAZY_FLAGS
            assert!(
                LAZY_FLAGS.contains(&to),
                "alias target {:?} should be a canonical flag",
                to
            );
        }
    }

    #[test]
    fn test_known_subcommands_list() {
        assert!(KNOWN_SUBCOMMANDS.contains(&"banner"));
        assert!(KNOWN_SUBCOMMANDS.contains(&"env"));
        assert!(KNOWN_SUBCOMMANDS.contains(&"install"));
        assert!(KNOWN_SUBCOMMANDS.contains(&"config"));
        assert!(KNOWN_SUBCOMMANDS.contains(&"daemon"));
        assert!(KNOWN_SUBCOMMANDS.contains(&"help"));
        assert!(!KNOWN_SUBCOMMANDS.contains(&"stats")); // not a real subcommand
        assert!(!KNOWN_SUBCOMMANDS.contains(&"mv"));
    }

    // ===== Property-based tests (using proptest) =====
    // These tests verify invariants that should hold for ALL inputs,
    // not just specific cases. They run at least 1000 cases each.

    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn prop_resolve_lazy_flag_is_total(s in any::<char>()) {
            // resolve_lazy_flag_char must never panic and must return
            // either Some(c) where c is a valid lazy flag, or None
            let result = resolve_lazy_flag_char(s);
            if let Some(c) = result {
                assert!(
                    LAZY_FLAGS.contains(&c) || LOWERCASE_ALIASES.iter().any(|&(_, t)| t == c),
                    "resolve_lazy_flag_char({:?}) returned {:?} which is not a valid lazy flag", s, c
                );
            }
        }

        #[test]
        fn prop_expand_lazy_flags_valid_chains(s in "[a-zA-Z]{1,20}") {
            // For any string of 1-20 alphabetic chars, if every char
            // resolves to a lazy flag, the expansion must succeed and
            // contain one entry per char
            if let Some(flags) = expand_lazy_flags(&s) {
                assert_eq!(flags.len(), s.chars().count(), "expansion length mismatch for {:?}", s);
                // Every expanded char must be a canonical lazy flag
                for c in &flags {
                    assert!(LAZY_FLAGS.contains(c) || LOWERCASE_ALIASES.iter().any(|&(_, t)| t == *c),
                        "expanded flag {:?} is not canonical for input {:?}", c, s);
                }
            }
        }

        #[test]
        fn prop_expand_empty_string_returns_none(_unit in Just(())) {
            prop_assert_eq!(expand_lazy_flags(""), None);
        }

        #[test]
        fn prop_is_explicit_path_dot_prefix(s in "[a-zA-Z0-9_/]{1,20}") {
            // Any string starting with . is an explicit path
            let arg = format!(".{}", s);
            prop_assert!(is_explicit_path(&arg), "expected {:?} to be explicit path", arg);
        }

        #[test]
        fn prop_is_explicit_path_slash_prefix(s in "[a-zA-Z0-9_]{1,20}") {
            // Any string starting with / is an explicit path
            let arg = format!("/{}", s);
            prop_assert!(is_explicit_path(&arg), "expected {:?} to be explicit path", arg);
        }

        #[test]
        fn prop_is_explicit_path_tilde_prefix(s in "[a-zA-Z0-9_/]{1,20}") {
            // Any string starting with ~ is an explicit path
            let arg = format!("~{}", s);
            prop_assert!(is_explicit_path(&arg), "expected {:?} to be explicit path", arg);
        }

        #[test]
        fn prop_is_explicit_path_bare_alpha_rejected(s in "[a-zA-Z]{1,20}") {
            // Bare alphabetic strings are NOT explicit paths
            // (they're lazy flag chains)
            // We filter out strings that happen to be explicit (e.g., starting with .)
            if !s.starts_with('.') && !s.starts_with('/') && !s.starts_with('~') {
                prop_assert!(!is_explicit_path(&s), "expected {:?} to NOT be explicit path", s);
            }
        }

        #[test]
        fn prop_expand_and_resolve_consistent(s in "[a-zA-Z]{1,10}") {
            // For any string, expand_lazy_flags must agree with
            // resolve_lazy_flag_char for every char in the string
            let expanded = expand_lazy_flags(&s);
            let mut manual = Vec::new();
            let mut all_resolve = true;
            for c in s.chars() {
                if let Some(r) = resolve_lazy_flag_char(c) {
                    manual.push(r);
                } else {
                    all_resolve = false;
                    break;
                }
            }
            if all_resolve {
                prop_assert_eq!(expanded.clone(), Some(manual), "mismatch for {:?}", s);
            } else {
                prop_assert_eq!(expanded, None, "expected None for {:?}", s);
            }
        }

        #[test]
        fn prop_chain_length_equals_input_length(s in "[a-zA-Z]{1,15}") {
            // For any valid chain, the expanded length must equal input length
            if let Some(flags) = expand_lazy_flags(&s) {
                prop_assert_eq!(flags.len(), s.chars().count());
            }
        }

        #[test]
        fn prop_no_panic_on_random_input(s in ".*") {
            // The parser must never panic on any input
            let _ = expand_lazy_flags(&s);
            let _ = is_explicit_path(&s);
            let _ = resolve_lazy_flag_char(s.chars().next().unwrap_or('a'));
        }
    }
}
