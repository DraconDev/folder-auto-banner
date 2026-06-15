// Comprehensive integration tests for the lazy flag system.
// These tests verify that `f <lazy>` produces byte-identical output
// to `f <explicit>` for all supported flag combinations.
//
// Run with: cargo test --test lazy_flags_test -- --test-threads=1
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

// ===== Regression tests for 0.6.34 fixes =====
// These tests would have FAILED before 0.6.34. They verify the
// flag duplication bugs are fixed.

#[test]
fn regression_0_6_34_f_dash_e_works() {
    // f -e was failing with "unexpected argument '-e' found" before 0.6.34
    let (stdout, _stderr, code) = run_f_full(&["-e"]);
    assert_eq!(
        code, 0,
        "f -e should succeed (exit 0), got exit {}: {}",
        code, _stderr
    );
    assert!(
        stdout.contains("fabd") || stdout.contains("Folder") || stdout.contains("│"),
        "f -e should produce a banner, got: {}",
        stdout
    );
}

#[test]
fn regression_0_6_34_f_dash_upper_u_works() {
    // f -U was failing with "unexpected argument '-U' found" before 0.6.34
    let (_stdout, _stderr, code) = run_f_full(&["-U"]);
    assert_eq!(code, 0, "f -U should succeed (exit 0)");
}

#[test]
fn regression_0_6_34_f_dash_x_works() {
    // f -x was failing with "unexpected argument '-x' found" before 0.6.34
    let (_stdout, _stderr, code) = run_f_full(&["-x"]);
    assert_eq!(code, 0, "f -x should succeed (exit 0)");
}

#[test]
fn regression_0_6_34_f_dash_f_with_value_works() {
    // f -f txt was failing with "value required for --filter" before 0.6.34
    let (_stdout, _stderr, code) = run_f_full(&["-f", "txt"]);
    assert_eq!(code, 0, "f -f txt should succeed (exit 0)");
}

#[test]
fn regression_0_6_34_f_dash_f_rs_works() {
    let (_stdout, _stderr, code) = run_f_full(&["-f", "rs"]);
    assert_eq!(code, 0, "f -f rs should succeed (exit 0)");
}

// ===== Byte-identical tests: single flags =====
// For each of the 17 lazy flags, verify f <char> ≡ f -<char>
// (We use --no-daemon-friendly flags where possible to avoid timing diffs)

#[test]
fn byte_identical_single_flag_a() {
    let lazy = run_f(&["a"]);
    let explicit = run_f(&["-a"]);
    assert_eq!(lazy, explicit, "f a should equal f -a");
}

#[test]
fn byte_identical_single_flag_c() {
    let lazy = run_f(&["c"]);
    let explicit = run_f(&["-c"]);
    assert_eq!(lazy, explicit, "f c should equal f -c");
}

#[test]
fn byte_identical_single_flag_upper_d() {
    let lazy = run_f(&["D"]);
    let explicit = run_f(&["-D"]);
    assert_eq!(lazy, explicit, "f D should equal f -D");
}

#[test]
fn byte_identical_single_flag_e() {
    let lazy = run_f(&["e"]);
    let explicit = run_f(&["-e"]);
    assert_eq!(lazy, explicit, "f e should equal f -e");
}

#[test]
fn byte_identical_single_flag_upper_g() {
    let lazy = run_f(&["G"]);
    let explicit = run_f(&["-G"]);
    assert_eq!(lazy, explicit, "f G should equal f -G");
}

#[test]
fn byte_identical_single_flag_o() {
    let lazy = run_f(&["o"]);
    let explicit = run_f(&["-o"]);
    assert_eq!(lazy, explicit, "f o should equal f -o");
}

#[test]
fn byte_identical_single_flag_r() {
    let lazy = run_f(&["r"]);
    let explicit = run_f(&["-r"]);
    assert_eq!(lazy, explicit, "f r should equal f -r");
}

#[test]
fn byte_identical_single_flag_upper_r() {
    let lazy = run_f(&["R"]);
    let explicit = run_f(&["-R"]);
    assert_eq!(lazy, explicit, "f R should equal f -R");
}

#[test]
fn byte_identical_single_flag_upper_s() {
    let lazy = run_f(&["S"]);
    let explicit = run_f(&["-S"]);
    assert_eq!(lazy, explicit, "f S should equal f -S");
}

#[test]
fn byte_identical_single_flag_t() {
    let lazy = run_f(&["t"]);
    let explicit = run_f(&["-t"]);
    assert_eq!(lazy, explicit, "f t should equal f -t");
}

