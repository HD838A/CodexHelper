use std::process::ExitCode;

use clap::{Parser, Subcommand};
use codex_helper::{create_project, CreateProjectOptions};

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
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::New { project_name } => {
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
            })
        }
    };

    match result {
        Ok(created) => {
            println!("Created project at {}", created.path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}
