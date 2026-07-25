use std::process::ExitCode;

use clap::{Parser, Subcommand};
use codex_helper::{check_project_health, create_project, CreateProjectOptions, ProjectHealth};

#[derive(Debug, Parser)]
#[command(name = "codex-helper")]
#[command(about = "Local helper tools for Codex projects")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a new empty project next to the current directory.
    New {
        /// Name of the project directory to create.
        project_name: String,
        /// Directory where the project directory should be created.
        #[arg(long)]
        target_dir: Option<std::path::PathBuf>,
        /// Create an initial git commit after generating the project files.
        #[arg(long)]
        initial_commit: bool,
    },
    /// Check whether a project has the common CodexHelper setup.
    Check {
        /// Project directory to check. Defaults to the current directory.
        path: Option<std::path::PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::New {
            project_name,
            target_dir,
            initial_commit,
        } => {
            let current_dir = match std::env::current_dir() {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("Error: failed to read current directory: {error}");
                    return ExitCode::FAILURE;
                }
            };

            create_project(CreateProjectOptions {
                current_dir,
                project_name,
                target_dir,
                initial_commit,
            })
            .map(|created| {
                println!("Created project at {}", created.path.display());
            })
        }
        Commands::Check { path } => {
            let project_dir = match path {
                Some(path) => path,
                None => match std::env::current_dir() {
                    Ok(path) => path,
                    Err(error) => {
                        eprintln!("Error: failed to read current directory: {error}");
                        return ExitCode::FAILURE;
                    }
                },
            };

            check_project_health(&project_dir).map(print_health)
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn print_health(health: ProjectHealth) {
    println!("Project: {}", health.path.display());
    println!("git repository: {}", status(health.is_git_repo));
    println!("README.md: {}", status(health.has_readme));
    println!(".gitignore: {}", status(health.has_gitignore));
    println!("AGENTS.md: {}", status(health.has_agents));
    println!(
        ".codex/environments/environment.toml: {}",
        status(health.has_environment)
    );
    match health.dirty_worktree {
        Some(true) => println!("working tree clean: no"),
        Some(false) => println!("working tree clean: yes"),
        None => println!("working tree clean: n/a"),
    }
    println!("ready: {}", status(health.is_ready()));
}

fn status(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}
