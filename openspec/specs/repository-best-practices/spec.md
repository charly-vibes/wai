# repository-best-practices Specification

## Purpose

The `wai way` command reports on agnostic repository capabilities rather than specific tool implementations. Each capability check includes its intent and success criteria to provide context for both human users and AI agents. Verbose output (`-v`) shows intent and criteria; JSON output (`--json`) always includes them.

## Requirements

### Requirement: Command Standardization

**Intent:** Provide a single, tool-agnostic entry point for common repository tasks (build, test, deploy).
**Success Criteria:** A standard interface (justfile, Makefile, npm scripts) exists for common tasks.

The system SHALL check for a task runner (justfile or Makefile) and recommend adoption if missing. When a justfile is found, the system SHALL parse it for useful recipes and display contextual suggestions to help users discover key workflows.

#### Scenario: Task runner present

- **WHEN** `wai way` runs
- **THEN** it checks for `justfile` or `Makefile` in current directory
- **AND** reports pass if either exists
- **AND** the capability name in output is "Command standardization"

#### Scenario: Justfile with useful recipes

- **WHEN** `wai way` runs and a `justfile` is found
- **THEN** it parses the justfile for known useful recipe names
- **AND** displays a message listing matched recipes

#### Scenario: No task runner

- **WHEN** `wai way` runs and neither `justfile` nor `Makefile` exists
- **THEN** it reports info status
- **AND** suggests adding a justfile

### Requirement: Pre-commit Quality Gates

**Intent:** Prevent low-quality commits by running automated checks before code is saved to history.
**Success Criteria:** Automated checks (linters, tests) run automatically before code is committed.

The system SHALL check for git hook manager configuration (prek or pre-commit) and recommend prek if missing.

#### Scenario: Prek configured

- **WHEN** `wai way` runs and `.prek.toml` exists
- **THEN** it reports pass
- **AND** the capability name in output is "Pre-commit quality gates"

#### Scenario: Pre-commit configured (legacy)

- **WHEN** `wai way` runs and `.pre-commit-config.yaml` exists (but not `.prek.toml`)
- **THEN** it reports pass
- **AND** suggests migrating to prek

#### Scenario: No hook manager config

- **WHEN** `wai way` runs and neither `.prek.toml` nor `.pre-commit-config.yaml` exists
- **THEN** it reports info status
- **AND** suggests adding prek

### Requirement: Consistent Formatting

**Intent:** Ensure consistent code formatting across different editors and IDEs.
**Success Criteria:** Project-wide style rules are enforced by a shared configuration file.

The system SHALL check for EditorConfig and recommend it for editor-agnostic formatting rules.

#### Scenario: EditorConfig present

- **WHEN** `wai way` runs and `.editorconfig` exists
- **THEN** it reports pass
- **AND** the capability name in output is "Consistent formatting"

#### Scenario: No EditorConfig

- **WHEN** `wai way` runs and `.editorconfig` doesn't exist
- **THEN** it reports info status
- **AND** suggests adding .editorconfig

### Requirement: Project Documentation

**Intent:** Provide essential project identity, onboarding, and legal/contribution guidance.
**Success Criteria:** Essential files (README, .gitignore, LICENSE) provide project context and rules.

The system SHALL check for essential documentation files (.gitignore, README.md, CONTRIBUTING.md, LICENSE) and recommend creating missing ones.

#### Scenario: All documentation present

- **WHEN** `wai way` runs and all four files exist
- **THEN** it reports pass
- **AND** the capability name in output is "Project documentation"

#### Scenario: Missing critical files

- **WHEN** `wai way` runs and `.gitignore` or `README.md` are missing
- **THEN** it reports info status with ⚠️ indicator
- **AND** suggests adding the missing critical files

#### Scenario: Partial documentation

- **WHEN** `wai way` runs and some files are present but not all
- **THEN** it reports based on whether critical files exist

#### Scenario: No documentation

- **WHEN** `wai way` runs and none of the documentation files exist
- **THEN** it reports info status

### Requirement: AI-Agent Context

**Intent:** Provide persistent "rules of the road" and project context for AI collaborators.
**Success Criteria:** Persistent instructions define coding standards and context for AI assistants.

The system SHALL check for AI assistant instructions file and recommend one if missing.

#### Scenario: CLAUDE.md present

- **WHEN** `wai way` runs and `CLAUDE.md` exists
- **THEN** it reports pass
- **AND** the capability name in output is "AI-agent context"
- **AND** if no WAI:REFLECT block is found, suggests running `wai reflect`

