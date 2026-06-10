use assert_cmd::cargo::CommandCargoExt;
use assert_cmd::Command;
use std::process::Command as StdCommand;

#[test]
fn test_banner_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("banner").arg("--help").assert().success();
}

#[test]
fn test_env_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("env").arg("--help").assert().success();
}

#[test]
fn test_pins_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("pins").arg("--help").assert().success();
}

#[test]
fn test_stats_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("stats").arg("--help").assert().success();
}

#[test]
fn test_clipboard_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("clipboard").arg("--help").assert().success();
}

#[test]
fn test_sessions_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("sessions").arg("--help").assert().success();
}

#[test]
fn test_diff_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("diff").arg("--help").assert().success();
}

#[test]
fn test_completion_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("completion").arg("--help").assert().success();
}

#[test]
fn test_config_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("config").arg("--help").assert().success();
}

#[test]
fn test_mv_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("mv").arg("--help").assert().success();
}

#[test]
fn test_cp_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("cp").arg("--help").assert().success();
}

#[test]
fn test_rm_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("rm").arg("--help").assert().success();
}

#[test]
fn test_trash_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("trash").arg("--help").assert().success();
}

#[test]
fn test_open_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("open").arg("--help").assert().success();
}

#[test]
fn test_do_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("do").arg("--help").assert().success();
}

#[test]
fn test_peek_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("peek").arg("--help").assert().success();
}

#[test]
fn test_root_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("root").arg("--help").assert().success();
}

#[test]
fn test_daemon_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("daemon").arg("--help").assert().success();
}

#[test]
fn test_banner_default_no_args() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.assert().success();
}

#[test]
fn test_banner_hidden_flag() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.args(["banner", "--hidden"]).assert().success();
}

#[test]
fn test_banner_filter_flag() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.args(["banner", "--filter", "rs"]).assert().success();
}

#[test]
fn test_banner_max_flag() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.args(["banner", "--max", "5"]).assert().success();
}

#[test]
fn test_banner_group_flag() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.args(["banner", "--group"]).assert().success();
}

#[test]
fn test_banner_sort_flag() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.args(["banner", "--sort", "size"]).assert().success();
}

#[test]
fn test_banner_reverse_flag() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.args(["banner", "--sort", "name", "--reverse"])
        .assert()
        .success();
}

#[test]
fn test_banner_json_output() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.args(["banner", "--json"]).assert().success();
}

#[test]
fn test_banner_raw_output() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.args(["banner", "--raw"]).assert().success();
}

#[test]
fn test_banner_combined_flags() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.args([
        "banner", "--hidden", "--filter", "rs", "--max", "10", "--sort", "size", "--group",
    ])
    .assert()
    .success();
}

/// Regression: `f N` must return the same path shown at position [N] in the banner.
/// Previously, `navigate_by_number` used `cwd` instead of the resolved `path`,
/// causing a mismatch when the user passed a path argument or when the
/// canonicalization differed.
#[test]
fn test_navigate_by_number_matches_banner() {
    // Create a temp directory with known files
    let tmp = std::env::temp_dir().join("fab_nav_test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    for name in &[
        "alpha.md",
        "bravo.md",
        "charlie.md",
        "delta.txt",
        "echo.txt",
    ] {
        std::fs::write(tmp.join(name), b"test content").unwrap();
    }

    // Get the JSON output from inside the temp dir to know the sort order.
    // `f banner --json` uses the current directory when no path is given,
    // matching how the shell function invokes `f N`.
    let json_output = StdCommand::cargo_bin("f")
        .unwrap()
        .args(["banner", "--json"])
        .current_dir(&tmp)
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let items = json["items"].as_array().unwrap();
    assert!(!items.is_empty(), "banner should list the temp dir items");

    // For each item, verify that `f N` returns the same path.
    // Run from inside the temp dir so the banner and navigation agree.
    for (idx, item) in items.iter().enumerate() {
        let num = idx + 1;
        let expected_path = item["path"].as_str().unwrap();
        let actual = StdCommand::cargo_bin("f")
            .unwrap()
            .args(["banner", &num.to_string()])
            .current_dir(&tmp)
            .output()
            .unwrap();
        let actual_path = String::from_utf8(actual.stdout).unwrap().trim().to_string();
        assert_eq!(
            actual_path, expected_path,
            "f {num} returned {actual_path} but banner shows {expected_path} at [{num}]"
        );
    }

    let _ = std::fs::remove_dir_all(&tmp);
}
