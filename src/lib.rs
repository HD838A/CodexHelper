use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

mod templates;

#[derive(Debug, Clone)]
pub struct CreateProjectOptions {
    pub current_dir: PathBuf,
    pub project_name: String,
    pub target_dir: Option<PathBuf>,
    pub initial_commit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedProject {
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitializedProject {
    pub path: PathBuf,
    pub created_files: Vec<PathBuf>,
    pub initialized_git: bool,
    pub updated_gitignore: bool,
}

#[derive(Debug)]
pub enum CodexHelperError {
    InvalidProjectName(String),
    MissingParent(PathBuf),
    ProjectDirectoryNotFound(PathBuf),
    ProjectPathNotDirectory(PathBuf),
    TargetAlreadyExists(PathBuf),
    Io(std::io::Error),
    GitInitFailed(String),
    GitCommitFailed(String),
    CleanupFailed {
        original: Box<CodexHelperError>,
        cleanup: std::io::Error,
    },
}

impl fmt::Display for CodexHelperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProjectName(reason) => write!(f, "invalid project name: {reason}"),
            Self::MissingParent(path) => {
                write!(f, "current directory has no parent: {}", path.display())
            }
            Self::ProjectDirectoryNotFound(path) => {
                write!(f, "project directory does not exist: {}", path.display())
            }
            Self::ProjectPathNotDirectory(path) => {
                write!(f, "project path is not a directory: {}", path.display())
            }
            Self::TargetAlreadyExists(path) => {
                write!(f, "target directory already exists: {}", path.display())
            }
            Self::Io(error) => write!(f, "{error}"),
            Self::GitInitFailed(message) => write!(f, "git init failed: {message}"),
            Self::GitCommitFailed(message) => write!(f, "initial git commit failed: {message}"),
            Self::CleanupFailed { original, cleanup } => {
                write!(
                    f,
                    "{original}; additionally failed to clean up the new project directory: {cleanup}"
                )
            }
        }
    }
}

impl Error for CodexHelperError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::CleanupFailed { cleanup, .. } => Some(cleanup),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CodexHelperError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub type Result<T> = std::result::Result<T, CodexHelperError>;

pub fn validate_project_name(name: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(CodexHelperError::InvalidProjectName(
            "name must not be empty".to_string(),
        ));
    }

    if trimmed != name {
        return Err(CodexHelperError::InvalidProjectName(
            "name must not start or end with whitespace".to_string(),
        ));
    }

    if matches!(trimmed, "." | "..") {
        return Err(CodexHelperError::InvalidProjectName(
            "name must not be '.' or '..'".to_string(),
        ));
    }

    if trimmed.ends_with('.') || trimmed.ends_with(' ') {
        return Err(CodexHelperError::InvalidProjectName(
            "name must not end with a dot or space".to_string(),
        ));
    }

    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err(CodexHelperError::InvalidProjectName(
            "name must not contain path separators".to_string(),
        ));
    }

    let path = Path::new(trimmed);
    if path.components().count() != 1 {
        return Err(CodexHelperError::InvalidProjectName(
            "name must be a single path component".to_string(),
        ));
    }

    if is_windows_reserved_name(trimmed) {
        return Err(CodexHelperError::InvalidProjectName(
            "name must not use a Windows reserved device name".to_string(),
        ));
    }

    Ok(())
}

