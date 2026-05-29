use assert_cmd::Command;

#[test]
fn test_banner_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("banner").arg("--help").assert().success();
}

#[test]
fn test_env_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("env").arg("--help").assert().success();
}

#[test]
fn test_pins_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("pins").arg("--help").assert().success();
}

#[test]
fn test_stats_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("stats").arg("--help").assert().success();
}

#[test]
fn test_clipboard_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("clipboard").arg("--help").assert().success();
}

#[test]
fn test_sessions_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("sessions").arg("--help").assert().success();
}

#[test]
fn test_diff_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("diff").arg("--help").assert().success();
}

#[test]
fn test_completion_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("completion").arg("--help").assert().success();
}
