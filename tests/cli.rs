use std::fs;
use std::process::Command;

use serde_json::Value;
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
    assert!(project_dir.join("TODO.md").is_file());
    assert!(project_dir.join("plans/README.md").is_file());
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
    assert!(stdout.contains("TODO.md: yes"));
    assert!(stdout.contains("plans/README.md: yes"));
    assert!(stdout.contains("configured: yes"));
    assert!(stdout.contains("ready: yes"));
    assert!(stdout.contains("working tree clean: yes"));
}

#[test]
fn check_command_returns_exit_code_two_when_project_is_not_configured() {
    let workspace = tempdir().unwrap();
    let project_dir = workspace.path().join("incomplete");
    fs::create_dir(&project_dir).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_codex-helper"))
        .arg("check")
        .arg(&project_dir)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stdout).contains("configured: no"));
}

#[test]
fn check_command_only_requires_clean_worktree_when_requested() {
    let workspace = tempdir().unwrap();
    let helper_dir = workspace.path().join("CodexHelper");
    fs::create_dir(&helper_dir).unwrap();

    let new_output = Command::new(env!("CARGO_BIN_EXE_codex-helper"))
        .args(["new", "demo", "--initial-commit"])
        .current_dir(&helper_dir)
        .output()
        .unwrap();
    assert!(new_output.status.success());

    let project_dir = workspace.path().join("demo");
    fs::write(project_dir.join("README.md"), "changed\n").unwrap();

    let normal = Command::new(env!("CARGO_BIN_EXE_codex-helper"))
        .arg("check")
        .arg(&project_dir)
        .output()
        .unwrap();
    assert!(normal.status.success());
    assert!(String::from_utf8_lossy(&normal.stdout).contains("working tree clean: no"));

    let strict = Command::new(env!("CARGO_BIN_EXE_codex-helper"))
        .arg("check")
        .arg(&project_dir)
        .arg("--require-clean")
        .output()
        .unwrap();
    assert_eq!(strict.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&strict.stdout).contains("ready: no"));
}

#[test]
fn check_command_supports_json_output() {
    let workspace = tempdir().unwrap();
    let helper_dir = workspace.path().join("CodexHelper");
    fs::create_dir(&helper_dir).unwrap();

    let new_output = Command::new(env!("CARGO_BIN_EXE_codex-helper"))
        .args(["new", "demo", "--initial-commit"])
        .current_dir(&helper_dir)
        .output()
        .unwrap();
    assert!(new_output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_codex-helper"))
        .arg("check")
        .arg(workspace.path().join("demo"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());

    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["configured"], true);
    assert_eq!(value["ready"], true);
    assert_eq!(value["checks"]["todo_plan_links_valid"], true);
}

#[test]
fn init_command_adds_missing_files_to_existing_project() {
    let workspace = tempdir().unwrap();
    let project_dir = workspace.path().join("existing");
    fs::create_dir(&project_dir).unwrap();
    fs::write(project_dir.join("README.md"), "# Existing\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_codex-helper"))
        .arg("init")
        .arg(&project_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(project_dir.join("README.md")).unwrap(),
        "# Existing\n"
    );
    assert!(project_dir.join("TODO.md").is_file());
    assert!(project_dir.join("plans/README.md").is_file());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Initialized project"));
}
