# CodexHelper

CodexHelper is a local CLI for preparing small Codex-friendly project shells.

## Install

```bash
cargo install --path .
```

This installs `codex-helper` into Cargo's binary directory. Make sure that directory is on `PATH`.

## Usage

Create a sibling project next to the current directory:

```bash
codex-helper new PhotoTools
```

The new project includes:

- `README.md`
- `.gitignore`
- `AGENTS.md`
- `.codex/environments/environment.toml`
- an initialized git repository

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
python3 /path/to/.codex/skills/.system/skill-creator/scripts/quick_validate.py skills/codexhelper-create-project
```
