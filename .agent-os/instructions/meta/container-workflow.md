---
description: Container-use integration with GitHub workflow
globs:
alwaysApply: false
version: 1.0
encoding: UTF-8
---

# Container-Use and GitHub Workflow Integration

## Overview

Container-use provides isolated environments for development work. When container-use is active in a session, follow these guidelines to maintain workflow integrity.

## Container-Use Detection

<conditional-block context-check="container-use-active">
IF container-use environment tools are available:
  APPLY: Container workflow rules below
  USE: Environment tools for ALL operations
  AVOID: Direct git CLI in environments
  INFORM: User about env_id for review
ELSE:
  SKIP: Container-specific workflow
  USE: Standard git workflow directly
</conditional-block>

## Workflow Integration

### When Container-Use is Active

<container_workflow>
1. **Development Phase**
   - Work in isolated environment
   - Make all changes using environment tools
   - Test thoroughly in container

2. **Review Phase**
   - User reviews with: `container-use diff <env_id>`
   - User logs activity: `container-use log <env_id>`
   - User decides on changes

3. **Integration Phase**
   - User applies changes: `container-use checkout <env_id>`
   - Changes merge to local filesystem
   - Continue with standard gh workflow
</container_workflow>

### Post-Container GitHub Workflow

<github_integration>
After container changes are applied to local:

1. **Verify Changes**
   ```bash
   git status
   git diff
   ```

2. **Create Feature Branch**
   ```bash
   git checkout -b feat/container-changes
   ```

3. **Commit with Conventional Format**
   ```bash
   git add .
   git commit -m "feat: implement changes from container environment"
   ```

4. **Create PR with gh CLI**
   ```bash
   gh pr create --title "feat: container-developed feature" \
     --body "Changes developed in isolated container environment"
   ```

5. **Monitor PR**
   ```bash
   gh pr status
   gh pr checks
   ```
</github_integration>

## Key Principles

<principles>
### Container-Use Active
- ALWAYS use environment tools for file operations
- NEVER modify .git directory directly
- ALWAYS inform user of env_id for review

### Container-Use Inactive
- Use standard file operations
- Follow gh CLI workflow directly
- Maintain conventional commit format
</principles>

## Communication Template

<communication>
When working in container:
```
Working in isolated environment: <env_id>

To review changes:
- View diff: container-use diff <env_id>
- See logs: container-use log <env_id>
- Apply changes: container-use checkout <env_id>
```

After container work:
```
Container work complete. Apply changes with:
container-use checkout <env_id>

Then create PR using:
gh pr create --title "feat: [description]"
```
</communication>

## Example Workflow

<example>
1. User requests feature development
2. Agent works in container environment
3. Agent completes work, provides env_id
4. User reviews: `container-use diff abc123`
5. User applies: `container-use checkout abc123`
6. Agent/User creates PR: `gh pr create`
7. Monitor with: `gh pr status`
</example>

## Integration with Agent-OS

This workflow complements existing Agent-OS patterns:
- Maintains isolation during development
- Preserves git history integrity
- Enables parallel development
- Supports conventional commits
- Leverages gh CLI for PR management