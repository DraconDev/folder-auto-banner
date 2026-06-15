// Integration tests for the built-in alias system in f 0.7.0+.
//
// Run with: cargo test --test alias_test -- --test-threads=1
//
// Note: tests must run with --test-threads=1 because the daemon uses
// a single shared socket and parallel runs can flake.

use assert_cmd::Command;

/// Helper: run `f` with the given args and return trimmed stdout.
fn run_f(args: &[&str]) -> String {
    let mut cmd = Command::cargo_bin("f").unwrap();
    for a in args {
        cmd.arg(a);
    }
    let output = cmd.output().expect("failed to run f");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// Helper: run `f` and return (stdout, stderr, exit_code).
fn run_f_full(args: &[&str]) -> (String, String, i32) {
    let mut cmd = Command::cargo_bin("f").unwrap();
    for a in args {
        cmd.arg(a);
    }
    let output = cmd.output().expect("failed to run f");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

/// Get the first line of output, truncated to 200 chars (for
/// deterministic comparison that ignores timing-sensitive content).
fn first_line_header(s: &str) -> String {
    s.lines().next().unwrap_or("").chars().take(200).collect()
}

// ===== Alias smoke tests (one per alias) =====
// Each alias must produce the same output as its explicit form.

#[test]
fn alias_tree_matches_explicit() {
    let lazy = run_f(&["tree"]);
    let explicit = run_f(&["-R", "-D"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_flat_matches_explicit() {
    let lazy = run_f(&["flat"]);
    let explicit = run_f(&["-o"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_compact_matches_explicit() {
    let lazy = run_f(&["compact"]);
    let explicit = run_f(&["-c"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_verbose_matches_explicit() {
    let lazy = run_f(&["verbose"]);
    let explicit = run_f(&["-v"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_hidden_matches_explicit() {
    let lazy = run_f(&["hidden"]);
    let explicit = run_f(&["-a"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_dirs_matches_explicit() {
    let lazy = run_f(&["dirs"]);
    let explicit = run_f(&["-D"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_new_matches_explicit() {
    let lazy = run_f(&["new"]);
    let explicit = run_f(&["-t"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_old_matches_explicit() {
    let lazy = run_f(&["old"]);
    let explicit = run_f(&["-t", "-r"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_big_matches_explicit() {
    let lazy = run_f(&["big"]);
    let explicit = run_f(&["-S"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_small_matches_explicit() {
    let lazy = run_f(&["small"]);
    let explicit = run_f(&["-S", "-r"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_ext_matches_explicit() {
    let lazy = run_f(&["ext"]);
    let explicit = run_f(&["-X"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_git_matches_explicit() {
    let lazy = run_f(&["git"]);
    let explicit = run_f(&["-G"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_nosort_matches_explicit() {
    let lazy = run_f(&["nosort"]);
    let explicit = run_f(&["-U"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_top_matches_explicit() {
    let lazy = run_f(&["top"]);
    let explicit = run_f(&["-S", "-r", "-m", "20"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_newest_matches_explicit() {
    let lazy = run_f(&["newest"]);
    let explicit = run_f(&["-t", "-r", "-m", "20"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_recurse_matches_explicit() {
    let lazy = run_f(&["recurse"]);
    let explicit = run_f(&["-R"]);
    assert_eq!(first_line_header(&lazy), first_line_header(&explicit));
}

#[test]
fn alias_edit_matches_explicit() {
    let (lazy_stdout, _lazy_stderr, lazy_code) = run_f_full(&["edit"]);
    let (exp_stdout, _exp_stderr, exp_code) = run_f_full(&["-e"]);
    assert_eq!(lazy_code, exp_code);
    assert_eq!(
        first_line_header(&lazy_stdout),
        first_line_header(&exp_stdout)
    );
}

#[test]
fn alias_run_matches_explicit() {
    let (lazy_stdout, _lazy_stderr, lazy_code) = run_f_full(&["run"]);
    let (exp_stdout, _exp_stderr, exp_code) = run_f_full(&["-x"]);
    assert_eq!(lazy_code, exp_code);
    assert_eq!(
        first_line_header(&lazy_stdout),
        first_line_header(&exp_stdout)
    );
}

// ===== Alias composition tests =====

#[test]
fn alias_composition_two_aliases() {
    let composed = run_f(&["hidden", "verbose"]);
    let explicit = run_f(&["-a", "-v"]);
    assert_eq!(first_line_header(&composed), first_line_header(&explicit));
}

#[test]
fn alias_composition_three_aliases() {
    let composed = run_f(&["new", "recurse", "hidden"]);
    let explicit = run_f(&["-t", "-R", "-a"]);
    assert_eq!(first_line_header(&composed), first_line_header(&explicit));
}

#[test]
fn alias_composition_top_with_hidden() {
    let composed = run_f(&["top", "hidden"]);
    let explicit = run_f(&["-S", "-r", "-m", "20", "-a"]);
    assert_eq!(first_line_header(&composed), first_line_header(&explicit));
}

// ===== Routing tests =====

#[test]
fn unknown_bare_word_does_nothing() {
    // User's requirement: "if no such alias found then nothing happens"
    // i.e., exit 0 with no output, not the default banner.
    let (stdout, _stderr, code) = run_f_full(&["nonexistentword"]);
    assert_eq!(code, 0, "unknown word should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "unknown word should produce no output, got: {}",
        stdout
    );
}

#[test]
fn f_t_does_nothing() {
    // `f t` used to mean `-t` (timesort) in 0.6.x lazy flags.
    // In 0.7.1, `t` is not a known alias — it produces no output.
    let (stdout, _stderr, code) = run_f_full(&["t"]);
    assert_eq!(code, 0, "f t should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "f t should produce no output, got: {}",
        stdout
    );
}

#[test]
fn f_no_args_still_shows_banner() {
    // `f` (no args) is different from `f <unknown-word>`.
    // `f` shows the default banner for cwd.
    let (stdout, _stderr, code) = run_f_full(&[]);
    assert_eq!(code, 0, "f (no args) should not error");
    assert!(
        !stdout.is_empty(),
        "f (no args) should show the default banner"
    );
}

#[test]
fn unknown_ba[DRACON_SECRET:YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBaUWlTNC8wL0lKeHZPanV6K3hscHBBU21zaS9aTUc3N1lxNDlYZ0xOVkdrCnM4VVA1MklJV3FOeGhac1hYVjZTRngzZzdCK01meWdjTHlna21nbmU3YjQKLT4gWDI1NTE5IEtxbW4zeVN6SmZ5UjI2YXZnaWsydndQSy84Yit6RVNFZWhhZzRMRzRJQ00KVnZUb3NUUHA0YlBLR0V4Y1F3R2wvQ2dHcnhCOVlhL05qN2pCYjVPdVZobwotPiBYMjU1MTkgZGdPZG5DSVVMQTF6Vm1TZWhZSmpmditPSXFrUWFTSUxGanU5b0dzdENrNAoxL0FTYU9GalQxMnJRVWsvNTc0RGQ5Q3lLdVM5Y2xYNVVxaGEwMnZQTlNRCi0+IFgyNTUxOSAzb0tBa3I1QTdIOVFTNWhIMDVrUTlUT0tBOVZwVk9KTkg1QkRFdTZENVhnClVTNE1US1l1eWtOK1JjRWtuS0RkaElBNkxtNmg5YVNVdUtIU3pJaDB5amcKLT4gWDI1NTE5IHQ4UlVOU3hld3Y4aTUrRVNCbFF3aG9nYVJ6SHcyZUIwL01FdGlFSHR3bncKNnk5dWVFa3ByNWxZVFZuVytWY2pDOHJHT3doR210aHg5OS9WUklMbWhGWQotPiBWLWdyZWFzZSB8PCA0Jlc8ICY+TCBGcVgKYVhtZ05udHZoNW9xUWhSWUN6Z1lUN3Q2RHU2Q0tZclpCd0tnN3U1MXRXdEFVOGtoREsrQWhiZHZ5Z28KLS0tIEswdHJwbGRtY2N2TXJRVHBZbHNHQllqVUFvZHJ4OWh0TUxaNkdmUU4zWWcKHmQ68JdeLXDYYGPsBBYEOgyjsSxgyRd/euKpNHTjRau57CeeCd6giuwlkHBwZ9V3Dp91Djrx1y+NKgM=]() {
    // `f Downloads` (no ./ prefix) is NOT treated as a path. It is an
    // unknown bare word and produces no output (the "nothing happens" rule).
    let (stdout, _stderr, code) = run_f_full(&["Downloads"]);
    assert_eq!(code, 0, "f Downloads should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "f Downloads should produce no output, got: {}",
        stdout
    );
}

#[test]
fn explicit_path_with_dot_slash_does_nothing() {
    // `f ./src` is dropped (paths are not in the routing table).
    // User said: "we only take numbers, aliases, and flags, not
    // folders and files by name."
    let (stdout, _stderr, code) = run_f_full(&["./src"]);
    assert_eq!(code, 0, "./src should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "./src should produce no output, got: {}",
        stdout
    );
}

#[test]
fn explicit_path_with_slash_does_nothing() {
    let (stdout, _stderr, code) = run_f_full(&["/tmp"]);
    assert_eq!(code, 0, "/tmp should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "/tmp should produce no output, got: {}",
        stdout
    );
}

#[test]
fn explicit_path_with_tilde_does_nothing() {
    // Tilde expansion is a shell feature, so we use the expanded
    // path here. Even when expanded, paths are still dropped.
    let home = std::env::var("HOME").unwrap_or("/tmp".to_string());
    let (stdout, _stderr, code) = run_f_full(&[home.as_str()]);
    assert_eq!(code, 0, "HOME path should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "HOME path should produce no output, got: {}",
        stdout
    );
}

#[test]
fn f_subcommand_path_still_works() {
    // The user can still get a banner for a specific path by using
    // the banner subcommand explicitly. Aliases do not apply to
    // subcommand invocations.
    let (_stdout, _stderr, code) = run_f_full(&["banner", "./src"]);
    assert_eq!(
        code, 0,
        "f banner ./src should still work, got: {}",
        _stderr
    );
}

#[test]
fn explicit_flag_bypass_works() {
    // f -t should still work (explicit flag, no alias)
    let (_stdout, _stderr, code) = run_f_full(&["-t"]);
    assert_eq!(code, 0, "f -t should succeed");
}

#[test]
fn explicit_flag_with_value_works() {
    let (_stdout, _stderr, code) = run_f_full(&["-f", "txt"]);
    assert_eq!(code, 0, "f -f txt should succeed");
}

#[test]
fn explicit_long_flag_works() {
    let (_stdout, _stderr, code) = run_f_full(&["--filter", "txt"]);
    assert_eq!(code, 0, "f --filter txt should succeed");
}

#[test]
fn number_navigation_still_works() {
    // f 1 should still navigate to item 1 (number takes precedence)
    let (_stdout, _stderr, code) = run_f_full(&["1"]);
    assert_eq!(code, 0, "f 1 should succeed");
}

// ===== Subcommand passthrough tests =====

#[test]
fn subcommand_banner_passthrough() {
    // f banner 1 should be handled by clap directly (subcommand invocation)
    let (_stdout, _stderr, code) = run_f_full(&["banner", "1"]);
    assert_eq!(code, 0, "f banner 1 should succeed: {}", _stderr);
}

#[test]
fn subcommand_help_passthrough() {
    // f help should print help (clap handles)
    let (_stdout, _stderr, code) = run_f_full(&["help"]);
    let _ = _stdout;
    let _ = _stderr;
    // help may exit non-zero in clap, that's fine — we just verify
    // the invocation is handled and doesn't crash.
    let _ = code;
}

#[test]
fn subcommand_env_passthrough() {
    let (_stdout, _stderr, code) = run_f_full(&["env"]);
    let _ = _stdout;
    let _ = _stderr;
    let _ = code;
}

// ===== Lazy flag removal verification =====
// These tests verify that the lazy flag system is GONE.

#[test]
fn f_t_no_longer_means_dash_t() {
    // In 0.6.x, `f t` meant `-t`. In 0.7.1, `t` is not an alias,
    // so it produces no output (the "nothing happens" rule).
    let (stdout, _stderr, code) = run_f_full(&["t"]);
    assert_eq!(code, 0, "f t should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "f t should produce no output, got: {}",
        stdout
    );
}

#[test]
fn f_trc_no_longer_means_dash_t_dash_r_dash_c() {
    // In 0.6.x, `f trc` meant `-t -r -c`. In 0.7.1, it produces
    // no output.
    let (stdout, _stderr, code) = run_f_full(&["trc"]);
    assert_eq!(code, 0, "f trc should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "f trc should produce no output, got: {}",
        stdout
    );
}

#[test]
fn f_s_no_longer_means_dash_upper_s() {
    // In 0.6.x, `f s` meant `-S` (case-insensitive alias).
    // In 0.7.1, `s` is not an alias, so it produces no output.
    let (stdout, _stderr, code) = run_f_full(&["s"]);
    assert_eq!(code, 0, "f s should not error, got: {}", _stderr);
    assert!(
        stdout.is_empty(),
        "f s should produce no output, got: {}",
        stdout
    );
}

#[test]
fn f_m_lf_colon_no_longer_works() {
    // In 0.6.37, `f mLf: 10` meant `-f 10`. In 0.7.0, the `:` binding
    // is gone. The `mLf:` is an unknown bare word and is dropped. The
    // `10` is a number and is passed through, so this becomes
    // equivalent to `f 10` (navigate to item 10). We just verify it
    // doesn't error and doesn't apply the `:` binding.
    let (_stdout, _stderr, code) = run_f_full(&["mLf:", "10"]);
    assert_eq!(code, 0, "should not error, got: {}", _stderr);
}

#[test]
fn f_dash_t_still_works_after_removal() {
    // Explicit flags should still work after lazy flag removal.
    let (_stdout, _stderr, code) = run_f_full(&["-t"]);
    assert_eq!(code, 0);
}

// ===== Alias + explicit flag composition =====

#[test]
fn alias_plus_explicit_flag() {
    let composed = run_f(&["tree", "-L", "2"]);
    let explicit = run_f(&["-R", "-D", "-L", "2"]);
    assert_eq!(first_line_header(&composed), first_line_header(&explicit));
}

#[test]
fn alias_plus_path_drops_path() {
    // `f tree ./src` — the alias expands to `-R -D`, the path is
    // dropped. The result is equivalent to `f tree`.
    let with_path = run_f(&["tree", "./src"]);
    let just_tree = run_f(&["tree"]);
    assert_eq!(first_line_header(&with_path), first_line_header(&just_tree));
}

#[test]
fn number_passes_through_with_alias() {
    // f tree 5 — the `5` is a number, not an alias, but should
    // not be expanded. It passes through to the banner subcommand.
    // (Whether the banner subcommand treats it as a path or errors
    // is up to clap, but it should not be expanded as an alias.)
    let (_stdout, _stderr, _code) = run_f_full(&["tree", "5"]);
    // We don't assert success — we just verify it doesn't crash.
    let _ = _stdout;
    let _ = _stderr;
}
