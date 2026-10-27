fn render(template: &str, project_name: &str) -> String {
    template.replace("{{project_name}}", project_name)
}

pub(crate) fn readme_contents(project_name: &str) -> String {
    render(include_str!("../templates/project-README.md"), project_name)
}

pub(crate) fn todo_contents(project_name: &str) -> String {
    render(include_str!("../templates/TODO.md"), project_name)
}

pub(crate) fn plans_readme_contents(project_name: &str) -> String {
    render(include_str!("../templates/plans-README.md"), project_name)
}

pub(crate) fn gitignore_contents() -> &'static str {
    include_str!("../templates/gitignore")
}

pub(crate) fn agents_contents(project_name: &str) -> String {
    render(include_str!("../templates/AGENTS.md"), project_name)
}

pub(crate) fn environment_contents() -> &'static str {
    include_str!("../templates/environment.toml")
}