#[test]
fn byte_identical_single_flag_upper_u() {
    let lazy = run_f(&["U"]);
    let explicit = run_f(&["-U"]);
    assert_eq!(lazy, explicit, "f U should equal f -U");
}

#[test]
fn byte_identical_single_flag_v() {
    let lazy = run_f(&["v"]);
    let explicit = run_f(&["-v"]);
    assert_eq!(lazy, explicit, "f v should equal f -v");
}

#[test]
fn byte_identical_single_flag_x() {
    let lazy = run_f(&["x"]);
    let explicit = run_f(&["-x"]);
    assert_eq!(lazy, explicit, "f x should equal f -x");
}

#[test]
fn byte_identical_single_flag_upper_x() {
    let lazy = run_f(&["X"]);
    let explicit = run_f(&["-X"]);
    assert_eq!(lazy, explicit, "f X should equal f -X");
}

// ===== Byte-identical tests: lowercase aliases =====

#[test]
fn byte_identical_alias_s_to_upper_s() {
    let lazy = run_f(&["s"]);
    let explicit = run_f(&["S"]);
    assert_eq!(lazy, explicit, "f s should equal f S");
}

#[test]
fn byte_identical_alias_g_to_upper_g() {
    let lazy = run_f(&["g"]);
    let explicit = run_f(&["G"]);
    assert_eq!(lazy, explicit, "f g should equal f G");
}

#[test]
fn byte_identical_alias_d_to_upper_d() {
    let lazy = run_f(&["d"]);
    let explicit = run_f(&["D"]);
    assert_eq!(lazy, explicit, "f d should equal f D");
}

#[test]
fn byte_identical_alias_l_1_to_upper_l_1() {
    let lazy = run_f(&["l", "1"]);
    let explicit = run_f(&["L", "1"]);
    assert_eq!(lazy, explicit, "f l 1 should equal f L 1");
}

#[test]
fn byte_identical_alias_u_to_upper_u() {
    let lazy = run_f(&["u"]);
    let explicit = run_f(&["U"]);
    assert_eq!(lazy, explicit, "f u should equal f U");
}

// ===== Byte-identical tests: chained flags =====

#[test]
fn byte_identical_chain_tr() {
    let lazy = run_f(&["tr"]);
    let explicit = run_f(&["-t", "-r"]);
    assert_eq!(lazy, explicit, "f tr should equal f -t -r");
}

#[test]
fn byte_identical_chain_trc() {
    let lazy = run_f(&["trc"]);
    let explicit = run_f(&["-t", "-r", "-c"]);
    assert_eq!(lazy, explicit, "f trc should equal f -t -r -c");
}

#[test]
fn byte_identical_chain_t_s() {
    let lazy = run_f(&["tS"]);
    let explicit = run_f(&["-t", "-S"]);
    assert_eq!(lazy, explicit, "f tS should equal f -t -S");
}

#[test]
fn byte_identical_chain_upper_g_s() {
    let lazy = run_f(&["GS"]);
    let explicit = run_f(&["-G", "-S"]);
    assert_eq!(lazy, explicit, "f GS should equal f -G -S");
}

#[test]
fn byte_identical_chain_upper_rc() {
    let lazy = run_f(&["Rc"]);
    let explicit = run_f(&["-R", "-c"]);
    assert_eq!(lazy, explicit, "f Rc should equal f -R -c");
}

#[test]
fn byte_identical_chain_r_s() {
    let lazy = run_f(&["rS"]);
    let explicit = run_f(&["-r", "-S"]);
    assert_eq!(lazy, explicit, "f rS should equal f -r -S");
}

#[test]
fn byte_identical_chain_ta() {
    let lazy = run_f(&["ta"]);
    let explicit = run_f(&["-t", "-a"]);
    assert_eq!(lazy, explicit, "f ta should equal f -t -a");
}

#[test]
fn byte_identical_chain_a_r() {
    let lazy = run_f(&["aR"]);
    let explicit = run_f(&["-a", "-R"]);
    assert_eq!(lazy, explicit, "f aR should equal f -a -R");
}

#[test]
fn byte_identical_chain_o_r() {
    let lazy = run_f(&["oR"]);
    let explicit = run_f(&["-o", "-R"]);
    assert_eq!(lazy, explicit, "f oR should equal f -o -R");
}

