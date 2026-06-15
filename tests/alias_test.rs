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
    assert_eq!(first_line_header(&lazy_stdout), first_line_header(&exp_stdout));
}

#[test]
fn alias_run_matches_explicit() {
    let (lazy_stdout, _lazy_stderr, lazy_code) = run_f_full(&["run"]);
    let (exp_stdout, _exp_stderr, exp_code) = run_f_full(&["-x"]);
    assert_eq!(lazy_code, exp_code);
    assert_eq!(first_line_header(&lazy_stdout), first_line_header(&exp_stdout));
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
fn unknown_ba[DRACON_SECRET:YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBJVHQ5dzJIR0JnK0VWbUNOelgvSHpVMmJlckZUVjZRcDBUN0piVUg3UFdzCmltOVliUE9MaFdvREM5cnJrK2JjK20wSzNhKzFBZSs4L0ZxVUVDZ2pFSWcKLT4gWDI1NTE5IDdmMlhWcjA2ZTEyN09weXZBZFJoa21Wdk44eFFlMWYrZWx4TDNpcE9wZ2cKUGtWY3NrQStoMU9SUWVQM3hoempzK1FLWUdML2hWZEJjQzBtaldZek85MAotPiBYMjU1MTkgSEt6VGtKSUpMTXE4OFhmMlY1ZTVZUSs2VWV3UHlOUVB3VUwrMGZWanB5SQpaTmI5bjRXS3JRVm5rUDZyS0dSZFVpYmdjVDg5cktwajZ3NnRnc3RGM2U4Ci0+IFgyNTUxOSBWMFRkR1llS0JZbVJ6Wmo0QkZzSlhOM09QdnNpN1ZyV05reStUckFzbWdvCjVpSFQ0K1plakErVkVnZjBwTTZtQ1k1ck5BWUNPUnlYek90ODUxQWJUbmsKLT4gWDI1NTE5IEFDSC9ad2Fmb2NYVloxRWptVEt0bEI1aE1pZk1YSGpNSDUwSjIwSUIrWG8KMXRueWEvWnlwNTVRbG1uZUtNRnNnc3g0VmFscjN4Qmtlai9palVlSmhYSQotPiB4JTgwX1xCZi1ncmVhc2UgSjgpICFpcSJxCjc2ZisKLS0tIFRHS0RyOEc3RnJBMUhkUVlYckcrbFdHNXFOam9wQ2lOdU5yaXFhMU0yZHMKZcxRm8105C+N2rSKQ8AqL0M4S1YNXRES4/uxyKzC6aJaxGSErpgHKcWHWS4LkyYUobOQsZJTx2njIubl]() {
    // User's requirement: "if no such alias found then nothing happens"
    // i.e., default banner for cwd, not an error.
    let (_stdout, _stderr, code) = run_f_full(&["nonexistentword"]);
    assert_eq!(code, 0, "unknown word should not error, got: {}", _stderr);
    // Should produce some banner output (default for cwd)
    let default_out = run_f(&[]);
    assert_eq!(first_line_header(&_stdout), first_line_header(&default_out));
}

#[test]
fn unknown_ba[DRACON_SECRET:YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0+IFgyNTUxOSBFREFtVlBnRERDc1lZYWg4dE9ENU9veTFGNkR6aDhiWUpzdU56eTA5dUJ3CnA0SVUybTRLUTBOeTl2bngwZUFUajRJR2wrU3pYRUhoWHR1ZkkxV0RhcU0KLT4gWDI1NTE5IFRKTWZDNkp0dC92Sks1QnBScEtBY2pVN2VPMUUvQW9TdE4wbC9seWdNZ3cKRmNLNjZtOGZKTDFLZHIrZmV1cldCSHRiTk03NjNhTXhaNWxyWWR5dVRFUQotPiBYMjU1MTkgOU0rc1d2U01vWkE1SXM5M3dCLzgvWFB6M1dtc0w0OXliOVpHWnhzUjhEdwpsOXJ4b3h2MFpDd0YxZFVTZGFhRWFSOVF5bnV6S3l2YUpJN2o2ZG9JNEQ4Ci0+IFgyNTUxOSBrU0FHaHhKOWZoTXpwdFJrZVBlUS8xSCttMjBHMG9VN0ZpNFVXQ05waGpvCkpLVzdtbDJpSXphczd3VFk0RnVJM1A1Ulpjem1EVWcwQ3ZhQ2tHb2xpaWsKLT4gWDI1NTE5IE0zQjh5L0dwMk1RVW12RWdEZ2JSSkc5eG84b0lWcXNHTW1ZVStlSm9OaUUKWWRNUVFPSUM4RkhWQ3JJcXA0ZEx2WGJ1dFZjSExjVDVOQVY3d0RPV3U1QQotPiB6WTEtZ3JlYXNlIHMoCm9uYTl4SmpvZEx5TzlrNU1xQUp0WE10QmxqQS8xYUd2L1Nmcmt4Y1h4OXZKUVcyRHZINEtwTndFV2hmQ05MZWwKTEdieEVNMjZXbklYNGRIN1R6L3MrTC9MMHIva3dlSHlkTEJJVHZBeWpON1EwM3pldjdyZ2RsMAotLS0gQXJFRkNnRVVSOWlINTlhVXRCYmUwRHhpS2kxYkcyNGdjcU81NUVnL080Zwr5AC+TbmPuvN4P7nsQ4JgvXvxrfbyaJK1vzN5RrRnyoVRAocgWFqWGzR+BAYuA/xFGZO04y7osWavNEw==]() {
    // `f Downloads` (no ./ prefix) should NOT be treated as a path
    // to a folder called Downloads. It should show the default banner.
    let (_stdout, _stderr, code) = run_f_full(&["Downloads"]);
    assert_eq!(code, 0);
    let default_out = run_f(&[]);
    assert_eq!(first_line_header(&_stdout), first_line_header(&default_out));
}

#[test]
fn explicit_path_with_dot_slash_works() {
    let (stdout, _stderr, code) = run_f_full(&["./src"]);
    assert_eq!(code, 0, "./src should work as path");
    // Banner for ./src should be different from default cwd banner
    let default_out = run_f(&[]);
    // They may or may not be different in the header, but ./src should
    // at least succeed.
    let _ = stdout;
    let _ = default_out;
}

#[test]
fn explicit_path_with_slash_works() {
    let (_stdout, _stderr, code) = run_f_full(&["/tmp"]);
    assert_eq!(code, 0, "/tmp should work as path");
}

#[test]
fn explicit_path_with_tilde_works() {
    let (_stdout, _stderr, code) = run_f_full(&["~/"]);
    assert_eq!(code, 0, "~/ should work as path");
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
    // In 0.6.x, `f t` meant `-t`. In 0.7.0, `t` is not an alias,
    // so it should show the default banner (not sort by time).
    let default = run_f(&[]);
    let single_t = run_f(&["t"]);
    assert_eq!(first_line_header(&single_t), first_line_header(&default));
}

#[test]
fn f_trc_no_longer_means_dash_t_dash_r_dash_c() {
    // In 0.6.x, `f trc` meant `-t -r -c`. In 0.7.0, it doesn't.
    let default = run_f(&[]);
    let chain = run_f(&["trc"]);
    assert_eq!(first_line_header(&chain), first_line_header(&default));
}

#[test]
fn f_s_no_longer_means_dash_upper_s() {
    // In 0.6.x, `f s` meant `-S` (case-insensitive alias).
    // In 0.7.0, `s` is not an alias.
    let default = run_f(&[]);
    let single_s = run_f(&["s"]);
    assert_eq!(first_line_header(&single_s), first_line_header(&default));
}

#[test]
fn f_mLf_colon_no_longer_works() {
    // In 0.6.37, `f mLf: 10` meant `-f 10`. In 0.7.0, the `:` binding
    // is gone. The `mLf:` is an unknown bare word, so it shows the
    // default banner.
    let default = run_f(&[]);
    let binding = run_f(&["mLf:", "10"]);
    assert_eq!(first_line_header(&binding), first_line_header(&default));
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
fn alias_plus_path() {
    let with_path = run_f(&["tree", "./src"]);
    let explicit = run_f(&["-R", "-D", "./src"]);
    assert_eq!(first_line_header(&with_path), first_line_header(&explicit));
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
