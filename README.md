# CodexHelper

CodexHelper is a local CLI for preparing small Codex-friendly project shells.

## Requirements

- Rust 1.85 or later (for building from source)
- Git available on `PATH`

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

## Usage

Create a sibling project next to the current directory:

```bash
codex-helper new PhotoTools
```

By default, this creates `../PhotoTools` relative to the current directory. The command never merges into or overwrites an existing directory.

Create inside an explicit workspace:

```bash
codex-helper new PhotoTools --target-dir /path/to/MySrc
```

Create and commit the generated project shell:

```bash
codex-helper new PhotoTools --initial-commit
```

The new project includes:

- `README.md`
- `.gitignore`
- `AGENTS.md`
- `.codex/environments/environment.toml`
- an initialized git repository

Check whether a project has the common CodexHelper setup:

```bash
codex-helper check /path/to/MySrc/PhotoTools
```

`check` reports each required file, whether the directory is a Git repository, whether its working tree is clean, and a final `ready: yes` or `ready: no` summary.

## Local operation and privacy

CodexHelper does not send project files or telemetry over the network. It writes files only to the requested project directory and invokes the local `git` executable to initialize repositories, inspect working-tree status, and optionally create an initial commit.

## Compatibility

Project names are validated for portability across macOS, Linux, and Windows. Git worktrees are supported by the `check` command.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The same Rust checks run in GitHub Actions for pushes and pull requests.

## License

CodexHelper is licensed under the [MIT License](LICENSE).
