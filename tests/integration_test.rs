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

#[test]
fn test_config_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("config").arg("--help").assert().success();
}

#[test]
fn test_mv_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("mv").arg("--help").assert().success();
}

#[test]
fn test_cp_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("cp").arg("--help").assert().success();
}

#[test]
fn test_rm_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("rm").arg("--help").assert().success();
}

#[test]
fn test_trash_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("trash").arg("--help").assert().success();
}

#[test]
fn test_open_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("open").arg("--help").assert().success();
}

#[test]
fn test_do_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("do").arg("--help").assert().success();
}

#[test]
fn test_peek_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("peek").arg("--help").assert().success();
}

#[test]
fn test_root_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("root").arg("--help").assert().success();
}

#[test]
fn test_daemon_help() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.arg("daemon").arg("--help").assert().success();
}

#[test]
fn test_banner_default_no_args() {
    let mut cmd = Command::cargo_bin("fm").unwrap();
    cmd.assert().success();
}
