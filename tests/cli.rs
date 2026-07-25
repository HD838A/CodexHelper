use std::fs;
use std::process::Command;

use tempfile::tempdir;

#[test]
fn new_command_creates_project_next_to_current_directory() {
    let workspace = tempdir().unwrap();
    let helper_dir = workspace.path().join("CodexHelper");
    fs::create_dir(&helper_dir).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_codex-helper"))
        .arg("new")
        .arg("demo")
        .current_dir(&helper_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let project_dir = workspace.path().join("demo");
    assert!(project_dir.join(".git").is_dir());
    assert!(project_dir.join(".gitignore").is_file());
    assert!(project_dir.join("README.md").is_file());
    assert!(project_dir.join("AGENTS.md").is_file());
    assert!(project_dir
        .join(".codex/environments/environment.toml")
        .is_file());
}

#[test]
fn new_command_accepts_target_dir_and_initial_commit() {
    let workspace = tempdir().unwrap();
    let helper_dir = workspace.path().join("CodexHelper");
    let projects_dir = workspace.path().join("Projects");
    fs::create_dir(&helper_dir).unwrap();
    fs::create_dir(&projects_dir).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_codex-helper"))
        .arg("new")
        .arg("demo")
        .arg("--target-dir")
        .arg(&projects_dir)
        .arg("--initial-commit")
        .current_dir(&helper_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log = Command::new("git")
        .arg("-C")
        .arg(projects_dir.join("demo"))
        .arg("log")
        .arg("--oneline")
        .arg("-1")
        .output()
        .unwrap();

    assert!(log.status.success());
    assert!(String::from_utf8_lossy(&log.stdout).contains("Initial commit"));
}

#[test]
fn check_command_reports_ready_project() {
    let workspace = tempdir().unwrap();
    let helper_dir = workspace.path().join("CodexHelper");
    fs::create_dir(&helper_dir).unwrap();

    let new_output = Command::new(env!("CARGO_BIN_EXE_codex-helper"))
        .arg("new")
        .arg("demo")
        .arg("--initial-commit")
        .current_dir(&helper_dir)
        .output()
        .unwrap();
    assert!(new_output.status.success());

    let check_output = Command::new(env!("CARGO_BIN_EXE_codex-helper"))
        .arg("check")
        .arg(workspace.path().join("demo"))
        .output()
        .unwrap();

    assert!(check_output.status.success());
    let stdout = String::from_utf8_lossy(&check_output.stdout);
    assert!(stdout.contains("ready: yes"));
    assert!(stdout.contains("working tree clean: yes"));
}
