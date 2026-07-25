pub(crate) fn readme_contents(project_name: &str) -> String {
    format!("# {project_name}\n\nProject description goes here.\n")
}

pub(crate) fn gitignore_contents() -> &'static str {
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

pub(crate) fn agents_contents(project_name: &str) -> String {
    format!(
        "# AGENTS.md instructions for {project_name}\n\n\
         - Read the existing files before making changes.\n\
         - Keep edits focused on the user's request.\n\
         - Preserve user changes and do not revert unrelated work.\n\
         - Avoid destructive commands unless the user explicitly asks for them.\n\
         - Verify the result when practical, and report anything that could not be verified.\n\
         - The `.codex/environments/environment.toml` file is a placeholder. After the project run workflow is known, update it to the correct state.\n"
    )
}

pub(crate) fn environment_contents() -> &'static str {
    "# Codex environment configuration placeholder.\n\
     # Update this file after the project run workflow is known.\n"
}