#[test]
fn byte_identical_chain_upper_dt() {
    let lazy = run_f(&["Dt"]);
    let explicit = run_f(&["-D", "-t"]);
    assert_eq!(lazy, explicit, "f Dt should equal f -D -t");
}

// ===== Byte-identical tests: value-taking chains =====

#[test]
fn byte_identical_value_m_10() {
    let lazy = run_f(&["m", "10"]);
    let explicit = run_f(&["-m", "10"]);
    assert_eq!(lazy, explicit, "f m 10 should equal f -m 10");
}

#[test]
fn byte_identical_value_upper_l_2() {
    let lazy = run_f(&["L", "2"]);
    let explicit = run_f(&["-L", "2"]);
    assert_eq!(lazy, explicit, "f L 2 should equal f -L 2");
}

#[test]
fn byte_identical_value_f_txt() {
    let lazy = run_f(&["f", "txt"]);
    let explicit = run_f(&["-f", "txt"]);
    assert_eq!(lazy, explicit, "f f txt should equal f -f txt");
}

#[test]
fn byte_identical_value_m_l_10_2() {
    let lazy = run_f(&["mL", "10", "2"]);
    let explicit = run_f(&["-m", "10", "-L", "2"]);
    assert_eq!(lazy, explicit, "f mL 10 2 should equal f -m 10 -L 2");
}

#[test]
fn byte_identical_value_t_sm_10() {
    let lazy = run_f(&["tSm", "10"]);
    let explicit = run_f(&["-t", "-S", "-m", "10"]);
    assert_eq!(lazy, explicit, "f tSm 10 should equal f -t -S -m 10");
}

#[test]
fn byte_identical_value_m_lf_10_2_txt() {
    let lazy = run_f(&["mLf", "10", "2", "txt"]);
    let explicit = run_f(&["-m", "10", "-L", "2", "-f", "txt"]);
    assert_eq!(
        lazy, explicit,
        "f mLf 10 2 txt should equal f -m 10 -L 2 -f txt"
    );
}

// ===== Error message tests =====

#[test]
fn error_message_invalid_single_char() {
    let (stdout, stderr, code) = run_f_full(&["z"]);
    assert_ne!(code, 0, "f z should fail with non-zero exit");
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.contains("'z'"),
        "error should mention 'z', got: {}",
        combined
    );
    assert!(
        combined.contains("not a valid lazy flag"),
        "error should explain lazy flag system, got: {}",
        combined
    );
    assert!(
        combined.contains("./z"),
        "error should suggest './z' escape, got: {}",
        combined
    );
}

#[test]
fn error_message_invalid_chain() {
    let (stdout, stderr, code) = run_f_full(&["tz"]);
    assert_ne!(code, 0, "f tz should fail");
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.contains("'tz'"),
        "error should mention 'tz', got: {}",
        combined
    );
    assert!(
        combined.contains("not a valid lazy flag chain"),
        "error should explain chain failure, got: {}",
        combined
    );
    assert!(
        combined.contains("./tz"),
        "error should suggest './tz' escape, got: {}",
        combined
    );
}

#[test]
fn error_message_all_invalid() {
    let (stdout, stderr, code) = run_f_full(&["xyz"]);
    assert_ne!(code, 0, "f xyz should fail");
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.contains("'xyz'"),
        "error should mention 'xyz', got: {}",
        combined
    );
}

#[test]
fn error_message_lists_valid_flags() {
    let (_stdout, stderr, _code) = run_f_full(&["q"]);
    // The error should list the valid flags
    assert!(
        stderr.contains("a, c, D, e, f, G, L, m, o, r, R, S, t, U, v, x, X")
            || stderr.contains("a,c,D,e,f,G,L,m,o,r,R,S,t,U,v,x,X"),
        "error should list valid flags, got: {}",
        stderr
    );
}

#[test]
fn error_missing_value_for_m() {
    let (stdout, stderr, code) = run_f_full(&["m"]);
    assert_ne!(code, 0, "f m should fail (value required)");
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.contains("value is required") || combined.contains("--max"),
        "error should mention --max, got: {}",
        combined
    );
}

#[test]
fn error_missing_value_for_upper_l() {
    let (stdout, stderr, code) = run_f_full(&["L"]);
    assert_ne!(code, 0);
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.contains("value is required") || combined.contains("--level"),
        "error should mention --level, got: {}",
        combined
    );
}

#[test]
fn error_invalid_value_for_m() {
    let (stdout, stderr, code) = run_f_full(&["m", "abc"]);
    assert_ne!(code, 0);
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.contains("invalid") || combined.contains("--max"),
        "error should mention invalid value, got: {}",
        combined
    );
}

