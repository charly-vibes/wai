# Repository Best Practices

## Purpose

Define repository structure and tooling standards that wai recommends and validates to help users adopt modern development practices.

## Problem Statement

Software repositories benefit from standardized tooling and conventions, but developers often don't know which tools to adopt or how to configure them. Without guidance, projects miss out on improved developer experience, code quality, and onboarding efficiency. This spec defines wai's opinionated recommendations based on 2026 industry best practices.

## Design Rationale

Wai provides *guidance*, not enforcement. All repository best practice checks use a dedicated status model (`WayWayStatus::Pass` or `WayWayStatus::Info`), allowing users to make informed decisions about which practices fit their context. This approach:
- Educates users about proven patterns without being prescriptive
- Reduces cognitive load by surfacing actionable recommendations
- Supports diverse project types and language ecosystems
- Creates a foundation for future automation (scaffolding commands)
- Uses separate status enum from `wai doctor` for clean separation of concerns

**Note**: Based on 2026 industry best practices. Review recommendations annually as tooling evolves.

## Scope and Requirements

This spec defines recommended repository standards that wai validates via `wai way`.

### Status Model

The `wai way` command uses a dedicated `WayStatus` enum (separate from `wai doctor`'s `Status` enum):

```rust
enum WayStatus {
    Pass,  // ✓ green - Practice adopted
    Info,  // ℹ blue - Recommendation/suggestion
}
```

This separation ensures `wai way` output is always informational and never fails (exit code 0), while `wai doctor` can report failures for wai-specific issues.

### Non-Goals
- Language-specific linting or formatting rules (out of scope)
- Enforcing or requiring any specific practices (all voluntary)
- Deep validation of tool configurations (file existence checks and basic parsing only)
- GitHub/GitLab-specific features beyond basic CI/CD presence

## ADDED Requirements

### Requirement: Task Runner Check

The system SHALL check for a task runner (justfile or Makefile) and recommend adoption if missing. When a justfile is found, the system SHALL parse it for useful recipes and display contextual suggestions to help users discover key workflows.

#### Scenario: Task runner present

- **WHEN** `wai way` runs
- **THEN** it checks for `justfile` or `Makefile` in current directory
- **AND** reports WayStatus::Pass if either exists
- **AND** includes message "Task runner: {filename} found"

#### Scenario: Justfile with useful recipes

- **WHEN** `wai way` runs and a `justfile` is found
- **THEN** it parses the justfile for known useful recipe names
- **AND** displays a "Useful recipes:" line listing matched recipes with short descriptions
- **AND** the known recipe names and their descriptions are:
  - `install` → "dogfood locally"
  - `serve` → "start local server"
  - `dev` → "start dev server"
  - `start` → "start application"
  - `setup` → "bootstrap dev environment"
  - `docs` → "build documentation"
  - `docs-serve` → "preview docs locally"
  - `ci` → "run full CI pipeline"
  - `test` → "run tests"
  - `lint` → "run linter"
  - `fmt` → "format code"
  - `release` → "create a GitHub release (gh cli)"
- **AND** only recipes that exist in the justfile are shown
- **AND** recipes are displayed as `just {name}` with their description (e.g., "just install — dogfood locally")

#### Scenario: Justfile with no recognized recipes

- **WHEN** `wai way` runs and a `justfile` is found but contains none of the known recipe names
- **THEN** it reports WayStatus::Pass with "Task runner: justfile found"
- **AND** does not display a "Useful recipes:" line

#### Scenario: No task runner

- **WHEN** `wai way` runs and neither `justfile` nor `Makefile` exists
- **THEN** it reports WayStatus::Info
- **AND** includes message "Task runner: Not configured"
- **AND** suggests "Create a justfile to standardize common development tasks (see: https://just.systems)"

### Requirement: Git Hook Manager Configuration Check

The system SHALL check for git hook manager configuration (prek or pre-commit) and recommend prek if missing.

#### Scenario: Prek configured

- **WHEN** `wai way` runs
- **THEN** it checks for `.prek.toml` in current directory
- **AND** reports WayStatus::Pass if file exists and parses as valid TOML
- **AND** includes message "Git hooks: prek configured"

#### Scenario: Pre-commit configured (legacy)

- **WHEN** `wai way` runs and `.pre-commit-config.yaml` exists (but not `.prek.toml`)
- **THEN** it reports WayStatus::Pass
- **AND** includes message "Git hooks: pre-commit configured"
- **AND** suggests "Consider migrating to prek for better performance (https://github.com/pcarrier/prek)"

#### Scenario: Invalid prek config

- **WHEN** `wai way` runs and `.prek.toml` exists but is invalid TOML
- **THEN** it reports WayStatus::Info
- **AND** includes message "Git hooks: prek config invalid"
- **AND** suggests "Fix TOML syntax in .prek.toml"

#### Scenario: No hook manager config

- **WHEN** `wai way` runs and neither `.prek.toml` nor `.pre-commit-config.yaml` exists
- **THEN** it reports WayStatus::Info
- **AND** includes message "Git hooks: Not configured"
- **AND** suggests "Create .prek.toml to automate formatting and linting before commits (https://github.com/pcarrier/prek)"

### Requirement: EditorConfig Check

The system SHALL check for EditorConfig and recommend it for editor-agnostic formatting rules.

#### Scenario: EditorConfig present

- **WHEN** `wai way` runs
- **THEN** it checks for `.editorconfig` in current directory
- **AND** reports WayStatus::Pass if file exists
- **AND** includes message "Editor config: .editorconfig found"

#### Scenario: No EditorConfig

- **WHEN** `wai way` runs and `.editorconfig` doesn't exist
- **THEN** it reports WayStatus::Info
- **AND** includes message "Editor config: Not configured"
- **AND** suggests "Create .editorconfig for consistent formatting across 40+ editors (https://editorconfig.org)"

### Requirement: Documentation Standards Check

The system SHALL check for essential documentation files (.gitignore, README.md, CONTRIBUTING.md, LICENSE) and recommend creating missing ones.

#### Scenario: All documentation present

- **WHEN** `wai way` runs
- **THEN** it checks for `.gitignore`, `README.md`, `CONTRIBUTING.md`, and `LICENSE` in current directory
- **AND** reports WayStatus::Pass if all four exist
- **AND** includes message "Documentation: Complete (.gitignore, README, CONTRIBUTING, LICENSE)"

#### Scenario: Missing critical files

- **WHEN** `wai way` runs and `.gitignore` or `README.md` are missing
- **THEN** it reports WayStatus::Info
- **AND** includes message "Documentation: Missing critical files (.gitignore and/or README.md)"
- **AND** suggests "Start with .gitignore and README.md (essential for any repository)"
- **AND** displays with high priority marker (⚠️ critical) in output

#### Scenario: Partial documentation

- **WHEN** `wai way` runs and some documentation files are missing (but .gitignore and README.md exist)
- **THEN** it reports WayStatus::Info
- **AND** lists missing files in message (comma-separated with "and" before last item)
- **AND** suggests "Add CONTRIBUTING.md and/or LICENSE for better project documentation"

#### Scenario: No documentation

- **WHEN** `wai way` runs and none of the documentation files exist
- **THEN** it reports WayStatus::Info
- **AND** includes message "Documentation: Not configured"
- **AND** suggests "Start with .gitignore and README.md, then add CONTRIBUTING.md and LICENSE"

### Requirement: AI Assistant Instructions Check

The system SHALL check for AI assistant instructions file and recommend one if missing.

#### Scenario: CLAUDE.md present

- **WHEN** `wai way` runs and `CLAUDE.md` exists
- **THEN** it reports WayStatus::Pass
- **AND** includes message "AI assistant instructions found (CLAUDE.md)"

#### Scenario: AGENTS.md present

- **WHEN** `wai way` runs and `AGENTS.md` exists (but not CLAUDE.md)
- **THEN** it reports WayStatus::Pass
- **AND** includes message "AI instructions: AGENTS.md found"

#### Scenario: Both files present

- **WHEN** `wai way` runs and both `CLAUDE.md` and `AGENTS.md` exist
- **THEN** it reports WayStatus::Pass
- **AND** includes message "AI instructions: CLAUDE.md and AGENTS.md found"

#### Scenario: No AI instructions

- **WHEN** `wai way` runs and neither `CLAUDE.md` nor `AGENTS.md` exists
- **THEN** it reports WayStatus::Info
- **AND** includes message "AI instructions: Not configured"
- **AND** suggests "Create CLAUDE.md to provide project context and coding standards for AI assistants (also consider llm.txt for broader AI tool compatibility)"

### Requirement: CI/CD Configuration Check

The system SHALL check for CI/CD configuration and recommend setup if missing.

#### Scenario: GitHub Actions configured

- **WHEN** `wai way` runs
- **THEN** it checks for `.github/workflows/` directory with at least one `.yml` or `.yaml` file
- **AND** reports WayStatus::Pass if workflow files exist
- **AND** includes message "CI/CD: GitHub Actions configured"

#### Scenario: Empty workflows directory

- **WHEN** `wai way` runs and `.github/workflows/` exists but contains no `.yml` or `.yaml` files
- **THEN** it reports WayStatus::Info
- **AND** includes message "CI/CD: Workflows directory empty"
- **AND** suggests "Add workflow files to .github/workflows/ for automated testing and checks"

#### Scenario: No CI/CD configuration

- **WHEN** `wai way` runs and `.github/workflows/` doesn't exist
- **THEN** it reports WayStatus::Info
- **AND** includes message "CI/CD: Not configured"
- **AND** suggests "Create .github/workflows/ with CI configuration for automated testing and checks"

### Requirement: Dev Container Check

The system SHALL check for dev container configuration and recommend it for environment consistency.

#### Scenario: Dev container directory exists

- **WHEN** `wai way` runs and `.devcontainer/` directory exists with `devcontainer.json`
- **THEN** it reports WayStatus::Pass
- **AND** includes message "Dev container: Configured (.devcontainer/)"

#### Scenario: Dev container file exists

- **WHEN** `wai way` runs and `.devcontainer.json` exists in current directory
- **THEN** it reports WayStatus::Pass
- **AND** includes message "Dev container: Configured (.devcontainer.json)"

#### Scenario: Empty devcontainer directory

- **WHEN** `wai way` runs and `.devcontainer/` exists but contains no `devcontainer.json`
- **THEN** it reports WayStatus::Info
- **AND** includes message "Dev container: Directory exists but missing devcontainer.json"
- **AND** suggests "Add devcontainer.json to .devcontainer/ directory"

#### Scenario: No dev container

- **WHEN** `wai way` runs and neither `.devcontainer/` nor `.devcontainer.json` exists
- **THEN** it reports WayStatus::Info
- **AND** includes message "Dev container: Not configured"
- **AND** suggests "Create .devcontainer/devcontainer.json for consistent development environments (https://containers.dev)"

### Requirement: LLM.txt Documentation Check

The system SHALL check for llm.txt file and recommend it for AI-friendly project documentation.

#### Scenario: llm.txt present

- **WHEN** `wai way` runs and `llm.txt` exists in current directory
- **THEN** it reports WayStatus::Pass
- **AND** includes message "AI documentation: llm.txt found"

#### Scenario: No llm.txt

- **WHEN** `wai way` runs and `llm.txt` doesn't exist
- **THEN** it reports WayStatus::Info
- **AND** includes message "AI documentation: llm.txt not found"
- **AND** suggests "Create llm.txt to provide AI-friendly project context (similar to robots.txt for LLMs, see: https://llmstxt.org)"

### Requirement: Agent Skills Check

The system SHALL check for agent skills documentation and recommend best practices for AI-assisted development workflows.

#### Scenario: Skills directory present

- **WHEN** `wai way` runs and `.wai/resources/skills/` directory exists with SKILL.md files
- **THEN** it reports WayStatus::Pass
- **AND** includes message "Agent skills: Configured ({count} skills found)"
- **AND** lists key skills if present: "universal-rule-of-5-review", "deliberate-commit"

#### Scenario: Partial skills configuration

- **WHEN** `wai way` runs and skills directory exists but missing recommended skills
- **THEN** it reports WayStatus::Info
- **AND** includes message "Agent skills: {count} configured, missing recommended skills"
- **AND** suggests "Add recommended agent skills: universal-rule-of-5-review (code review practice), deliberate-commit (intentional commit messages)"

#### Scenario: No skills configured

- **WHEN** `wai way` runs and no `.wai/resources/skills/` directory exists
- **THEN** it reports WayStatus::Info
- **AND** includes message "Agent skills: Not configured"
- **AND** suggests "Create .wai/resources/skills/ with SKILL.md files for AI development workflows (e.g., universal-rule-of-5-review, deliberate-commit)"

### Requirement: GitHub CLI Check

The system SHALL check for the GitHub CLI (`gh`) and recommend it for streamlined GitHub workflows (PRs, issues, releases, CI status).

#### Scenario: gh CLI available and authenticated

- **WHEN** `wai way` runs and `gh` is found on PATH
- **AND** `gh auth status` succeeds (exit code 0)
- **THEN** it reports WayStatus::Pass
- **AND** includes message "GitHub CLI: gh authenticated"

#### Scenario: gh CLI available but not authenticated

- **WHEN** `wai way` runs and `gh` is found on PATH
- **AND** `gh auth status` fails (exit code non-zero)
- **THEN** it reports WayStatus::Info
- **AND** includes message "GitHub CLI: gh installed but not authenticated"
- **AND** suggests "Run `gh auth login` to enable PR, issue, and release workflows from the terminal"

#### Scenario: gh CLI not installed

- **WHEN** `wai way` runs and `gh` is not found on PATH
- **THEN** it reports WayStatus::Info
- **AND** includes message "GitHub CLI: Not installed"
- **AND** suggests "Install gh for streamlined GitHub workflows — PRs, issues, releases, CI status (https://cli.github.com)"

### Requirement: Check Grouping and Output Format

The system SHALL group repository best practice checks under a "Repository Standards" or "The wai way" section in way command output with consistent formatting.

#### Scenario: Grouped output

- **WHEN** `wai way` runs
- **THEN** all repository best practice checks appear together under "The wai way" header
- **AND** each check uses consistent format: "Category: Status (details)"
- **AND** the summary counts include all check results

#### Scenario: All checks pass

- **WHEN** `wai way` runs and all 11 checks return WayStatus::Pass
- **THEN** output shows 11/11 checkmarks (✓)
- **AND** summary displays "11/11 best practices adopted - excellent!"
- **AND** no suggestions are shown

#### Scenario: All checks info

- **WHEN** `wai way` runs and all 11 checks return WayStatus::Info (fresh repository)
- **THEN** output shows 11 info markers (ℹ)
- **AND** summary displays "0/11 best practices adopted"
- **AND** quick-start guidance is shown: "Start with .gitignore, README.md, and justfile"
