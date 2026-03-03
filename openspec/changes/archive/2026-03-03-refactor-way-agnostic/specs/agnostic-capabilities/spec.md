# OpenSpec Delta: Agnostic Way Capabilities

**Change ID:** `refactor-way-agnostic`
**Capability:** `agnostic-capabilities`
**Status:** `draft`

## ADDED Requirements

### Requirement: Agnostic Repository Best Practice Checks

The `wai way` command SHALL report on agnostic repository capabilities rather than specific tool implementations. Each check SHALL include its intent and success criteria to provide context for both human users and AI agents.

#### Scenario: Check output with intent and success criteria (Verbose)

- **GIVEN** a project with a `justfile`
- **WHEN** user runs `wai way -v`
- **THEN** the output SHALL show the capability name "Command standardization" instead of "Task runner"
- **AND** the output SHALL include an "Intent:" line with the agnostic purpose of the check
- **AND** the output SHALL include a "Success:" line with the agnostic criteria for passing the check

#### Scenario: Check output with intent and success criteria (JSON)

- **GIVEN** any project
- **WHEN** user runs `wai way --json`
- **THEN** the JSON payload SHALL include `intent` and `success_criteria` fields for every check result
- **AND** these fields SHALL be optional (`null` if not provided) to ensure backward compatibility

#### Scenario: Agnostic Check Mapping

The system SHALL map tool-specific checks to agnostic capabilities, each with defined intent and success criteria:

| Capability | Intent | Success Criteria |
| :--- | :--- | :--- |
| **Command standardization** | Provide a single, tool-agnostic entry point for common repository tasks (build, test, deploy). | A standard interface exists for common tasks. |
| **Pre-commit quality gates** | Prevent low-quality commits by running automated checks before code is saved to history. | Automated checks run automatically before code is committed. |
| **Consistent formatting** | Ensure consistent code formatting across different editors and IDEs. | Project-wide style rules are enforced by a shared configuration file. |
| **Project documentation** | Provide essential project identity, onboarding, and legal/contribution guidance. | Essential files provide project context and rules. |
| **AI-agent context** | Provide persistent "rules of the road" and project context for AI collaborators. | Persistent instructions define coding standards and context for AI assistants. |
| **LLM-friendly context** | Provide machine-readable project context and navigation for LLMs. | Machine-readable project documentation (llm.txt) exists. |
| **Extended agent capabilities** | Enhance agent functionality with specialized iterative review and commit workflows. | Specialized agent workflows are active. |
| **Integration & automation** | Streamline repository interactions (PRs, issues, releases) from the CLI. | CLI tools are configured for seamless integration with the hosting provider. |
| **Automated verification** | Ensure code quality and correctness through automated builds and tests on every change. | Every change is automatically validated by a remote build/test pipeline. |
| **Reproducible environments** | Provide a standardized, containerized environment for all contributors. | A configuration exists for a consistent, reproducible dev environment. |
| **Automated delivery** | Automate the process of building, packaging, and publishing software releases. | Software releases and distribution are fully automated. |

### Requirement: Plugin Agnostic Context Support

The plugin system SHALL allow TOML-defined plugins to define their own `intent` and `success_criteria`.

#### Scenario: Plugin with custom context

- **GIVEN** a plugin TOML with `intent` and `success_criteria` defined
- **WHEN** the plugin definition is parsed
- **THEN** these fields are available on the `PluginDef` struct
- **AND** default to `None` if not present in the TOML