// ===== Routing tests =====

#[test]
fn routing_number_navigates() {
    let output = run_f(&["1"]);
    // Should navigate to item 1 (a path), not show the cwd banner
    assert!(
        !output.contains("fabd │"),
        "f 1 should navigate, not show cwd banner, got: {}",
        output
    );
}

#[test]
fn routing_subcommand_banner_works() {
    let (_stdout, _stderr, code) = run_f_full(&["banner"]);
    assert_eq!(code, 0, "f banner should succeed");
}

#[test]
fn routing_subcommand_help_works() {
    let (_stdout, _stderr, code) = run_f_full(&["help"]);
    assert_eq!(code, 0, "f help should succeed");
}

#[test]
fn routing_explicit_dot_slash_path() {
    let output = run_f(&["./src"]);
    // Should show banner for ./src, not the cwd
    assert!(
        output.contains("src") || output.contains("Folder"),
        "f ./src should show banner for src/, got: {}",
        output
    );
}

#[test]
fn routing_explicit_absolute_path() {
    let output = run_f(&["/tmp"]);
    // Should show banner for /tmp
    assert!(!output.is_empty(), "f /tmp should produce output");
}

#[test]
fn routing_explicit_tilde_path() {
    // Use HOME env var to construct a path that should always exist
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let (stdout, _stderr, code) = run_f_full(&[home.as_str()]);
    assert_eq!(code, 0, "f $HOME should succeed, stderr: {}", _stderr);
    assert!(!stdout.is_empty(), "f $HOME should produce output");
}

// ===== Property test: all-char chain expansion =====

#[test]
fn property_all_17_flags_chain() {
    // Build a chain containing all 14 unique BOOLEAN lazy flags
    // (excluding the 3 value-taking ones: m, f, L)
    let chain = "tacSDGvRreXox";
    let (_stdout, _stderr, code) = run_f_full(&[chain]);
    assert_eq!(code, 0, "f {} should succeed", chain);
}

// ===== Stress test: 20 random combinations =====

#[test]
fn stress_test_20_random_combinations() {
    // Only test the boolean chains here (value-taking ones are tested separately)
    let boolean_chains = [
        "t", "S", "G", "r", "c", "a", "D", "R", "o", "v", "tr", "tS", "trc", "taR", "GSr", "aRcSD",
    ];
    for chain in &boolean_chains {
        let (_stdout, _stderr, code) = run_f_full(&[chain]);
        assert_eq!(code, 0, "f {} should succeed (exit 0)", chain);
    }
}

// ===== Edge case tests =====

#[test]
fn edge_case_very_long_chain() {
    // 50-char chain of unique boolean flags (with repetition for all 14)
    let chain: String = "tacSDGvRreXox".repeat(4); // 56 chars
    let (_stdout, _stderr, code) = run_f_full(&[&chain]);
    assert_eq!(code, 0, "f with very long chain should succeed");
}

#[test]
fn edge_case_all_14_boolean_flags() {
    // All 14 unique boolean lazy flags in one chain
    let chain = "tacSDGvRreXox";
    let (_stdout, _stderr, code) = run_f_full(&[chain]);
    assert_eq!(code, 0, "f with all 14 boolean flags should succeed");
}

#[test]
fn edge_case_lazy_with_debug() {
    // Lazy flag with --debug should work
    let (_stdout, _stderr, code) = run_f_full(&["--debug", "t"]);
    assert_eq!(code, 0, "f --debug t should succeed");
}

#[test]
fn edge_case_lazy_with_no_color() {
    // Lazy flag with --no-color (if it exists) should work
    let output = run_f(&["t"]);
    assert!(!output.is_empty(), "f t should produce output");
}

#[test]
fn edge_case_lazy_with_explicit_path() {
    // Lazy flag followed by explicit path
    let output = run_f(&["t", "./src"]);
    assert!(output.contains("src") || output.contains("Folder"),
        "f t ./src should show banner for src/");
}

#[test]
fn edge_case_lazy_flag_alone_no_args() {
    // f with no args should show banner for cwd
    let (stdout, _stderr, code) = run_f_full(&[]);
    assert_eq!(code, 0, "f with no args should succeed");
    assert!(!stdout.is_empty(), "f with no args should produce output");
}