fn is_windows_reserved_name(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name);
    matches!(
        stem.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

pub fn default_target_parent(current_dir: &Path) -> Result<PathBuf> {
    current_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| CodexHelperError::MissingParent(current_dir.to_path_buf()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectHealth {
    pub path: PathBuf,
    pub is_git_repo: bool,
    pub has_todo: bool,
    pub has_plans_guide: bool,
    pub has_readme: bool,
    pub has_gitignore: bool,
    pub has_agents: bool,
    pub has_environment: bool,
    pub management_docs_not_ignored: bool,
    pub todo_plan_links_valid: bool,
    pub dirty_worktree: Option<bool>,
}

impl ProjectHealth {
    pub fn is_configured(&self) -> bool {
        self.is_git_repo
            && self.has_todo
            && self.has_plans_guide
            && self.has_readme
            && self.has_gitignore
            && self.has_agents
            && self.has_environment
            && self.management_docs_not_ignored
            && self.todo_plan_links_valid
    }

    pub fn is_ready(&self) -> bool {
        self.is_configured()
    }

    pub fn meets_requirements(&self, require_clean: bool) -> bool {
        self.is_configured() && (!require_clean || self.dirty_worktree == Some(false))
    }
}

pub fn check_project_health(path: &Path) -> Result<ProjectHealth> {
    let path = path.to_path_buf();
    let is_git_repo = path.join(".git").exists();
    let dirty_worktree = if is_git_repo {
        Some(is_git_worktree_dirty(&path)?)
    } else {
        None
    };
    let management_docs_not_ignored = if is_git_repo {
        management_documents_are_not_ignored(&path)?
    } else {
        false
    };
    let todo_plan_links_valid = validate_todo_plan_links(&path)?;

    Ok(ProjectHealth {
        has_todo: path.join("TODO.md").is_file(),
        has_plans_guide: path.join("plans/README.md").is_file(),
        has_readme: path.join("README.md").is_file(),
        has_gitignore: path.join(".gitignore").is_file(),
        has_agents: path.join("AGENTS.md").is_file(),
        has_environment: path.join(".codex/environments/environment.toml").is_file(),
        management_docs_not_ignored,
        todo_plan_links_valid,
        path,
        is_git_repo,
        dirty_worktree,
    })
}

pub fn initialize_project(path: &Path) -> Result<InitializedProject> {
    if !path.exists() {
        return Err(CodexHelperError::ProjectDirectoryNotFound(
            path.to_path_buf(),
        ));
    }
    if !path.is_dir() {
        return Err(CodexHelperError::ProjectPathNotDirectory(
            path.to_path_buf(),
        ));
    }

    let path = path.canonicalize()?;
    let initialized_git = !path.join(".git").exists();
    if initialized_git {
        init_git_repo(&path, "git")?;
    }

    let project_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("Project");
    let mut created_files = Vec::new();

    write_if_missing(
        &path.join("TODO.md"),
        &templates::todo_contents(project_name),
        &mut created_files,
    )?;

    let plans_dir = path.join("plans");
    fs::create_dir_all(&plans_dir)?;
    write_if_missing(
        &plans_dir.join("README.md"),
        &templates::plans_readme_contents(project_name),
        &mut created_files,
    )?;
    write_if_missing(
        &path.join("README.md"),
        &templates::readme_contents(project_name),
        &mut created_files,
    )?;
    write_if_missing(
        &path.join("AGENTS.md"),
        &templates::agents_contents(project_name),
        &mut created_files,
    )?;

    let environment_dir = path.join(".codex/environments");
    fs::create_dir_all(&environment_dir)?;
    write_if_missing(
        &environment_dir.join("environment.toml"),
        templates::environment_contents(),
        &mut created_files,
    )?;

    let gitignore_path = path.join(".gitignore");
    let (gitignore_created, updated_gitignore) =
        ensure_management_documents_not_ignored(&gitignore_path)?;
    if gitignore_created {
        created_files.push(gitignore_path);
    }

    Ok(InitializedProject {
        path,
        created_files,
        initialized_git,
        updated_gitignore,
    })
}

fn write_if_missing(path: &Path, contents: &str, created_files: &mut Vec<PathBuf>) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    fs::write(path, contents)?;
    created_files.push(path.to_path_buf());
    Ok(())
}

const MANAGEMENT_DOCUMENTS: [&str; 4] = ["TODO.md", "README.md", "AGENTS.md", "plans/README.md"];

const MANAGEMENT_GITIGNORE_RULES: [&str; 5] = [
    "!/TODO.md",
    "!/README.md",
    "!/AGENTS.md",
    "!/plans/",
    "!/plans/**",
];

fn ensure_management_documents_not_ignored(path: &Path) -> Result<(bool, bool)> {
    if !path.exists() {
        fs::write(path, templates::gitignore_contents())?;
        return Ok((true, true));
    }

    let contents = fs::read_to_string(path)?;
    let existing_rules: HashSet<&str> = contents.lines().map(str::trim).collect();
    let missing_rules: Vec<&str> = MANAGEMENT_GITIGNORE_RULES
        .iter()
        .copied()
        .filter(|rule| !existing_rules.contains(rule))
        .collect();

    if missing_rules.is_empty() {
        return Ok((false, false));
    }

    let mut updated = contents;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str("\n# CodexHelper management documents\n");
    for rule in missing_rules {
        updated.push_str(rule);
        updated.push('\n');
    }
    fs::write(path, updated)?;
    Ok((false, true))
}

fn management_documents_are_not_ignored(project_dir: &Path) -> Result<bool> {
    for document in MANAGEMENT_DOCUMENTS {
        let output = Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index", "--", document])
            .current_dir(project_dir)
            .output()?;

        match output.status.code() {
            Some(0) => return Ok(false),
            Some(1) => {}
            _ => {
                return Err(CodexHelperError::Io(std::io::Error::other(
                    command_message(&output),
                )))
            }
        }
    }

    Ok(true)
}

fn validate_todo_plan_links(project_dir: &Path) -> Result<bool> {
    let todo_path = project_dir.join("TODO.md");
    let plans_dir = project_dir.join("plans");
    if !todo_path.is_file() || !plans_dir.join("README.md").is_file() {
        return Ok(false);
    }

    let todo = fs::read_to_string(todo_path)?;
    let mut task_ids = HashSet::new();
    let mut referenced_plans = HashSet::new();

    for line in todo.lines().filter(|line| is_checkbox_task(line)) {
        let ids = task_ids_in_line(line);
        if ids.len() != 1 {
            return Ok(false);
        }

        let task_id = &ids[0];
        if !task_ids.insert(task_id.clone()) {
            return Ok(false);
        }

        let Some(plan_link) = matching_plan_link(line, task_id) else {
            return Ok(false);
        };
        if !project_dir.join(&plan_link).is_file() {
            return Ok(false);
        }
        referenced_plans.insert(plan_link);
    }

    for entry in fs::read_dir(plans_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let file_name = entry.file_name();
        if file_name == "README.md"
            || Path::new(&file_name)
                .extension()
                .and_then(|ext| ext.to_str())
                != Some("md")
        {
            continue;
        }
        let relative_path = PathBuf::from("plans").join(file_name);
        if !referenced_plans.contains(&relative_path) {
            return Ok(false);
        }
    }

    Ok(true)
}

fn is_checkbox_task(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]")
}

fn task_ids_in_line(line: &str) -> Vec<String> {
    let bytes = line.as_bytes();
    let mut ids = Vec::new();
    let mut index = 0;

    while index + 5 <= bytes.len() {
        let candidate = &bytes[index..index + 5];
        let previous_is_identifier = index > 0 && bytes[index - 1].is_ascii_alphanumeric();
        let next_is_digit = index + 5 < bytes.len() && bytes[index + 5].is_ascii_digit();
        if candidate[0] == b'T'
            && candidate[1] == b'-'
            && candidate[2..].iter().all(u8::is_ascii_digit)
            && !previous_is_identifier
            && !next_is_digit
        {
            let task_id = String::from_utf8_lossy(candidate).into_owned();
            if !ids.contains(&task_id) {
                ids.push(task_id);
            }
            index += 5;
        } else {
            index += 1;
        }
    }

    ids
}

fn matching_plan_link(line: &str, task_id: &str) -> Option<PathBuf> {
    let mut remainder = line;
    while let Some(link_marker) = remainder.find("](plans/") {
        let path_start = link_marker + 2;
        let link_content = &remainder[path_start..];
        let path_end = link_content.find(')')?;
        let link = &link_content[..path_end];
        let path = Path::new(link);
        let valid_components = path
            .components()
            .all(|component| matches!(component, Component::Normal(_)));
        let file_name = path.file_name().and_then(|name| name.to_str());
        let valid_file_name = file_name.is_some_and(|name| {
            name == format!("{task_id}.md")
                || (name.starts_with(&format!("{task_id}-")) && name.ends_with(".md"))
        });
        if valid_components && path.parent() == Some(Path::new("plans")) && valid_file_name {
            return Some(path.to_path_buf());
        }
        remainder = &link_content[path_end + 1..];
    }

    None
}

pub fn create_project(options: CreateProjectOptions) -> Result<CreatedProject> {
    create_project_with_git(options, "git")
}

fn create_project_with_git(
    options: CreateProjectOptions,
    git_program: &str,
) -> Result<CreatedProject> {
    validate_project_name(&options.project_name)?;

    let parent = match &options.target_dir {
        Some(target_dir) => target_dir.to_path_buf(),
        None => default_target_parent(&options.current_dir)?,
    };
    let target = parent.join(&options.project_name);
    ensure_target_inside_parent(&parent, &target)?;

    if target.exists() {
        return Err(CodexHelperError::TargetAlreadyExists(target));
    }

    fs::create_dir(&target)?;
    if let Err(error) = write_project_files(&target, &options.project_name)
        .and_then(|_| init_git_repo(&target, git_program))
        .and_then(|_| {
            if options.initial_commit {
                create_initial_commit(&target, git_program)
            } else {
                Ok(())
            }
        })
    {
        return Err(cleanup_created_project(&target, error));
    }

    Ok(CreatedProject { path: target })
}

fn cleanup_created_project(target: &Path, error: CodexHelperError) -> CodexHelperError {
    match fs::remove_dir_all(target) {
        Ok(()) => error,
        Err(cleanup) => CodexHelperError::CleanupFailed {
            original: Box::new(error),
            cleanup,
        },
    }
}

fn ensure_target_inside_parent(parent: &Path, target: &Path) -> Result<()> {
    let canonical_parent = parent.canonicalize()?;
    let target_parent = target
        .parent()
        .ok_or_else(|| CodexHelperError::MissingParent(target.to_path_buf()))?
        .canonicalize()?;

    if target_parent != canonical_parent {
        return Err(CodexHelperError::InvalidProjectName(
            "target must remain inside the current directory's parent".to_string(),
        ));
    }

    Ok(())
}

fn write_project_files(project_dir: &Path, project_name: &str) -> Result<()> {
    fs::write(
        project_dir.join("TODO.md"),
        templates::todo_contents(project_name),
    )?;
    let plans_dir = project_dir.join("plans");
    fs::create_dir(&plans_dir)?;
    fs::write(
        plans_dir.join("README.md"),
        templates::plans_readme_contents(project_name),
    )?;
    fs::write(
        project_dir.join("README.md"),
        templates::readme_contents(project_name),
    )?;
    fs::write(
        project_dir.join(".gitignore"),
        templates::gitignore_contents(),
    )?;
    fs::write(
        project_dir.join("AGENTS.md"),
        templates::agents_contents(project_name),
    )?;

    let environment_dir = project_dir.join(".codex/environments");
    fs::create_dir_all(&environment_dir)?;
    fs::write(
        environment_dir.join("environment.toml"),
        templates::environment_contents(),
    )?;

    Ok(())
}

fn init_git_repo(project_dir: &Path, git_program: &str) -> Result<()> {
    let output = Command::new(git_program)
        .args(["init", "--initial-branch=main"])
        .current_dir(project_dir)
        .output()?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let message = if stderr.is_empty() { stdout } else { stderr };

    Err(CodexHelperError::GitInitFailed(message))
}

fn create_initial_commit(project_dir: &Path, git_program: &str) -> Result<()> {
    let add_output = Command::new(git_program)
        .arg("add")
        .arg(".")
        .current_dir(project_dir)
        .output()?;

    if !add_output.status.success() {
        return Err(CodexHelperError::GitCommitFailed(command_message(
            &add_output,
        )));
    }

    let has_user_name = git_config_value(project_dir, git_program, "user.name").is_some();
    let has_user_email = git_config_value(project_dir, git_program, "user.email").is_some();
    let mut commit_command = Command::new(git_program);
    if !has_user_name {
        commit_command.args(["-c", "user.name=CodexHelper"]);
    }
    if !has_user_email {
        commit_command.args(["-c", "user.email=codex-helper@example.invalid"]);
    }
    let commit_output = commit_command
        .args(["commit", "-m", "Initial commit"])
        .current_dir(project_dir)
        .output()?;

    if commit_output.status.success() {
        return Ok(());
    }

    Err(CodexHelperError::GitCommitFailed(command_message(
        &commit_output,
    )))
}

fn git_config_value(project_dir: &Path, git_program: &str, key: &str) -> Option<String> {
    let output = Command::new(git_program)
        .args(["config", "--get", key])
        .current_dir(project_dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn is_git_worktree_dirty(project_dir: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(project_dir)
        .output()?;

    if output.status.success() {
        return Ok(!output.stdout.is_empty());
    }

    Err(CodexHelperError::Io(std::io::Error::other(
        command_message(&output),
    )))
}

fn command_message(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stderr.is_empty() {
        stdout
    } else {
        stderr
    }
}

#[cfg(test)]
mod tests {
    use super::{create_project_with_git, CreateProjectOptions};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn removes_created_directory_when_git_init_cannot_start() {
        let workspace = tempdir().unwrap();
        let helper_dir = workspace.path().join("CodexHelper");
        fs::create_dir(&helper_dir).unwrap();

        let result = create_project_with_git(
            CreateProjectOptions {
                current_dir: helper_dir,
                project_name: "BrokenProject".to_string(),
                target_dir: None,
                initial_commit: false,
            },
            "codex-helper-missing-git-binary",
        );

        assert!(result.is_err());
        assert!(!workspace.path().join("BrokenProject").exists());
    }
}
