---
name: codexhelper-create-project
description: Create a new empty project with CodexHelper and apply basic common project setup. Use when the user asks to create, initialize, scaffold, or prepare a new project by name, including requests like "create a new project named X", "new project X", "initialize X with basic config", "新建一个叫 X 的项目", or "初始化 X 项目". Do not use for implementing business features inside the new project.
---

# CodexHelper Create Project

## Overview

Use this skill to turn a natural-language project creation request into a single CodexHelper CLI call. The goal is only to create an empty project shell with common files and git initialized.

## Workflow

1. Extract the project name from the user's request.
2. If the project name is missing or ambiguous, ask only for the project name.
3. Do not implement product features, app code, UI, backend services, or language-specific scaffolding as part of this skill. If the user asks for creation plus development work, create only the project shell and report that feature work is outside this skill.
4. Run:

```bash
codex-helper new <project-name>
```

5. If the CLI is not installed but the current workspace is the CodexHelper repository, run the local binary through the Rust project instead:

```bash
cargo run -- new <project-name>
```

6. Report the created project path and the generated files: `.gitignore`, `README.md`, `AGENTS.md`, and `.codex/environments/environment.toml`.

## Boundaries

- Do not bypass the CLI by manually creating the project files.
- Do not overwrite or merge into an existing directory.
- Do not install this skill globally unless the user explicitly asks.
- Keep the response focused on creation status and any CLI errors.

## Examples

User: "帮我创建一个叫 PhotoTools 的新项目。"

Action: run `codex-helper new PhotoTools`.

User: "新建 TaskRunner，并加一个同步功能。"

Action: run `codex-helper new TaskRunner`, then explain that implementation of the sync feature should be handled as a separate development task.