#### Scenario: AGENTS.md present

- **WHEN** `wai way` runs and `AGENTS.md` exists (but not CLAUDE.md)
- **THEN** it reports pass
- **AND** suggests adding CLAUDE.md for Claude Code compatibility

#### Scenario: No AI instructions

- **WHEN** `wai way` runs and neither `CLAUDE.md` nor `AGENTS.md` exists
- **THEN** it reports info status
- **AND** suggests creating CLAUDE.md

### Requirement: Automated Verification

**Intent:** Ensure code quality and correctness through automated builds and tests on every change.
**Success Criteria:** Every change is automatically validated by a remote build/test pipeline.

The system SHALL check for CI/CD configuration and recommend setup if missing.

#### Scenario: GitHub Actions configured

- **WHEN** `wai way` runs and `.github/workflows/` contains at least one workflow file
- **THEN** it reports pass
- **AND** the capability name in output is "Automated verification"

#### Scenario: Empty workflows directory

- **WHEN** `wai way` runs and `.github/workflows/` exists but is empty
- **THEN** it reports info status

#### Scenario: GitLab CI or CircleCI configured

- **WHEN** `wai way` runs and `.gitlab-ci.yml` or `.circleci/config.yml` exists
- **THEN** it reports pass

#### Scenario: No CI/CD configuration

- **WHEN** `wai way` runs and no CI/CD config is found
- **THEN** it reports info status
- **AND** suggests setting up continuous integration

### Requirement: Reproducible Environments

**Intent:** Provide a standardized, containerized environment for all contributors.
**Success Criteria:** A configuration exists to spin up a consistent, reproducible dev environment.

The system SHALL check for dev container configuration and recommend it for environment consistency.

#### Scenario: Dev container directory exists

- **WHEN** `wai way` runs and `.devcontainer/` directory exists
- **THEN** it reports pass
- **AND** the capability name in output is "Reproducible environments"

#### Scenario: Dev container file exists

- **WHEN** `wai way` runs and `.devcontainer.json` exists
- **THEN** it reports pass

#### Scenario: No dev container

- **WHEN** `wai way` runs and neither `.devcontainer/` nor `.devcontainer.json` exists
- **THEN** it reports info status
- **AND** suggests adding .devcontainer/

### Requirement: LLM-Friendly Context

**Intent:** Provide machine-readable project context and navigation for LLMs.
**Success Criteria:** Machine-readable project documentation (llm.txt) exists for AI tools.

The system SHALL check for llm.txt file and recommend it for AI-friendly project documentation.

#### Scenario: llm.txt present

- **WHEN** `wai way` runs and `llm.txt` exists
- **THEN** it reports pass
- **AND** the capability name in output is "LLM-friendly context"

#### Scenario: No llm.txt

- **WHEN** `wai way` runs and `llm.txt` doesn't exist
- **THEN** it reports info status
- **AND** suggests adding llm.txt

### Requirement: Extended Agent Capabilities

**Intent:** Enhance agent functionality with specialized iterative review and commit workflows.
**Success Criteria:** Specialized agent workflows (Rule of 5, Deliberate Commits) are active.

The system SHALL check for agent skills in `.wai/resources/agent-config/skills/` and recommend `rule-of-5-universal` and `commit`. Skill identity is resolved by directory name OR by `aliases` declared in SKILL.md frontmatter.

#### Scenario: Both recommended skills present

- **WHEN** `wai way` runs and both `rule-of-5-universal` (or `ro5`) and `commit` exist
- **THEN** it reports pass
- **AND** the capability name in output is "Extended agent capabilities"

#### Scenario: Skills present via alias (ro5)

- **WHEN** a skill directory exists with `aliases: [ro5]` in SKILL.md frontmatter
- **THEN** that skill satisfies the `rule-of-5-universal` check

#### Scenario: Partial skills configuration

- **WHEN** skills directory exists but one or both recommended skills are missing
- **THEN** it reports info status and lists what's missing

#### Scenario: No skills configured

- **WHEN** `.wai/resources/agent-config/skills/` does not exist
- **THEN** it reports info status

#### Scenario: Skills directory empty

- **WHEN** `.wai/resources/agent-config/skills/` exists but contains no SKILL.md files
- **THEN** it reports info status

### Requirement: Agent Skills Fix

The system SHALL provide `wai way --fix skills` to scaffold missing recommended skills.

#### Scenario: Fix scaffolds missing skills

- **WHEN** user runs `wai way --fix skills`
- **THEN** the system creates the skills directory if it does not exist
- **AND** creates `rule-of-5-universal/SKILL.md` if not present
- **AND** creates `commit/SKILL.md` if not present
- **AND** reports each created skill

