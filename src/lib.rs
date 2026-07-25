use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
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

#[derive(Debug)]
pub enum CodexHelperError {
    InvalidProjectName(String),
    MissingParent(PathBuf),
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

    Ok(())
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
    pub has_readme: bool,
    pub has_gitignore: bool,
    pub has_agents: bool,
    pub has_environment: bool,
    pub dirty_worktree: Option<bool>,
}

impl ProjectHealth {
    pub fn is_ready(&self) -> bool {
        self.is_git_repo
            && self.has_readme
            && self.has_gitignore
            && self.has_agents
            && self.has_environment
            && self.dirty_worktree == Some(false)
    }
}

pub fn check_project_health(path: &Path) -> Result<ProjectHealth> {
    let path = path.to_path_buf();
    let is_git_repo = path.join(".git").is_dir();
    let dirty_worktree = if is_git_repo {
        Some(is_git_worktree_dirty(&path)?)
    } else {
        None
    };

    Ok(ProjectHealth {
        has_readme: path.join("README.md").is_file(),
        has_gitignore: path.join(".gitignore").is_file(),
        has_agents: path.join("AGENTS.md").is_file(),
        has_environment: path.join(".codex/environments/environment.toml").is_file(),
        path,
        is_git_repo,
        dirty_worktree,
    })
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
        .arg("init")
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

    let commit_output = Command::new(git_program)
        .arg("-c")
        .arg("user.name=CodexHelper")
        .arg("-c")
        .arg("user.email=codex-helper@example.invalid")
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .current_dir(project_dir)
        .output()?;

    if commit_output.status.success() {
        return Ok(());
    }

    Err(CodexHelperError::GitCommitFailed(command_message(
        &commit_output,
    )))
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
