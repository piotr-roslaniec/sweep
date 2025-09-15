---
description: Communication and workflow guidelines for Agent OS
globs:
alwaysApply: false
version: 1.0
encoding: UTF-8
---

# Communication and Workflow Guidelines

## Language Standards

<language_principles>
### Factual Communication
- Present information directly without qualifiers
- Avoid sycophantic language: "best", "highest quality", "most correct", "excellent", "perfect"
- Avoid comparative language: "improved", "enhanced", "better", "new version", "upgraded"
- State facts as they are: "This implements X" not "This implements an improved X"

### Examples
<incorrect_examples>
❌ "Creating the best possible solution"
❌ "Implementing enhanced error handling"
❌ "Building a new and improved version"
❌ "Ensuring the highest quality output"
</incorrect_examples>

<correct_examples>
✅ "Creating the solution"
✅ "Implementing error handling"
✅ "Building this version"
✅ "Generating the output"
</correct_examples>
</language_principles>

## Git Workflow Standards

<git_workflow>
### Commit Practices
- Use conventional commits format: type(scope): description
- Commit frequently for large changes
- Keep commits atomic and focused

### GitHub CLI Integration
Always use gh CLI for repository operations:

<gh_commands>
# Check PR status
gh pr status

# Create pull request
gh pr create --title "feat: Add authentication" --body "Implements user authentication"

# Review PR checks
gh pr checks

# View PR details
gh pr view

# List issues
gh issue list

# Check workflow runs
gh run list
</gh_commands>

### Pull Request Workflow
1. Create feature branch for changes
2. Make commits following conventional format
3. Push branch to remote
4. Create PR using gh CLI
5. Monitor PR status and checks
6. Address review feedback
7. Merge when approved

### Conventional Commit Types
- feat: A new feature
- fix: A bug fix
- docs: Documentation changes
- style: Code style changes
- refactor: Code refactoring
- perf: Performance improvements
- test: Test additions or corrections
- build: Build system changes
- ci: CI configuration changes
- chore: Maintenance tasks

### Example Workflow
<example>
# Create feature branch
git checkout -b feat/user-authentication

# Make changes and commit
git add src/auth.js
git commit -m "feat(auth): add JWT token validation"

# Push and create PR
git push -u origin feat/user-authentication
gh pr create --title "feat: Add user authentication" \
  --body "Implements JWT-based authentication system"

# Check PR status
gh pr status
gh pr checks

# After approval, merge
gh pr merge --squash
</example>
</git_workflow>

## Container-Use Integration

<container_awareness>
When container-use is detected in session:
- Work exclusively in isolated environment
- Never use git CLI directly in containers
- Always provide env_id for user review
- Guide user through checkout process before git operations

Container communication pattern:
```
Development complete in environment: <env_id>
Review: container-use diff <env_id>
Apply: container-use checkout <env_id>
Then proceed with gh pr create
```
</container_awareness>

## Integration with Other Instructions

These guidelines apply across all Agent OS operations:
- When creating tasks.md: Use factual descriptions
- When writing commit messages: Follow conventional format
- When reporting status: State facts without embellishment
- When using git: Always leverage gh CLI capabilities
- When in containers: Follow container-use workflow first