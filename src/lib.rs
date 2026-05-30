use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct CreateProjectOptions {
    pub current_dir: PathBuf,
    pub project_name: String,
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
        }
    }
}

impl Error for CodexHelperError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
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

pub fn create_project(options: CreateProjectOptions) -> Result<CreatedProject> {
    validate_project_name(&options.project_name)?;

    let parent = default_target_parent(&options.current_dir)?;
    let target = parent.join(&options.project_name);
    ensure_target_inside_parent(&parent, &target)?;

    if target.exists() {
        return Err(CodexHelperError::TargetAlreadyExists(target));
    }

    fs::create_dir(&target)?;
    write_project_files(&target, &options.project_name)?;
    init_git_repo(&target)?;

    Ok(CreatedProject { path: target })
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
    fs::write(project_dir.join("README.md"), readme_contents(project_name))?;
    fs::write(project_dir.join(".gitignore"), gitignore_contents())?;
    fs::write(project_dir.join("AGENTS.md"), agents_contents(project_name))?;
    Ok(())
}

fn init_git_repo(project_dir: &Path) -> Result<()> {
    let output = Command::new("git")
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

fn readme_contents(project_name: &str) -> String {
    format!("# {project_name}\n\nProject description goes here.\n")
}

fn gitignore_contents() -> &'static str {
    ".DS_Store\n\
     .idea/\n\
     .vscode/\n\
     *.log\n\
     .env\n\
     .env.*\n\
     !.env.example\n\
     tmp/\n\
     temp/\n\
     cache/\n\
     .cache/\n\
     build/\n\
     dist/\n\
     target/\n"
}

fn agents_contents(project_name: &str) -> String {
    format!(
        "# AGENTS.md instructions for {project_name}\n\n\
         - Read the existing files before making changes.\n\
         - Keep edits focused on the user's request.\n\
         - Preserve user changes and do not revert unrelated work.\n\
         - Avoid destructive commands unless the user explicitly asks for them.\n\
         - Verify the result when practical, and report anything that could not be verified.\n"
    )
}
