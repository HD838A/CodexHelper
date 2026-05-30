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
}
