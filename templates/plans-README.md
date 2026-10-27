# {{project_name}} Requirement Plans

Create one plan file for each concrete requirement. Do not use one large plan for the whole project.

## Naming and Linking

Use the task ID from `TODO.md` followed by a short description:

`T-001-short-description.md`

Every checkbox task in `TODO.md` must have one unique `T-xxx` ID and link its matching plan:

```markdown
- [ ] [T-001 Requirement title](plans/T-001-requirement-title.md)
```

When the requirement is complete, keep the plan file in place and move the linked task to `Done` in `TODO.md` so links remain stable.

## Plan Template

```markdown
# T-001 Requirement title

## Objective

Describe the result this requirement must achieve.

## Scope

- Components that will change

## Non-goals

- Work explicitly excluded from this requirement

## Steps

- [ ] Implement the requirement
- [ ] Verify the result
- [ ] Update relevant documentation

## Verification

- Commands and acceptance criteria

## Risks

- Known risks and compatibility impact
```
