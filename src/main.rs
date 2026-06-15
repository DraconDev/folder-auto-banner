use anyhow::Result;
use clap::Parser;
use folder_auto_banner::cli;
use std::env;

/// Built-in aliases. Each alias expands to a list of clap flags.
///
/// Multiple aliases compose: `f hidden verbose` → `-a -v`.
/// Aliases compose with explicit flags: `f tree -L 2` → `-R -D -L 2`.
/// Aliases compose with paths: `f top ./src` → `-S -r -m 20` for `./src`.
///
/// User intent: bare words (not numbers, not paths) are alias lookups.
/// Unknown bare words show the default cwd banner (no error).
const BUILTIN_ALIASES: &[(&str, &[&str])] = &[
    // Display modes
    ("tree", &["-R", "-D"]), // Recursive, only dirs (like `tree`)
    ("flat", &["-o"]),       // One file per line
    ("compact", &["-c"]),    // Compact output
    ("verbose", &["-v"]),    // Verbose output
    ("hidden", &["-a"]),     // Show hidden files
    ("dirs", &["-D"]),       // Only directories
    // Sort modes
    ("new", &["-t"]),         // Sort by time, newest first
    ("old", &["-t", "-r"]),   // Sort by time, oldest first
    ("big", &["-S"]),         // Sort by size, largest first
    ("small", &["-S", "-r"]), // Sort by size, smallest first
    ("ext", &["-X"]),         // Sort by extension
    ("git", &["-G"]),         // Sort by git status
    ("nosort", &["-U"]),      // No sort
    // Limits
    ("top", &["-S", "-r", "-m", "20"]),    // Top 20 largest files
    ("newest", &["-t", "-r", "-m", "20"]), // 20 newest files
    // Recursion
    ("recurse", &["-R"]), // Recurse into subdirectories
    // Actions
    ("edit", &["-e"]), // Force open in editor
    ("run", &["-x"]),  // Force run file
];

/// Look up an alias by name. Returns the flag list if found, None otherwise.
fn lookup_alias(name: &str) -> Option<&'static [&'static str]> {
    BUILTIN_ALIASES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, flags)| *flags)
}

/// Returns true if the arg looks like an explicit path (starts with
/// `.`, `/`, or `~`). Used to disambiguate paths from aliases.
fn is_explicit_path(arg: &str) -> bool {
    if arg.is_empty() {
        return false;
    }
    let first = arg.chars().next().unwrap();
    first == '.' || first == '/' || first == '~'
}

/// Expand a list of args: for each non-flag, non-path arg that matches
/// a built-in alias, substitute the alias's flag list. Unknown bare
/// words are DROPPED (the user said "if no alias found, nothing happens").
/// Returns the expanded arg list ready for clap to parse.
///
/// The expansion produces: ["f", "banner", <expanded-flags-and-other-args>].
fn expand_aliases_in_args(args: &[String]) -> Vec<String> {
    let mut new_args: Vec<String> = vec!["f".to_string(), "banner".to_string()];
    for a in args {
        if a.starts_with('-') || is_explicit_path(a) || a.parse::<usize>().is_ok() {
            // Explicit flag, path, or number — pass through unchanged.
            new_args.push(a.clone());
        } else if let Some(flags) = lookup_alias(a) {
            // Known alias — expand to its flags.
            for flag in flags {
                new_args.push(flag.to_string());
            }
        }
        // Unknown bare word — DROP (the "nothing happens" rule).
    }
    new_args
}

