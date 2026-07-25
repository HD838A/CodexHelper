# CodexHelper

CodexHelper is a local CLI for preparing small Codex-friendly project shells.

## Install

```bash
cargo install --path .
```

This installs `codex-helper` into Cargo's binary directory. Make sure that directory is on `PATH`.

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

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
python3 /path/to/.codex/skills/.system/skill-creator/scripts/quick_validate.py skills/codexhelper-create-project
```