#### Scenario: Fix skips existing skills

- **WHEN** user runs `wai way --fix skills` and a recommended skill already exists
- **THEN** the system skips that skill and reports "already present"

#### Scenario: Unknown fix target

- **WHEN** user runs `wai way --fix <unknown>`
- **THEN** the system exits with an error: "Unknown fix target '{value}'. Available: skills"

### Requirement: Integration & Automation

**Intent:** Streamline repository interactions (PRs, issues, releases) from the CLI.
**Success Criteria:** CLI tools are configured for seamless integration with the hosting provider.

The system SHALL check for the GitHub CLI (`gh`) and recommend it for streamlined GitHub workflows.

#### Scenario: gh CLI available and authenticated

- **WHEN** `wai way` runs and `gh` is on PATH and `gh auth status` succeeds
- **THEN** it reports pass
- **AND** the capability name in output is "Integration & automation"

#### Scenario: gh CLI available but not authenticated

- **WHEN** `wai way` runs and `gh` is on PATH but `gh auth status` fails
- **THEN** it reports info status
- **AND** suggests running `gh auth login`

#### Scenario: gh CLI not installed

- **WHEN** `wai way` runs and `gh` is not on PATH
- **THEN** it reports info status
- **AND** suggests installing gh

### Requirement: Automated Delivery

**Intent:** Automate the process of building, packaging, and publishing software releases.
**Success Criteria:** Software releases and distribution (packages, binaries) are fully automated.

The system SHALL check for a release pipeline in binary projects and recommend setup if missing. Library projects are exempt.

#### Scenario: Library project

- **WHEN** `wai way` runs and the project has no binary target
- **THEN** it reports pass with "Library project — release pipeline not required"
- **AND** the capability name in output is "Automated delivery"

#### Scenario: Release tool detected

- **WHEN** `wai way` runs and goreleaser, cargo-dist, or a release workflow is found
- **THEN** it reports pass

#### Scenario: No release pipeline

- **WHEN** `wai way` runs, the project has a binary target, and no release tool is found
- **THEN** it reports info status
- **AND** suggests goreleaser or cargo-dist

### Requirement: Verbose Output

The system SHALL show `intent` and `success_criteria` for each capability when `--verbose` (`-v`) is used.

#### Scenario: Verbose output includes agnostic context

- **GIVEN** any project
- **WHEN** user runs `wai way -v`
- **THEN** each capability line is followed by an "Intent:" line and a "Success:" line
- **AND** normal (non-verbose) output does not show these lines

### Requirement: JSON Output

The system SHALL include `intent` and `success_criteria` in JSON output. These fields are optional (`null` if not set) for backward compatibility.

#### Scenario: JSON includes agnostic fields

- **GIVEN** any project
- **WHEN** user runs `wai way --json`
- **THEN** the JSON payload includes `intent` and `success_criteria` for every check result

### Requirement: Plugin Agnostic Context Support

The plugin system SHALL allow TOML-defined plugins to define their own `intent` and `success_criteria`.

#### Scenario: Plugin with custom context

- **GIVEN** a plugin TOML with `intent` and `success_criteria` defined
- **WHEN** the plugin definition is parsed
- **THEN** these fields are available on the `PluginDef` struct
- **AND** default to `None` if not present in the TOML

#### Scenario: TOML plugin file parsed correctly

- **GIVEN** a `.toml` plugin file in `.wai/plugins/` with content:
  ```toml
  name = "my-check"
  intent = "Ensure the repo has a CODEOWNERS file."
  success_criteria = "CODEOWNERS file exists in the repo root or .github/ directory."
  ```
- **WHEN** the plugin loader scans `.wai/plugins/` for `*.toml` files
- **THEN** the file is parsed with `toml::from_str`
- **AND** `intent` and `success_criteria` are available on the resulting `PluginDef`
- **AND** YAML files (`*.yml`, `*.yaml`) are not loaded

### Requirement: Output Format

The system SHALL format all checks consistently.

#### Scenario: All checks pass

- **WHEN** `wai way` runs and all checks return pass
- **THEN** output shows checkmarks (✓) for each
- **AND** summary displays "excellent! All best practices adopted"

#### Scenario: Fresh repository

- **WHEN** `wai way` runs on a fresh repository with no tools configured
- **THEN** output shows info markers (ℹ) for each check
- **AND** summary displays "0/{n} best practices adopted — quick-start: add README.md, justfile, .gitignore"