/// Known subcommands that clap should parse. When the first non-flag
/// arg is one of these, we skip alias expansion and let clap handle
/// the invocation directly.
const KNOWN_SUBCOMMANDS: &[&str] = &["banner", "env", "install", "config", "daemon", "help"];

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    // If the first arg is a known subcommand, let clap handle it
    // directly. Aliases do not apply to subcommand invocations.
    if let Some(first) = args.first() {
        if KNOWN_SUBCOMMANDS.contains(&first.as_str()) {
            let cli = cli::Cli::parse();
            return cli.run();
        }
    }

    // If the args contain only flags (no paths, no aliases, no numbers),
    // let clap handle directly. This covers `f -V`, `f --help`, `f -e`,
    // `f -f txt`, etc. — invocations where the top-level Cli flags are
    // sufficient and no path/alias expansion is needed.
    let has_explicit_flag = args.iter().any(|a| a.starts_with('-'));
    let has_path_or_alias_or_number = args
        .iter()
        .any(|a| is_explicit_path(a) || lookup_alias(a).is_some() || a.parse::<usize>().is_ok());
    if has_explicit_flag && !has_path_or_alias_or_number {
        let cli = cli::Cli::parse();
        return cli.run();
    }

    // Expand any built-in aliases in the args. Unknown bare words are
    // dropped (the "nothing happens" rule). Explicit flags, paths, and
    // numbers pass through. The result is routed to the banner subcommand.
    let expanded = expand_aliases_in_args(&args);
    let cli = cli::Cli::parse_from(expanded);
    cli.run()
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;

    // ===== BUILTIN_ALIASES integrity tests =====

    #[test]
    fn test_builtin_aliases_count_is_18() {
        assert_eq!(
            BUILTIN_ALIASES.len(),
            18,
            "BUILTIN_ALIASES should have 18 entries"
        );
    }

    #[test]
    fn test_builtin_aliases_no_duplicate_names() {
        let mut names: Vec<&str> = BUILTIN_ALIASES.iter().map(|(n, _)| *n).collect();
        names.sort();
        names.dedup();
        assert_eq!(
            names.len(),
            BUILTIN_ALIASES.len(),
            "BUILTIN_ALIASES must have no duplicate names"
        );
    }

    #[test]
    fn test_builtin_aliases_no_empty_flag_list() {
        for (name, flags) in BUILTIN_ALIASES {
            assert!(!flags.is_empty(), "alias {:?} has empty flag list", name);
        }
    }

    #[test]
    fn test_builtin_aliases_value_flag_pairs_valid() {
        // Each value-taking flag (like `-m`) must be followed by a value
        // (a token that doesn't start with `-`). Each boolean flag
        // (like `-t`) is a standalone token.
        const VALUE_TAKING: &[&str] = &["-m", "-f", "-L"];
        for (name, flags) in BUILTIN_ALIASES {
            let mut i = 0;
            while i < flags.len() {
                let flag = flags[i];
                if VALUE_TAKING.contains(&flag) {
                    // Next token must be a value (not start with `-`)
                    assert!(
                        i + 1 < flags.len(),
                        "alias {:?} has value-taking flag {:?} without a value",
                        name,
                        flag
                    );
                    let value = flags[i + 1];
                    assert!(
                        !value.starts_with('-'),
                        "alias {:?} has value-taking flag {:?} followed by another flag {:?}",
                        name,
                        flag,
                        value
                    );
                    i += 2;
                } else {
                    assert!(
                        flag.starts_with('-'),
                        "alias {:?} has flag {:?} that doesn't start with '-'",
                        name,
                        flag
                    );
                    i += 1;
                }
            }
        }
    }

    // ===== lookup_alias tests (one per alias) =====

    #[test]
    fn test_lookup_tree() {
        assert_eq!(lookup_alias("tree"), Some(&["-R", "-D"][..]));
    }

    #[test]
    fn test_lookup_flat() {
        assert_eq!(lookup_alias("flat"), Some(&["-o"][..]));
    }

    #[test]
    fn test_lookup_compact() {
        assert_eq!(lookup_alias("compact"), Some(&["-c"][..]));
    }

    #[test]
    fn test_lookup_verbose() {
        assert_eq!(lookup_alias("verbose"), Some(&["-v"][..]));
    }

    #[test]
    fn test_lookup_hidden() {
        assert_eq!(lookup_alias("hidden"), Some(&["-a"][..]));
    }

    #[test]
    fn test_lookup_dirs() {
        assert_eq!(lookup_alias("dirs"), Some(&["-D"][..]));
    }

    #[test]
    fn test_lookup_new() {
        assert_eq!(lookup_alias("new"), Some(&["-t"][..]));
    }

    #[test]
    fn test_lookup_old() {
        assert_eq!(lookup_alias("old"), Some(&["-t", "-r"][..]));
    }

    #[test]
    fn test_lookup_big() {
        assert_eq!(lookup_alias("big"), Some(&["-S"][..]));
    }

    #[test]
    fn test_lookup_small() {
        assert_eq!(lookup_alias("small"), Some(&["-S", "-r"][..]));
    }

    #[test]
    fn test_lookup_ext() {
        assert_eq!(lookup_alias("ext"), Some(&["-X"][..]));
    }

    #[test]
    fn test_lookup_git() {
        assert_eq!(lookup_alias("git"), Some(&["-G"][..]));
    }

    #[test]
    fn test_lookup_nosort() {
        assert_eq!(lookup_alias("nosort"), Some(&["-U"][..]));
    }

    #[test]
    fn test_lookup_top() {
        assert_eq!(lookup_alias("top"), Some(&["-S", "-r", "-m", "20"][..]));
    }

    #[test]
    fn test_lookup_newest() {
        assert_eq!(lookup_alias("newest"), Some(&["-t", "-r", "-m", "20"][..]));
    }

    #[test]
    fn test_lookup_recurse() {
        assert_eq!(lookup_alias("recurse"), Some(&["-R"][..]));
    }

    #[test]
    fn test_lookup_edit() {
        assert_eq!(lookup_alias("edit"), Some(&["-e"][..]));
    }

    #[test]
    fn test_lookup_run() {
        assert_eq!(lookup_alias("run"), Some(&["-x"][..]));
    }

    #[test]
    fn test_lookup_unknown_returns_none() {
        assert_eq!(lookup_alias(""), None);
        assert_eq!(lookup_alias("foo"), None);
        assert_eq!(lookup_alias("Tree"), None); // case-sensitive
        assert_eq!(lookup_alias("TREE"), None); // case-sensitive
        assert_eq!(lookup_alias("t"), None); // single char not an alias
        assert_eq!(lookup_alias("trc"), None); // chains not aliases
    }

    // ===== is_explicit_path tests =====

    #[test]
    fn test_is_explicit_path_dot_prefix() {
        assert!(is_explicit_path("."));
        assert!(is_explicit_path(".."));
        assert!(is_explicit_path("./Downloads"));
        assert!(is_explicit_path("../sibling"));
        assert!(is_explicit_path(".hidden"));
    }

    #[test]
    fn test_is_explicit_path_slash_prefix() {
        assert!(is_explicit_path("/"));
        assert!(is_explicit_path("/tmp"));
        assert!(is_explicit_path("/home/user"));
    }

    #[test]
    fn test_is_explicit_path_tilde_prefix() {
        assert!(is_explicit_path("~"));
        assert!(is_explicit_path("~/"));
        assert!(is_explicit_path("~/Downloads"));
    }

    #[test]
    fn test_is_explicit_path_bare_words_false() {
        assert!(!is_explicit_path("Downloads"));
        assert!(!is_explicit_path("tree"));
        assert!(!is_explicit_path("hidden"));
        assert!(!is_explicit_path(""));
    }

    // ===== expand_aliases_in_args tests =====

    #[test]
    fn test_expand_aliases_empty_args() {
        let result = expand_aliases_in_args(&[]);
        assert_eq!(result, vec!["f", "banner"]);
    }

    #[test]
    fn test_expand_aliases_single_alias() {
        let result = expand_aliases_in_args(&["tree".to_string()]);
        assert_eq!(result, vec!["f", "banner", "-R", "-D"]);
    }

    #[test]
    fn test_expand_aliases_two_aliases() {
        let result = expand_aliases_in_args(&["hidden".to_string(), "verbose".to_string()]);
        assert_eq!(result, vec!["f", "banner", "-a", "-v"]);
    }

    #[test]
    fn test_expand_aliases_three_aliases() {
        let result = expand_aliases_in_args(&[
            "new".to_string(),
            "recurse".to_string(),
            "hidden".to_string(),
        ]);
        assert_eq!(result, vec!["f", "banner", "-t", "-R", "-a"]);
    }

    #[test]
    fn test_expand_aliases_unknown_word_dropped() {
        // User's rule: "if no alias found, nothing happens" — unknown
        // bare words are dropped from the args, leaving the default
        // banner for cwd.
        let result = expand_aliases_in_args(&["nonexistentword".to_string()]);
        assert_eq!(result, vec!["f", "banner"]);
    }

    #[test]
    fn test_expand_aliases_with_explicit_path() {
        let result = expand_aliases_in_args(&["tree".to_string(), "./src".to_string()]);
        assert_eq!(result, vec!["f", "banner", "-R", "-D", "./src"]);
    }

    #[test]
    fn test_expand_aliases_with_number() {
        let result = expand_aliases_in_args(&["1".to_string()]);
        assert_eq!(result, vec!["f", "banner", "1"]);
    }

    #[test]
    fn test_expand_aliases_alias_with_value_flags() {
        // top has 4 flags including a value
        let result = expand_aliases_in_args(&["top".to_string()]);
        assert_eq!(result, vec!["f", "banner", "-S", "-r", "-m", "20"]);
    }

    #[test]
    fn test_expand_aliases_mix_of_alias_and_path() {
        let result = expand_aliases_in_args(&[
            "hidden".to_string(),
            "/tmp".to_string(),
            "verbose".to_string(),
        ]);
        assert_eq!(result, vec!["f", "banner", "-a", "/tmp", "-v"]);
    }
}
