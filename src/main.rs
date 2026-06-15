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

/// Expand a list of args for the banner subcommand: pass through
/// flags and numbers; expand aliases to their flag lists; drop
/// paths and unknown words. Used for non-banner routing where
/// paths are not allowed.
fn expand_args_strict(args: &[String]) -> Vec<String> {
    let mut new_args: Vec<String> = vec!["f".to_string(), "banner".to_string()];
    for a in args {
        if a.starts_with('-') || a.parse::<usize>().is_ok() {
            // Explicit flag or number — pass through unchanged.
            new_args.push(a.clone());
        } else if let Some(flags) = lookup_alias(a) {
            // Known alias — expand to its flags.
            for flag in flags {
                new_args.push(flag.to_string());
            }
        }
        // Paths and unknown words are dropped.
    }
    new_args
}

/// Expand a list of args for the banner subcommand in banner mode
/// (`-b`): pass through flags, paths, and numbers; expand aliases to
/// their flag lists; drop unknown words.
fn expand_args_for_banner(args: &[String]) -> Vec<String> {
    let mut new_args: Vec<String> = vec!["f".to_string(), "banner".to_string()];
    for a in args {
        if a == "-b" {
            // The banner switch itself is consumed.
            continue;
        }
        if a.starts_with('-') || is_path_like(a) || a.parse::<usize>().is_ok() {
            // Explicit flag, path, or number — pass through unchanged.
            new_args.push(a.clone());
        } else if let Some(flags) = lookup_alias(a) {
            // Known alias — expand to its flags.
            for flag in flags {
                new_args.push(flag.to_string());
            }
        }
        // Unknown bare word — DROP.
    }
    new_args
}

/// Returns true if the arg looks like an explicit path (starts with
/// `.`, `/`, or `~`). Used only in banner mode (`-b`) where paths
/// are allowed.
fn is_path_like(arg: &str) -> bool {
    if arg.is_empty() {
        return false;
    }
    let first = arg.chars().next().unwrap();
    first == '.' || first == '/' || first == '~'
}

/// Known subcommands that clap should parse. When the first non-flag
/// arg is one of these, we skip alias expansion and let clap handle
/// the invocation directly.
const KNOWN_SUBCOMMANDS: &[&str] = &["banner", "env", "install", "config", "daemon", "help"];

/// Returns true if the user-provided args contain an alias, number,
/// or flag (i.e., anything the banner subcommand should act on).
/// Paths and unknown bare words are not considered useful — they
/// are dropped per the "nothing happens" rule.
fn args_contain_something_useful(args: &[String]) -> bool {
    args.iter()
        .any(|a| a.starts_with('-') || lookup_alias(a).is_some() || a.parse::<usize>().is_ok())
}