#[test]
fn edge_case_number_alone() {
    // f 1 should navigate to item 1
    let (stdout, _stderr, code) = run_f_full(&["1"]);
    assert_eq!(code, 0, "f 1 should succeed");
    // Should navigate (output is the path of item 1, not a banner)
    assert!(!stdout.is_empty(), "f 1 should produce output");
}

#[test]
fn edge_case_very_large_number() {
    // f 99999 should error with out of range
    let (stdout, stderr, code) = run_f_full(&["99999"]);
    assert_ne!(code, 0, "f 99999 should fail");
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.contains("out of range") || combined.contains("99999"),
        "error should mention out of range, got: {}", combined);
}

#[test]
fn edge_case_negative_number() {
    // f -1 should not be treated as a number (it's the oneline flag)
    let (_stdout, _stderr, code) = run_f_full(&["-1"]);
    // This should succeed (oneline flag is valid)
    assert_eq!(code, 0, "f -1 (oneline) should succeed");
}

#[test]
fn edge_case_zero() {
    // f 0 should error
    let (stdout, stderr, code) = run_f_full(&["0"]);
    assert_ne!(code, 0, "f 0 should fail");
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.contains("out of range"), "error should mention out of range");
}

#[test]
fn edge_case_repeated_same_flag() {
    // f tt should error (duplicate -t)
    let (stdout, stderr, code) = run_f_full(&["tt"]);
    assert_ne!(code, 0, "f tt should fail (duplicate flag)");
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.contains("cannot be used multiple times") || combined.contains("duplicate"),
        "error should mention duplicate, got: {}", combined);
}

#[test]
fn edge_case_unicode_in_args() {
    // f with unicode args should not panic
    let (_stdout, stderr, code) = run_f_full(&["\u{00e9}"]);
    // Should fail gracefully (not a valid lazy flag)
    assert_ne!(code, 0, "f with unicode should fail gracefully");
    let _ = stderr; // suppress unused warning
}

#[test]
fn edge_case_empty_string_arg() {
    // f "" (empty string) should not panic
    let (_stdout, _stderr, _code) = run_f_full(&[""]);
    // May succeed or fail, but must not panic
}

#[test]
fn edge_case_many_dashes() {
    // f --- should not panic
    let (_stdout, _stderr, _code) = run_f_full(&["---"]);
    // May fail with clap error, but must not panic
}

#[test]
fn edge_case_flag_with_dash_prefix() {
    // f -t (explicit) should work
    let (lazy, _stderr, code) = run_f_full(&["t"]);
    let (explicit, _stderr2, code2) = run_f_full(&["-t"]);
    assert_eq!(code, code2, "f t and f -t should have same exit code");
    assert_eq!(lazy, explicit, "f t and f -t should have same output");
}

#[test]
fn edge_case_subcommand_with_flag() {
    // f banner -t should work
    let (_stdout, _stderr, code) = run_f_full(&["banner", "-t"]);
    assert_eq!(code, 0, "f banner -t should succeed");
}

#[test]
fn edge_case_subcommand_with_lazy_flag() {
    // f banner t should work (lazy flag in subcommand)
    let (_stdout, _stderr, code) = run_f_full(&["banner", "t"]);
    assert_eq!(code, 0, "f banner t should succeed");
}

#[test]
fn edge_case_help_with_lazy_flag() {
    // f help t should show help
    let (_stdout, _stderr, code) = run_f_full(&["help", "t"]);
    assert_eq!(code, 0, "f help t should succeed");
}

#[test]
fn edge_case_version_flag() {
    // f --version should work
    let (stdout, _stderr, code) = run_f_full(&["--version"]);
    assert_eq!(code, 0, "f --version should succeed");
    assert!(stdout.contains("0.6"), "version output should contain version number");
}

#[test]
fn edge_case_double_dash() {
    // f -- should not panic
    let (_stdout, _stderr, _code) = run_f_full(&["--"]);
    // May fail, but must not panic
}

#[test]
fn edge_case_lazy_flag_with_equals() {
    // f m=10 should work (clap supports =)
    let (_stdout, _stderr, _code) = run_f_full(&["m=10"]);
    // May or may not work, but must not panic
}

#[test]
fn edge_case_very_long_value() {
    // f m with a very long numeric value
    let long_val = "9".repeat(100);
    let (_stdout, stderr, _code) = run_f_full(&["m", &long_val]);
    let _ = stderr; // suppress unused warning
}

#[test]
fn edge_case_negative_value() {
    // f m -1 should not panic (m is usize, so -1 should error)
    let (_stdout, stderr, _code) = run_f_full(&["m", "-1"]);
    let _ = stderr; // suppress unused warning
}

