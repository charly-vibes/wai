# skill-sharing Specification

## Purpose
TBD - created by archiving change add-skill-sharing. Update Purpose after archive.
## Requirements
### Requirement: Global Skill Library

The system SHALL maintain a global skill library at `~/.wai/resources/skills/`
that is available in all projects, with project-local skills taking priority
over global skills when names conflict.

#### Scenario: Global skill available in all projects

- **WHEN** a skill is installed globally via `wai resource install <skill> --global`
- **THEN** it is accessible in any project via `wai resource list skills`
- **AND** is marked as `[global]` in the listing output

#### Scenario: Local skill overrides global

- **WHEN** a skill named `issue/gather` exists both locally and globally
- **THEN** the local version is used in all operations within that project
- **AND** the global version remains unchanged

### Requirement: Skill Installation

The CLI SHALL support installing skills from the current project or another local
repository into the global library.

#### Scenario: Install skill globally

- **WHEN** user runs `wai resource install issue/gather --global`
- **THEN** the system copies the skill from the current project's skills directory
  to `~/.wai/resources/skills/issue/gather/SKILL.md`
- **AND** confirms the installation path

#### Scenario: Install skill from another repository

- **WHEN** user runs `wai resource install issue/gather --from-repo ../other-project`
- **THEN** the system copies the skill from `../other-project/.wai/resources/agent-config/skills/issue/gather/SKILL.md`
  into the current project's skills directory
- **AND** confirms the installation path

#### Scenario: Hardcoded project names warned

- **WHEN** installing a skill whose content contains the literal value of the current
  project name or absolute paths beginning with the repository root
- **THEN** the system issues a warning listing the suspicious strings
- **AND** proceeds with installation (warning, not error)

### Requirement: Skill Export and Import

The CLI SHALL support bundling skills for sharing and importing bundles from
external sources.

#### Scenario: Export skills to archive

- **WHEN** user runs `wai resource export issue/gather impl/run --output skills.tar.gz`
- **THEN** the system creates a tar.gz archive containing the SKILL.md files
  preserving the `<category>/<name>/SKILL.md` directory structure

#### Scenario: Import skills from archive

- **WHEN** user runs `wai resource import skills.tar.gz`
- **THEN** the system extracts skill files into the current project's skills directory
- **AND** prompts before overwriting any existing skill
- **AND** reports which skills were installed

#### Scenario: Non-interactive import with --yes

- **WHEN** user runs `wai resource import skills.tar.gz --yes`
- **THEN** the system overwrites existing skills without prompting
- **AND** reports all skills that were overwritten