/// Returns true if the invocation should exit 0 with no output —
/// the "nothing happens" rule. This is when the user typed at least
/// one arg but none of them are aliases, numbers, or flags. Paths
/// and unknown bare words both match. E.g. `f t`, `f foo`,
/// `f ./src`, `f /tmp`, `f Downloads` all match.
/// `f` (no args) returns false (default banner is shown).
/// `f tree`, `f 4`, `f -t`, `f -V` all return false.
fn should_exit_silently(args: &[String]) -> bool {
    !args.is_empty() && !args_contain_something_useful(args)
}

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

    // `-b` flag: switch to banner mode, which allows paths. This is
    // the way to see a banner for a specific path without using the
    // `banner` subcommand. `f -b` shows the default banner, `f -b
    // ./src` shows the banner for ./src, `f -b tree` shows a tree
    // banner, etc. Aliases still expand; unknown words are dropped.
    if args.iter().any(|a| a == "-b") {
        let expanded = expand_args_for_banner(&args);
        let cli = cli::Cli::parse_from(expanded);
        return cli.run();
    }

    // "Nothing happens" rule: if the user typed only args that match
    // no alias, number, or flag (e.g. `f t`, `f foo`, `f ./src`,
    // `f /tmp`, `f Downloads`), the command exits 0 with no output.
    // The user said: "we only take numbers, aliases, and flags, not
    // folders and files by name." `f` (no args) still shows the
    // default banner for cwd. Use `f -b <path>` for path-specific
    // banners.
    if should_exit_silently(&args) {
        return Ok(());
    }

    // If the args contain only flags (no aliases, no numbers), let
    // clap handle directly. This covers `f -V`, `f --help`, `f -e`,
    // `f -f txt`, etc. — invocations where the top-level Cli flags
    // are sufficient and no alias expansion is needed.
    let has_alias_or_number = args
        .iter()
        .any(|a| lookup_alias(a).is_some() || a.parse::<usize>().is_ok());
    if !has_alias_or_number && args.iter().any(|a| a.starts_with('-')) {
        let cli = cli::Cli::parse();
        return cli.run();
    }

    // Expand any built-in aliases in the args. Explicit flags and
    // numbers pass through. Anything else (paths, unknown words) is
    // dropped. The result is routed to the banner subcommand.
    let expanded = expand_args_strict(&args);
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

    // ===== expand_args_for_banner tests =====

    #[test]
    fn test_expand_aliases_empty_args() {
        let result = expand_args_for_banner(&[]);
        assert_eq!(result, vec!["f", "banner"]);
    }

    // ===== should_exit_silently tests =====

    #[test]
    fn test_should_exit_silently_no_args() {
        // `f` (no args) shows the default banner — not silent.
        assert!(!should_exit_silently(&[]));
    }

    #[test]
    fn test_should_exit_silently_unknown_word() {
        // `f t`, `f foo` — no flag/path/alias/number, exit silently.
        assert!(should_exit_silently(&["t".to_string()]));
        assert!(should_exit_silently(&["foo".to_string()]));
        assert!(should_exit_silently(&["Downloads".to_string()]));
    }

    #[test]
    fn test_should_exit_silently_with_flag() {
        // `f -e` — has flag, not silent.
        assert!(!should_exit_silently(&["-e".to_string()]));
        assert!(!should_exit_silently(&["-t".to_string()]));
        assert!(!should_exit_silently(&["--version".to_string()]));
    }

    #[test]
    fn test_should_exit_silently_with_path() {
        // `f ./src`, `f /tmp` — paths are dropped, exit silently.
        assert!(should_exit_silently(&["./src".to_string()]));
        assert!(should_exit_silently(&["/tmp".to_string()]));
        assert!(should_exit_silently(&["~/Downloads".to_string()]));
    }

    #[test]
    fn test_should_exit_silently_with_alias() {
        // `f tree`, `f hidden verbose` — has alias, not silent.
        assert!(!should_exit_silently(&["tree".to_string()]));
        assert!(!should_exit_silently(&[
            "hidden".to_string(),
            "verbose".to_string()
        ]));
    }

    #[test]
    fn test_should_exit_silently_with_number() {
        // `f 4` — has number, not silent.
        assert!(!should_exit_silently(&["4".to_string()]));
        assert!(!should_exit_silently(&["42".to_string()]));
    }

    #[test]
    fn test_should_exit_silently_mixed() {
        // `f t 4` — has number 4, not silent.
        assert!(!should_exit_silently(&["t".to_string(), "4".to_string()]));
        // `f t ./src` — paths are dropped, so the only useful arg is
        // the (unknown) `t`, which means silent.
        assert!(should_exit_silently(&[
            "t".to_string(),
            "./src".to_string()
        ]));
    }

    #[test]
    fn test_expand_aliases_single_alias() {
        let result = expand_args_for_banner(&["tree".to_string()]);
        assert_eq!(result, vec!["f", "banner", "-R", "-D"]);
    }

    #[test]
    fn test_expand_aliases_two_aliases() {
        let result = expand_args_for_banner(&["hidden".to_string(), "verbose".to_string()]);
        assert_eq!(result, vec!["f", "banner", "-a", "-v"]);
    }

    #[test]
    fn test_expand_aliases_three_aliases() {
        let result = expand_args_for_banner(&[
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
        let result = expand_args_for_banner(&["nonexistentword".to_string()]);
        assert_eq!(result, vec!["f", "banner"]);
    }

    #[test]
    fn test_expand_aliases_strict_drops_paths() {
        // In strict (non-banner) mode, paths are dropped. Use `f -b
        // <path>` for path-specific banners.
        let result = expand_args_strict(&["tree".to_string(), "./src".to_string()]);
        assert_eq!(result, vec!["f", "banner", "-R", "-D"]);
    }

    #[test]
    fn test_expand_aliases_keeps_paths() {
        // In banner mode (`-b`), paths are passed through.
        let result = expand_args_for_banner(&["tree".to_string(), "./src".to_string()]);
        assert_eq!(result, vec!["f", "banner", "-R", "-D", "./src"]);
    }

    #[test]
    fn test_expand_aliases_drops_unknown_words() {
        // Unknown bare words (e.g. `f`, `foo`, `Downloads`) are dropped.
        let result =
            expand_args_for_banner(&["f".to_string(), "foo".to_string(), "Downloads".to_string()]);
        assert_eq!(result, vec!["f", "banner"]);
    }

    #[test]
    fn test_expand_aliases_with_number() {
        let result = expand_args_for_banner(&["1".to_string()]);
        assert_eq!(result, vec!["f", "banner", "1"]);
    }

    #[test]
    fn test_expand_aliases_alias_with_value_flags() {
        // top has 4 flags including a value
        let result = expand_args_for_banner(&["top".to_string()]);
        assert_eq!(result, vec!["f", "banner", "-S", "-r", "-m", "20"]);
    }

    #[test]
    fn test_expand_aliases_mix_of_alias_and_path_drops_path_in_strict() {
        // In strict mode, paths are dropped even when mixed with aliases.
        let result = expand_args_strict(&[
            "hidden".to_string(),
            "/tmp".to_string(),
            "verbose".to_string(),
        ]);
        assert_eq!(result, vec!["f", "banner", "-a", "-v"]);
    }

    #[test]
    fn test_expand_aliases_mix_of_alias_and_path_keeps_path() {
        // In banner mode, paths are kept alongside aliases.
        let result = expand_args_for_banner(&[
            "hidden".to_string(),
            "/tmp".to_string(),
            "verbose".to_string(),
        ]);
        assert_eq!(result, vec!["f", "banner", "-a", "/tmp", "-v"]);
    }

    // ===== -b (banner switch) tests =====

    #[test]
    fn test_b_flag_alone() {
        // `f -b` — just the switch, no path, default banner for cwd.
        let result = expand_args_for_banner(&["-b".to_string()]);
        assert_eq!(result, vec!["f", "banner"]);
    }

    #[test]
    fn test_b_flag_with_path() {
        // `f -b ./src` — banner for ./src.
        let result = expand_args_for_banner(&["-b".to_string(), "./src".to_string()]);
        assert_eq!(result, vec!["f", "banner", "./src"]);
    }

    #[test]
    fn test_b_flag_with_absolute_path() {
        // `f -b /tmp` — banner for /tmp.
        let result = expand_args_for_banner(&["-b".to_string(), "/tmp".to_string()]);
        assert_eq!(result, vec!["f", "banner", "/tmp"]);
    }

    #[test]
    fn test_b_flag_with_tilde_path() {
        // `f -b ~/Downloads` — banner for ~/Downloads.
        let result = expand_args_for_banner(&["-b".to_string(), "~/Downloads".to_string()]);
        assert_eq!(result, vec!["f", "banner", "~/Downloads"]);
    }

    #[test]
    fn test_b_flag_with_alias() {
        // `f -b tree` — tree alias expands, path is implicit (cwd).
        let result = expand_args_for_banner(&["-b".to_string(), "tree".to_string()]);
        assert_eq!(result, vec!["f", "banner", "-R", "-D"]);
    }

    #[test]
    fn test_b_flag_with_path_and_alias() {
        // `f -b tree ./src` — alias expands, path is passed through.
        let result =
            expand_args_for_banner(&["-b".to_string(), "tree".to_string(), "./src".to_string()]);
        assert_eq!(result, vec!["f", "banner", "-R", "-D", "./src"]);
    }

    #[test]
    fn test_b_flag_drops_unknown_words() {
        // `f -b foo` — unknown word dropped, only the switch remains.
        let result = expand_args_for_banner(&["-b".to_string(), "foo".to_string()]);
        assert_eq!(result, vec!["f", "banner"]);
    }

    #[test]
    fn test_b_flag_with_explicit_flag() {
        // `f -b -t` — explicit flag is preserved.
        let result = expand_args_for_banner(&["-b".to_string(), "-t".to_string()]);
        assert_eq!(result, vec!["f", "banner", "-t"]);
    }

    #[test]
    fn test_b_flag_with_number() {
        // `f -b 5` — navigate to item 5.
        let result = expand_args_for_banner(&["-b".to_string(), "5".to_string()]);
        assert_eq!(result, vec!["f", "banner", "5"]);
    }

    // ===== is_path_like tests =====

    #[test]
    fn test_is_path_like() {
        assert!(is_path_like("."));
        assert!(is_path_like(".."));
        assert!(is_path_like("./src"));
        assert!(is_path_like("/"));
        assert!(is_path_like("/tmp"));
        assert!(is_path_like("~"));
        assert!(is_path_like("~/Downloads"));
        assert!(!is_path_like("tree"));
        assert!(!is_path_like("foo"));
        assert!(!is_path_like(""));
    }
}