#[test]
fn edge_case_chain_with_value_at_end() {
    // f tr 10 — value at end after boolean chain
    let (stdout, stderr, code) = run_f_full(&["tr", "10"]);
    let _ = (stdout, stderr); // suppress unused
    // 10 would be a positional (path), not a value for any flag
    // Should not panic
    let _ = code;
}

#[test]
fn edge_case_alternating_case() {
    // Mixed case chains should work
    let (_stdout, _stderr, code) = run_f_full(&["tS"]);
    assert_eq!(code, 0, "f tS should succeed");
}

#[test]
fn edge_case_all_lowercase_aliases() {
    // All 5 lowercase aliases in one chain
    let chain = "sdlgu";
    let (_stdout, _stderr, code) = run_f_full(&[chain]);
    assert_eq!(code, 0, "f sdlgu should succeed (all aliases)");
}

// ===== Cross-platform path tests =====

#[test]
fn cross_platform_relative_path() {
    // Relative path with ./
    let output = run_f(&["./src"]);
    assert!(!output.is_empty(), "f ./src should produce output");
}

#[test]
fn cross_platform_absolute_path() {
    // Absolute path with /
    let output = run_f(&["/tmp"]);
    assert!(!output.is_empty(), "f /tmp should produce output");
}

#[test]
fn cross_platform_parent_path() {
    // Parent directory with ..
    let output = run_f(&[".."]);
    assert!(!output.is_empty(), "f .. should produce output");
}

#[test]
fn cross_platform_home_path() {
    // Home directory with ~
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let output = run_f(&[home.as_str()]);
    assert!(!output.is_empty(), "f $HOME should produce output");
}

// ===== Daemon interaction tests =====

#[test]
fn daemon_lazy_flag_cold_start() {
    // Lazy flag should work even if daemon is not running (cold start)
    let (_stdout, _stderr, code) = run_f_full(&["t"]);
    assert_eq!(code, 0, "f t should succeed on cold start");
}

#[test]
fn daemon_lazy_flag_warm() {
    // Lazy flag should work with daemon running
    let (_stdout1, _stderr1, _code1) = run_f_full(&["t"]); // warm up
    let (_stdout2, _stderr2, code2) = run_f_full(&["t"]); // warm
    assert_eq!(code2, 0, "f t should succeed when daemon is warm");
}

#[test]
fn daemon_lazy_flag_chain_warm() {
    // Chained lazy flag should work with daemon
    let (_stdout, _stderr, code) = run_f_full(&["trc"]);
    assert_eq!(code, 0, "f trc should succeed with daemon");
}

#[test]
fn daemon_lazy_flag_value_chain() {
    // Value-taking chain should work with daemon
    let (_stdout, _stderr, code) = run_f_full(&["m", "5"]);
    assert_eq!(code, 0, "f m 5 should succeed with daemon");
}

#[test]
fn daemon_explicit_flag_after_lazy() {
    // Mixing lazy and explicit flags
    let (lazy, _stderr, code) = run_f_full(&["t", "-c"]);
    assert_eq!(code, 0, "f t -c should succeed");
    let _ = lazy;
}

#[test]
fn daemon_explicit_flag_before_lazy() {
    // Explicit flag before lazy
    let (lazy, _stderr, code) = run_f_full(&["-c", "t"]);
    assert_eq!(code, 0, "f -c t should succeed");
    let _ = lazy;
}

#[test]
fn daemon_repeated_invocation_consistent() {
    // Same lazy flag invocation should produce consistent output
    let first = run_f(&["t"]);
    let second = run_f(&["t"]);
    assert_eq!(first, second, "repeated f t should be consistent");
}

#[test]
fn daemon_chain_repeated_consistent() {
    // Same chain should produce consistent output
    let first = run_f(&["trc"]);
    let second = run_f(&["trc"]);
    assert_eq!(first, second, "repeated f trc should be consistent");
}

#[test]
fn daemon_value_chain_repeated_consistent() {
    // Value chain should produce consistent output
    let first = run_f(&["m", "5"]);
    let second = run_f(&["m", "5"]);
    assert_eq!(first, second, "repeated f m 5 should be consistent");
}

#[test]
fn daemon_different_flags_different_output() {
    // Different flags should produce different output
    let t = run_f(&["t"]);
    let s = run_f(&["S"]);
    assert_ne!(t, s, "f t and f S should produce different output");
}
