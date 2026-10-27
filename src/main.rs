use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use codex_helper::{
    check_project_health, create_project, initialize_project, CreateProjectOptions, ProjectHealth,
    Result,
};
use serde_json::json;

const HEALTH_CHECK_FAILED_EXIT_CODE: u8 = 2;

#[derive(Debug, Parser)]
#[command(name = "codex-helper")]
#[command(about = "Local helper tools for Codex projects")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a new empty project next to the current directory.
    New {
        /// Name of the project directory to create.
        project_name: String,
        /// Directory where the project directory should be created.
        #[arg(long)]
        target_dir: Option<PathBuf>,
        /// Create an initial git commit after generating the project files.
        #[arg(long)]
        initial_commit: bool,
    },
    /// Add missing CodexHelper management files to an existing project.
    Init {
        /// Existing project directory to initialize. Defaults to the current directory.
        path: Option<PathBuf>,
    },
    /// Check whether a project has the common CodexHelper setup.
    Check {
        /// Project directory to check. Defaults to the current directory.
        path: Option<PathBuf>,
        /// Also fail when the Git working tree has uncommitted changes.
        #[arg(long)]
        require_clean: bool,
        /// Output format for humans or automation.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(HEALTH_CHECK_FAILED_EXIT_CODE),
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<bool> {
    match cli.command {
        Commands::New {
            project_name,
            target_dir,
            initial_commit,
        } => {
            let current_dir = std::env::current_dir()?;
            let created = create_project(CreateProjectOptions {
                current_dir,
                project_name,
                target_dir,
                initial_commit,
            })?;
            println!("Created project at {}", created.path.display());
            Ok(true)
        }
        Commands::Init { path } => {
            let project_dir = path.unwrap_or(std::env::current_dir()?);
            let initialized = initialize_project(&project_dir)?;
            println!("Initialized project at {}", initialized.path.display());
            if initialized.initialized_git {
                println!("Initialized Git repository with default branch main");
            }
            for created_file in initialized.created_files {
                println!("Created {}", created_file.display());
            }
            if initialized.updated_gitignore {
                println!("Updated .gitignore management-document rules");
            }
            Ok(true)
        }
        Commands::Check {
            path,
            require_clean,
            format,
        } => {
            let project_dir = path.unwrap_or(std::env::current_dir()?);
            let health = check_project_health(&project_dir)?;
            match format {
                OutputFormat::Text => print_health_text(&health, require_clean),
                OutputFormat::Json => print_health_json(&health, require_clean),
            }
            Ok(health.meets_requirements(require_clean))
        }
    }
}

fn print_health_text(health: &ProjectHealth, require_clean: bool) {
    println!("Project: {}", health.path.display());
    println!("git repository: {}", status(health.is_git_repo));
    println!("TODO.md: {}", status(health.has_todo));
    println!("plans/README.md: {}", status(health.has_plans_guide));
    println!("README.md: {}", status(health.has_readme));
    println!(".gitignore: {}", status(health.has_gitignore));
    println!("AGENTS.md: {}", status(health.has_agents));
    println!(
        ".codex/environments/environment.toml: {}",
        status(health.has_environment)
    );
    println!(
        "management documents not ignored: {}",
        status(health.management_docs_not_ignored)
    );
    println!(
        "TODO plan links valid: {}",
        status(health.todo_plan_links_valid)
    );
    match health.dirty_worktree {
        Some(true) => println!("working tree clean: no"),
        Some(false) => println!("working tree clean: yes"),
        None => println!("working tree clean: n/a"),
    }
    println!("configured: {}", status(health.is_configured()));
    println!("require clean: {}", status(require_clean));
    println!(
        "ready: {}",
        status(health.meets_requirements(require_clean))
    );
}

fn print_health_json(health: &ProjectHealth, require_clean: bool) {
    let output = json!({
        "schema_version": 1,
        "path": health.path.display().to_string(),
        "checks": {
            "git_repository": health.is_git_repo,
            "todo": health.has_todo,
            "plans_guide": health.has_plans_guide,
            "readme": health.has_readme,
            "gitignore": health.has_gitignore,
            "agents": health.has_agents,
            "environment": health.has_environment,
            "management_documents_not_ignored": health.management_docs_not_ignored,
            "todo_plan_links_valid": health.todo_plan_links_valid,
            "working_tree_clean": health.dirty_worktree.map(|dirty| !dirty),
        },
        "configured": health.is_configured(),
        "require_clean": require_clean,
        "ready": health.meets_requirements(require_clean),
    });
    println!("{output}");
}

fn status(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}
