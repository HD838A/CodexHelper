use std::fs;
use std::process::Command;

use codex_helper::{
    check_project_health, create_project, default_target_parent, validate_project_name,
    CreateProjectOptions,
};
use tempfile::tempdir;

#[test]
fn rejects_invalid_project_names() {
    for name in [
        "",
        "   ",
        "../demo",
        "nested/demo",
        "nested\\demo",
        ".",
        "..",
    ] {
        assert!(validate_project_name(name).is_err(), "{name:?} should fail");
    }
}

#[test]
fn accepts_simple_project_names() {
    for name in ["demo", "PhotoTools", "codex-helper_2"] {
        assert!(validate_project_name(name).is_ok(), "{name:?} should pass");
    }
}

#[test]
fn resolves_default_target_parent_from_current_directory() {
    let workspace = tempdir().unwrap();
    let helper_dir = workspace.path().join("CodexHelper");
    fs::create_dir(&helper_dir).unwrap();

    assert_eq!(
        default_target_parent(&helper_dir).unwrap(),
        workspace.path()
    );
}

#[test]
fn creates_empty_project_with_common_files_and_git_repo() {
    let workspace = tempdir().unwrap();
    let helper_dir = workspace.path().join("CodexHelper");
    fs::create_dir(&helper_dir).unwrap();

    let created = create_project(CreateProjectOptions {
        current_dir: helper_dir.clone(),
        project_name: "PhotoTools".to_string(),
        target_dir: None,
        initial_commit: false,
    })
    .unwrap();

    let project_dir = workspace.path().join("PhotoTools");
    assert_eq!(created.path, project_dir);
    assert!(project_dir.is_dir());
    assert!(project_dir.join(".git").is_dir());
    assert!(project_dir.join(".gitignore").is_file());
    assert!(project_dir.join("README.md").is_file());
    assert!(project_dir.join("AGENTS.md").is_file());
    assert!(project_dir
        .join(".codex/environments/environment.toml")
        .is_file());

    let agents = fs::read_to_string(project_dir.join("AGENTS.md")).unwrap();
    assert!(agents.contains(".codex/environments/environment.toml"));
    assert!(agents.contains("update it to the correct state"));
    assert!(!agents.contains("RTK"));
    assert!(!agents.contains("rtk"));
}

#[test]
fn creates_project_in_explicit_target_dir() {
    let workspace = tempdir().unwrap();
    let helper_dir = workspace.path().join("CodexHelper");
    let projects_dir = workspace.path().join("Projects");
    fs::create_dir(&helper_dir).unwrap();
    fs::create_dir(&projects_dir).unwrap();

    let created = create_project(CreateProjectOptions {
        current_dir: helper_dir,
        project_name: "PhotoTools".to_string(),
        target_dir: Some(projects_dir.clone()),
        initial_commit: false,
    })
    .unwrap();

    assert_eq!(created.path, projects_dir.join("PhotoTools"));
    assert!(created.path.join(".git").is_dir());
}

#[test]
fn can_create_initial_commit() {
    let workspace = tempdir().unwrap();
    let helper_dir = workspace.path().join("CodexHelper");
    fs::create_dir(&helper_dir).unwrap();

    let created = create_project(CreateProjectOptions {
        current_dir: helper_dir,
        project_name: "PhotoTools".to_string(),
        target_dir: None,
        initial_commit: true,
    })
    .unwrap();

    let output = Command::new("git")
        .arg("-C")
        .arg(&created.path)
        .arg("log")
        .arg("--oneline")
        .arg("-1")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Initial commit"));
}

#[test]
fn checks_project_health() {
    let workspace = tempdir().unwrap();
    let helper_dir = workspace.path().join("CodexHelper");
    fs::create_dir(&helper_dir).unwrap();

    let created = create_project(CreateProjectOptions {
        current_dir: helper_dir,
        project_name: "PhotoTools".to_string(),
        target_dir: None,
        initial_commit: true,
    })
    .unwrap();

    let health = check_project_health(&created.path).unwrap();

    assert!(health.is_ready());
    assert!(health.is_git_repo);
    assert!(health.has_readme);
    assert!(health.has_gitignore);
    assert!(health.has_agents);
    assert!(health.has_environment);
    assert_eq!(health.dirty_worktree, Some(false));
}

#[test]
fn fails_when_target_directory_exists_without_modifying_it() {
    let workspace = tempdir().unwrap();
    let helper_dir = workspace.path().join("CodexHelper");
    let existing = workspace.path().join("PhotoTools");
    fs::create_dir(&helper_dir).unwrap();
    fs::create_dir(&existing).unwrap();
    fs::write(existing.join("keep.txt"), "keep").unwrap();

    let result = create_project(CreateProjectOptions {
        current_dir: helper_dir,
        project_name: "PhotoTools".to_string(),
        target_dir: None,
        initial_commit: false,
    });

    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(existing.join("keep.txt")).unwrap(),
        "keep"
    );
    assert!(!existing.join("README.md").exists());
}
