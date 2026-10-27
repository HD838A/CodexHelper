# CodexHelper

CodexHelper is a local CLI for preparing small Codex-friendly project shells.

## Requirements

- Rust 1.85 or later (for building from source)
- Git 2.28 or later available on `PATH`

## Install

Install from a local checkout:

```bash
cargo install --path .
```

This installs `codex-helper` into Cargo's binary directory. Make sure that directory is on `PATH`.

To update a local installation after pulling changes:

```bash
cargo install --path . --force
```

To remove it:

```bash
cargo uninstall codex-helper
```

Install the bundled skill for use from any Codex project:

```bash
mkdir -p ~/.codex/skills
cp -R skills/codexhelper-create-project ~/.codex/skills/
```

When project creation is requested through the bundled skill, it first searches for actively maintained, license-compatible open-source projects that closely match the requested functionality. If a strong match exists, the skill presents the candidates and waits for confirmation before creating anything. If the request does not describe the intended functionality, the skill asks for that context first. Users can explicitly ask to skip this preflight check.

This research gate belongs to the agent skill because it needs the full natural-language requirement and network tools. Calling the `codex-helper` CLI directly remains deterministic and does not access the network.

## Usage

Create a sibling project next to the current directory:

```bash
codex-helper new PhotoTools
```

By default, this creates `../PhotoTools` relative to the current directory. The command never merges into or overwrites an existing directory.
New repositories use `main` as their initial branch.

Create inside an explicit workspace:

```bash
codex-helper new PhotoTools --target-dir /path/to/MySrc
```

Create and commit the generated project shell:

```bash
codex-helper new PhotoTools --initial-commit
```

The new project includes:

- `TODO.md`
- `plans/README.md` with a per-requirement plan template
- `README.md`
- `.gitignore`
- `AGENTS.md`
- `.codex/environments/environment.toml`
- an initialized git repository

`TODO.md`, `README.md`, `AGENTS.md`, and the `plans/` directory are explicitly excluded from generated `.gitignore` rules, so Git can track them. The initial TODO contains no placeholder task. When a concrete requirement exists, add one checkbox with a unique `T-xxx` ID, create its separate `plans/T-xxx-description.md` file, and link the task to that plan.

Initialize an existing project without overwriting files that are already present:

```bash
codex-helper init /path/to/existing-project
```

`init` creates only missing management files, initializes Git with the `main` branch when needed, and appends the required management-document exceptions to an existing `.gitignore`. Running it again is safe and makes no duplicate changes.

Check whether a project has the common CodexHelper setup:

```bash
codex-helper check /path/to/MySrc/PhotoTools
```

`check` validates the required files, Git ignore rules, TODO-to-plan links, and repository state. A project can be configured and ready while its working tree contains normal development changes. Require a clean worktree explicitly when needed:

```bash
codex-helper check /path/to/project --require-clean
```

Use stable JSON output for scripts and agents:

```bash
codex-helper check /path/to/project --format json
```

The command exits with code `0` when the selected requirements pass, `2` when the project is not ready, and `1` for operational errors.

## Local operation and privacy

CodexHelper does not send project files or telemetry over the network. It writes files only to the requested project directory and invokes the local `git` executable to initialize repositories, inspect working-tree status, and optionally create an initial commit.

## Compatibility

CodexHelper is tested on macOS and Linux. Git worktrees are supported by the `check` command.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The same Rust checks run in GitHub Actions for pushes and pull requests.
CI runs on both Linux and macOS. Dependabot monitors Cargo and GitHub Actions dependencies, and the audit job checks Rust dependencies for known security advisories.

Project file templates live under `templates/`, making Markdown workflow changes reviewable without editing Rust string literals.

## License

CodexHelper is licensed under the [MIT License](LICENSE).
