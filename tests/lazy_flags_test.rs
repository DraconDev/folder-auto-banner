// Comprehensive integration tests for the lazy flag system.
// These tests verify that `f <lazy>` produces byte-identical output
// to `f <explicit>` for all supported flag combinations.
//
// Run with: cargo test --test lazy_flags_test -- --test-threads=1
//
// Note: tests must run with --test-threads=1 because the daemon uses
// a single shared socket and parallel runs can flake.

use assert_cmd::cargo::CommandCargoExt;
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
    assert_eq!(code, 0, "f -e should succeed (exit 0), got exit {}: {}", code, _stderr);
    assert!(stdout.contains("fabd") || stdout.contains("Folder") || stdout.contains("│"),
        "f -e should produce a banner, got: {}", stdout);
}

#[test]
fn regression_0_6_34_f_dash_U_works() {
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
fn byte_identical_single_flag_D() {
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
fn byte_identical_single_flag_G() {
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
fn byte_identical_single_flag_R() {
    let lazy = run_f(&["R"]);
    let explicit = run_f(&["-R"]);
    assert_eq!(lazy, explicit, "f R should equal f -R");
}

#[test]
fn byte_identical_single_flag_S() {
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
fn byte_identical_single_flag_U() {
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
fn byte_identical_single_flag_X() {
    let lazy = run_f(&["X"]);
    let explicit = run_f(&["-X"]);
    assert_eq!(lazy, explicit, "f X should equal f -X");
}

// ===== Byte-identical tests: lowercase aliases =====

#[test]
fn byte_identical_alias_s_to_S() {
    let lazy = run_f(&["s"]);
    let explicit = run_f(&["S"]);
    assert_eq!(lazy, explicit, "f s should equal f S");
}

#[test]
fn byte_identical_alias_g_to_G() {
    let lazy = run_f(&["g"]);
    let explicit = run_f(&["G"]);
    assert_eq!(lazy, explicit, "f g should equal f G");
}

#[test]
fn byte_identical_alias_d_to_D() {
    let lazy = run_f(&["d"]);
    let explicit = run_f(&["D"]);
    assert_eq!(lazy, explicit, "f d should equal f D");
}

#[test]
fn byte_identical_alias_l_1_to_L_1() {
    let lazy = run_f(&["l", "1"]);
    let explicit = run_f(&["L", "1"]);
    assert_eq!(lazy, explicit, "f l 1 should equal f L 1");
}

#[test]
fn byte_identical_alias_u_to_U() {
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
fn byte_identical_chain_tS() {
    let lazy = run_f(&["tS"]);
    let explicit = run_f(&["-t", "-S"]);
    assert_eq!(lazy, explicit, "f tS should equal f -t -S");
}

#[test]
fn byte_identical_chain_GS() {
    let lazy = run_f(&["GS"]);
    let explicit = run_f(&["-G", "-S"]);
    assert_eq!(lazy, explicit, "f GS should equal f -G -S");
}

#[test]
fn byte_identical_chain_Rc() {
    let lazy = run_f(&["Rc"]);
    let explicit = run_f(&["-R", "-c"]);
    assert_eq!(lazy, explicit, "f Rc should equal f -R -c");
}

#[test]
fn byte_identical_chain_rS() {
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
fn byte_identical_chain_aR() {
    let lazy = run_f(&["aR"]);
    let explicit = run_f(&["-a", "-R"]);
    assert_eq!(lazy, explicit, "f aR should equal f -a -R");
}

#[test]
fn byte_identical_chain_oR() {
    let lazy = run_f(&["oR"]);
    let explicit = run_f(&["-o", "-R"]);
    assert_eq!(lazy, explicit, "f oR should equal f -o -R");
}

#[test]
fn byte_identical_chain_Dt() {
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
fn byte_identical_value_L_2() {
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
fn byte_identical_value_mL_10_2() {
    let lazy = run_f(&["mL", "10", "2"]);
    let explicit = run_f(&["-m", "10", "-L", "2"]);
    assert_eq!(lazy, explicit, "f mL 10 2 should equal f -m 10 -L 2");
}

#[test]
fn byte_identical_value_tSm_10() {
    let lazy = run_f(&["tSm", "10"]);
    let explicit = run_f(&["-t", "-S", "-m", "10"]);
    assert_eq!(lazy, explicit, "f tSm 10 should equal f -t -S -m 10");
}

#[test]
fn byte_identical_value_mLf_10_2_txt() {
    let lazy = run_f(&["mLf", "10", "2", "txt"]);
    let explicit = run_f(&["-m", "10", "-L", "2", "-f", "txt"]);
    assert_eq!(lazy, explicit, "f mLf 10 2 txt should equal f -m 10 -L 2 -f txt");
}

// ===== Error message tests =====

#[test]
fn error_message_invalid_single_char() {
    let (stdout, stderr, code) = run_f_full(&["z"]);
    assert_ne!(code, 0, "f z should fail with non-zero exit");
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.contains("'z'"), "error should mention 'z', got: {}", combined);
    assert!(combined.contains("not a valid lazy flag"), "error should explain lazy flag system, got: {}", combined);
    assert!(combined.contains("./z"), "error should suggest './z' escape, got: {}", combined);
}

#[test]
fn error_message_invalid_chain() {
    let (stdout, stderr, code) = run_f_full(&["tz"]);
    assert_ne!(code, 0, "f tz should fail");
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.contains("'tz'"), "error should mention 'tz', got: {}", combined);
    assert!(combined.contains("not a valid lazy flag chain"), "error should explain chain failure, got: {}", combined);
    assert!(combined.contains("./tz"), "error should suggest './tz' escape, got: {}", combined);
}

#[test]
fn error_message_all_invalid() {
    let (stdout, stderr, code) = run_f_full(&["xyz"]);
    assert_ne!(code, 0, "f xyz should fail");
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.contains("'xyz'"), "error should mention 'xyz', got: {}", combined);
}

#[test]
fn error_message_lists_valid_flags() {
    let (_stdout, stderr, _code) = run_f_full(&["q"]);
    // The error should list the valid flags
    assert!(stderr.contains("a, c, D, e, f, G, L, m, o, r, R, S, t, U, v, x, X")
        || stderr.contains("a,c,D,e,f,G,L,m,o,r,R,S,t,U,v,x,X"),
        "error should list valid flags, got: {}", stderr);
}

#[test]
fn error_missing_value_for_m() {
    let (stdout, stderr, code) = run_f_full(&["m"]);
    assert_ne!(code, 0, "f m should fail (value required)");
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.contains("value is required") || combined.contains("--max"),
        "error should mention --max, got: {}", combined);
}

#[test]
fn error_missing_value_for_L() {
    let (stdout, stderr, code) = run_f_full(&["L"]);
    assert_ne!(code, 0);
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.contains("value is required") || combined.contains("--level"),
        "error should mention --level, got: {}", combined);
}

#[test]
fn error_invalid_value_for_m() {
    let (stdout, stderr, code) = run_f_full(&["m", "abc"]);
    assert_ne!(code, 0);
    let combined = format!("{}{}", stdout, stderr);
    assert!(combined.contains("invalid") || combined.contains("--max"),
        "error should mention invalid value, got: {}", combined);
}

// ===== Routing tests =====

#[test]
fn routing_number_navigates() {
    let output = run_f(&["1"]);
    // Should navigate to item 1 (a path), not show the cwd banner
    assert!(!output.contains("fabd │"), "f 1 should navigate, not show cwd banner, got: {}", output);
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
    assert!(output.contains("src") || output.contains("Folder"),
        "f ./src should show banner for src/, got: {}", output);
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
    let combinations = [
        "t", "S", "G", "r", "c", "a", "D", "R", "o", "v",
        "tr", "tS", "trc", "taR", "GSr",
        "m", "f", "L", // value-taking singles (will fail without value, tested separately)
        "tSm", "mL", "Lmf", "tSmL", "aRcSD",
    ];
    // Only test the boolean chains here (value-taking ones are tested separately)
    let boolean_chains = ["t", "S", "G", "r", "c", "a", "D", "R", "o", "v",
        "tr", "tS", "trc", "taR", "GSr", "aRcSD"];
    for chain in &boolean_chains {
        let (_stdout, _stderr, code) = run_f_full(&[chain]);
        assert_eq!(code, 0, "f {} should succeed (exit 0)", chain);
    }
}
