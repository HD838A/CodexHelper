use std::fs;

use codex_helper::{
    create_project, default_target_parent, validate_project_name, CreateProjectOptions,
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
    })
    .unwrap();

    let project_dir = workspace.path().join("PhotoTools");
    assert_eq!(created.path, project_dir);
    assert!(project_dir.is_dir());
    assert!(project_dir.join(".git").is_dir());
    assert!(project_dir.join(".gitignore").is_file());
    assert!(project_dir.join("README.md").is_file());
    assert!(project_dir.join("AGENTS.md").is_file());

    let agents = fs::read_to_string(project_dir.join("AGENTS.md")).unwrap();
    assert!(!agents.contains("RTK"));
    assert!(!agents.contains("rtk"));
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
    });

    assert!(result.is_err());
    assert_eq!(
        fs::read_to_string(existing.join("keep.txt")).unwrap(),
        "keep"
    );
    assert!(!existing.join("README.md").exists());
}
