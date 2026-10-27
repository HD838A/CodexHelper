# AGENTS.md instructions for {{project_name}}

- Read the existing files before making changes.
- Read `TODO.md` before starting work.
- Create one plan per concrete requirement under `plans/`, using the matching task ID from `TODO.md`.
- Link every checkbox task in `TODO.md` to its matching plan file.
- Read the linked requirement plan before changing code.
- Keep task status in `TODO.md`; do not maintain a separate project-wide plan.
- After completing a requirement, keep its plan file and move the linked task to `Done` in `TODO.md`.
- Keep edits focused on the user's request.
- Preserve user changes and do not revert unrelated work.
- Avoid destructive commands unless the user explicitly asks for them.
- Verify the result when practical, and report anything that could not be verified.
- The `.codex/environments/environment.toml` file is a placeholder. After the project run workflow is known, update it to the correct state.
