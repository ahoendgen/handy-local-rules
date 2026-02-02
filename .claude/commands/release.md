---
description: Update CHANGELOG with git commits, bump version, and create release
argument-hint: [version-type]
allowed-tools: Read, Edit, Bash, Glob, AskUserQuestion
---

# Release handy-local-rules

Release a new version with automated changelog generation from git commits.

## Context

First, locate the project configuration:

@Cargo.toml @package.json @CHANGELOG.md

## Your Task

Follow this workflow to release:

### 1. Determine Version Type

- Parse argument: $ARGUMENTS (patch/minor/major, defaults to patch)
- Read current version from Cargo.toml
- Calculate new version based on semver rules

### 2. Check for Uncommitted Changes

```bash
git status --porcelain
```

- If there are uncommitted changes, warn the user and ask if they want to continue

### 3. Generate Changelog from Git Commits

```bash
# Get last release commit hash from CHANGELOG.md
# Format: ## [0.1.0] - 2026-02-02
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

# Get commits since last tag (or all commits if no tag)
if [ -n "$LAST_TAG" ]; then
  git --no-pager log ${LAST_TAG}..HEAD --oneline
else
  git --no-pager log --oneline -20
fi
```

- Parse conventional commits (feat, fix, docs, chore, refactor, etc.)
- Group commits by type
- **CRITICAL**: SKIP any commits starting with `chore(release):` to avoid circular references

### 4. Update CHANGELOG.md

- Add new version entry at the TOP (do NOT modify existing entries)
- Use today's date (YYYY-MM-DD format)
- Group changes by type:
  - **Features** (feat:)
  - **Bug Fixes** (fix:)
  - **Documentation** (docs:)
  - **Refactoring** (refactor:)
  - **Rules** (for rule changes)
- Format: `## [version] - YYYY-MM-DD`

### 5. Update Version in Cargo.toml and package.json

- Update the `version = "x.y.z"` line in Cargo.toml
- Update the `"version": "x.y.z"` line in package.json
- Both versions must match

### 6. Show Preview and Confirm

- Display the changelog diff
- **Use AskUserQuestion** to ask:
  - Question: "Does this changelog look correct?"
  - Header: "Confirm"
  - Options:
    1. "Yes, create release" - Proceed with commit and tag
    2. "Edit changelog" - Let me modify the changelog first
    3. "Cancel" - Abort the release

### 7. Commit the Release

```bash
git add CHANGELOG.md Cargo.toml package.json
git commit -m "chore(release): v{version}"
```

### 8. Create Git Tag

```bash
git tag -a v{version} -m "Release {version}"
```

### 9. Push to Remote

- **Use AskUserQuestion** to ask:
  - Question: "Push to origin? This will trigger the GitHub release pipeline."
  - Header: "Push"
  - Options:
    1. "Yes, push" - Push commit and tag
    2. "No, I'll push later" - Keep changes local

```bash
git push origin main --tags
```

### 10. Confirm Completion

- Show the new version number
- Display the git tag
- Confirm if pushed to remote
- Remind that GitHub Actions will create the release with binaries

## Changelog Format Example

```markdown
## [0.2.0] - 2026-02-03

### Features

- Add clipboard operations for text manipulation
- Implement new rule type

### Bug Fixes

- Fix whitespace handling in cleanup rules
- Correct priority ordering

### Rules

- Add German punctuation rules
- Improve cleanup patterns
```

## Important Notes

- **NEVER modify already-versioned CHANGELOG entries** - only add new entries
- **CHECK for duplicate entries** - review existing entries to avoid duplicates
- Follow semantic versioning (patch for fixes, minor for features, major for breaking)
- The GitHub release workflow will automatically extract the changelog and attach binaries
