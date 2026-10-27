---
name: codexhelper-create-project
description: Research matching open-source alternatives before creating a new empty project with CodexHelper, pause for confirmation when a strong reusable match exists, and otherwise apply the common project setup. Use when the user asks to create, initialize, scaffold, or prepare a new project by name, including requests like "create a new project named X", "new project X", "initialize X with basic config", "新建一个叫 X 的项目", or "初始化 X 项目". Do not use for implementing business features inside the new project.
---

# CodexHelper Create Project

## Overview

Use this skill to evaluate open-source reuse before turning a natural-language project request into a CodexHelper CLI call. Create only an empty project shell with common files and git initialized.

## Workflow

1. Extract the project name and requested functionality from the user's request.
2. If the project name is missing or ambiguous, ask for it. If there is not enough functional context to search for alternatives, ask for a one-sentence description of the project's purpose. Do not create the project yet.
3. Unless the user explicitly asks to skip research, search GitHub and other relevant open-source registries for projects that address the requested functionality.
4. Treat a candidate as a strong match only when its core functionality and target platform or constraints align closely, its license permits the intended use, and it is not archived or evidently abandoned.
5. If one or more strong matches exist, report the best candidates with their links, matching reasons, license, and maintenance status. Ask whether to reuse, fork, integrate, or still create a new project. Stop before running CodexHelper until the user confirms.
6. If no strong match exists, continue without requiring another confirmation. If research cannot be completed, explain the limitation and ask whether to proceed.
7. Do not implement product features, app code, UI, backend services, or language-specific scaffolding as part of this skill. If the user asks for creation plus development work, create only the project shell and report that feature work is outside this skill.
8. Run:

```bash
codex-helper new <project-name>
```

9. If the user gives an explicit destination directory, pass it with `--target-dir <path>`.
10. If the user asks for the generated shell to be committed, pass `--initial-commit`.
11. If the CLI is not installed but the current workspace is the CodexHelper repository, run the local binary through the Rust project instead:

```bash
cargo run -- new <project-name>
```

12. Report the created project path and the generated files: `.gitignore`, `TODO.md`, `plans/README.md`, `README.md`, `AGENTS.md`, and `.codex/environments/environment.toml`. Explain that the initial TODO is empty, concrete requirements get unique `T-xxx` IDs and separate linked `plans/T-xxx-description.md` files, these management documents are explicitly kept out of Git ignore rules, and the Git repository starts on the `main` branch.

## Boundaries

- Do not bypass the CLI by manually creating the project files.
- Do not create a project before completing the open-source check, unless the user explicitly skips it.
- Do not treat a keyword match, inactive repository, or incompatible license as a strong reusable match.
- Do not overwrite or merge into an existing directory.
- Do not install this skill globally unless the user explicitly asks.
- Keep the response focused on creation status and any CLI errors.

## Examples

User: "帮我创建一个叫 PhotoTools 的新项目。"

Action: ask what PhotoTools should do, because the name alone is not enough for an open-source search.

User: "新建 TaskRunner，并加一个同步功能。"

Action: search for actively maintained, license-compatible task synchronization projects. If a strong match exists, present it and wait for confirmation. Otherwise run `codex-helper new TaskRunner`, then explain that implementation of the sync feature should be handled as a separate development task.

User: "不要查开源库，直接创建 TaskRunner。"

Action: run `codex-helper new TaskRunner` because the user explicitly skipped the default research step.
