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
fn test_install_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("install").arg("--help").assert().success();
}

#[test]
fn test_uninstall_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("uninstall").arg("--help").assert().success();
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("-V").assert().success();
}

#[test]
fn test_uninstall_roundtrip() {
    // `f install` then `f uninstall` must leave the rc file and bin dir
    // exactly as they were (the phantom subcommands removed in v0.3.0
    // previously had no removal path at all).
    use std::fs;
    let home = std::env::temp_dir().join(format!("fab-it-{}", std::process::id()));
    fs::create_dir_all(home.join(".local/bin")).unwrap();
    fs::write(home.join(".zshrc"), "export FOO=bar\n").unwrap();

    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.env("HOME", &home).arg("install").assert().success();
    let rc = fs::read_to_string(home.join(".zshrc")).unwrap();
    assert!(rc.contains("fab-shell.zsh"), "install should add the source line");
    assert!(home.join(".local/bin/fab-shell.zsh").exists());

    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.env("HOME", &home).arg("uninstall").assert().success();
    let rc = fs::read_to_string(home.join(".zshrc")).unwrap();
    assert!(
        !rc.contains("fab-shell"),
        "uninstall should remove the source line, got: {rc}"
    );
    assert!(!home.join(".local/bin/fab-shell.zsh").exists());

    // Idempotent second uninstall.
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.env("HOME", &home).arg("uninstall").assert().success();
    fs::remove_dir_all(home).ok();
}

#[test]
fn test_config_help() {
    let mut cmd = Command::cargo_bin("f").unwrap();
    cmd.arg("config").arg("--help").assert().success();
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
