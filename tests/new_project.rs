use std::fs;
use std::process::Command;

use codex_helper::{
    check_project_health, create_project, default_target_parent, initialize_project,
    validate_project_name, CreateProjectOptions,
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
        "CON",
        "aux.md",
        "trailing.",
        "trailing ",
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
    assert!(project_dir.join("TODO.md").is_file());
    assert!(project_dir.join("plans/README.md").is_file());
    assert!(!project_dir.join("PLAN.md").exists());
    assert!(project_dir.join("README.md").is_file());
    assert!(project_dir.join("AGENTS.md").is_file());
    assert!(project_dir
        .join(".codex/environments/environment.toml")
        .is_file());

    let agents = fs::read_to_string(project_dir.join("AGENTS.md")).unwrap();
    assert!(agents.contains(".codex/environments/environment.toml"));
    assert!(agents.contains("update it to the correct state"));
    assert!(agents.contains("one plan per concrete requirement"));
    assert!(agents.contains("Read the linked requirement plan"));
    assert!(agents.contains("Keep task status in `TODO.md`"));
    assert!(!agents.contains("RTK"));
    assert!(!agents.contains("rtk"));

    let todo = fs::read_to_string(project_dir.join("TODO.md")).unwrap();
    assert!(!todo.contains("T-001"));

    let ignored = Command::new("git")
        .arg("-C")
        .arg(&project_dir)
        .arg("check-ignore")
        .args(["TODO.md", "README.md", "AGENTS.md", "plans/README.md"])
        .output()
        .unwrap();
    assert!(
        !ignored.status.success(),
        "generated project documents must not be ignored"
    );
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

    let branch = Command::new("git")
        .arg("-C")
        .arg(&created.path)
        .args(["branch", "--show-current"])
        .output()
        .unwrap();
    assert!(branch.status.success());
    assert_eq!(String::from_utf8_lossy(&branch.stdout).trim(), "main");
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
    assert!(health.has_todo);
    assert!(health.has_plans_guide);
    assert!(health.has_readme);
    assert!(health.has_gitignore);
    assert!(health.has_agents);
    assert!(health.has_environment);
    assert!(health.management_docs_not_ignored);
    assert!(health.todo_plan_links_valid);
    assert_eq!(health.dirty_worktree, Some(false));
}

#[test]
fn validates_todo_plan_links_and_gitignore_rules() {
    let workspace = tempdir().unwrap();
    let helper_dir = workspace.path().join("CodexHelper");
    fs::create_dir(&helper_dir).unwrap();

    let created = create_project(CreateProjectOptions {
        current_dir: helper_dir,
        project_name: "PhotoTools".to_string(),
        target_dir: None,
        initial_commit: false,
    })
    .unwrap();

    fs::write(
        created.path.join("TODO.md"),
        "# TODO\n\n## Now\n\n- [ ] T-001 Missing plan\n",
    )
    .unwrap();
    assert!(
        !check_project_health(&created.path)
            .unwrap()
            .todo_plan_links_valid
    );

    fs::write(
        created.path.join("TODO.md"),
        "# TODO\n\n## Now\n\n- [ ] [T-001 Add import](plans/T-001-add-import.md)\n",
    )
    .unwrap();
    fs::write(
        created.path.join("plans/T-001-add-import.md"),
        "# T-001 Add import\n",
    )
    .unwrap();
    assert!(
        check_project_health(&created.path)
            .unwrap()
            .todo_plan_links_valid
    );

    let mut gitignore = fs::read_to_string(created.path.join(".gitignore")).unwrap();
    gitignore.push_str("\nTODO.md\n");
    fs::write(created.path.join(".gitignore"), gitignore).unwrap();
    assert!(
        !check_project_health(&created.path)
            .unwrap()
            .management_docs_not_ignored
    );
}

#[test]
fn initializes_existing_project_without_overwriting_files() {
    let workspace = tempdir().unwrap();
    let project_dir = workspace.path().join("ExistingProject");
    fs::create_dir(&project_dir).unwrap();
    fs::write(project_dir.join("README.md"), "# Keep this README\n").unwrap();
    fs::write(project_dir.join(".gitignore"), "*.md\n").unwrap();

    let initialized = initialize_project(&project_dir).unwrap();

    assert!(initialized.initialized_git);
    assert!(initialized.updated_gitignore);
    assert_eq!(
        fs::read_to_string(project_dir.join("README.md")).unwrap(),
        "# Keep this README\n"
    );
    assert!(project_dir.join("TODO.md").is_file());
    assert!(project_dir.join("plans/README.md").is_file());
    assert!(project_dir.join("AGENTS.md").is_file());
    assert!(project_dir
        .join(".codex/environments/environment.toml")
        .is_file());
    assert!(
        check_project_health(&project_dir)
            .unwrap()
            .management_docs_not_ignored
    );

    let second = initialize_project(&project_dir).unwrap();
    assert!(!second.initialized_git);
    assert!(!second.updated_gitignore);
    assert!(second.created_files.is_empty());
}

#[test]
fn dirty_worktree_only_fails_when_cleanliness_is_required() {
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
    fs::write(created.path.join("README.md"), "changed\n").unwrap();

    let health = check_project_health(&created.path).unwrap();
    assert!(health.is_configured());
    assert!(health.is_ready());
    assert!(health.meets_requirements(false));
    assert!(!health.meets_requirements(true));
}

#[test]
fn checks_git_worktree_health() {
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
    let worktree_dir = workspace.path().join("PhotoTools-worktree");

    let output = Command::new("git")
        .arg("-C")
        .arg(&created.path)
        .args(["worktree", "add"])
        .arg(&worktree_dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(worktree_dir.join(".git").is_file());
    assert!(check_project_health(&worktree_dir).unwrap().is_ready());
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
