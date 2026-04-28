## MODIFIED Requirements

### Requirement: Skill Template Library

The CLI SHALL provide built-in templates that scaffold common agent pipeline skill
patterns when creating a new skill.

#### Scenario: Create skill from template

- **WHEN** user runs `wai resource add skill issue/gather --template gather`
- **THEN** the system creates `SKILL.md` with the gather template body
- **AND** the template uses `$ARGUMENTS`, `$PROJECT`, `$REPO_ROOT` placeholders
- **AND** the SKILL.md is ready to use without further editing of placeholders

#### Scenario: Available templates

- **WHEN** user runs `wai resource add skill <name> --template <name>`
- **THEN** the system accepts the following template names:
  - `gather` — research stub: `wai search $ARGUMENTS`, codebase exploration,
    `wai add research`
  - `create` — creation stub: artifact retrieval via `wai search --latest`,
    item loop with output tracking
  - `tdd` — test-first stub: RED/GREEN/REFACTOR loop with `cargo test` / `just check`
  - `rule-of-5` — review stub: 5-pass review with convergence check and
    APPROVED / NEEDS_CHANGES / NEEDS_HUMAN verdict
  - `ubiquitous-language` — terminology curation stub: read the ubiquitous-language
    index, search artifacts for domain terms, and update the relevant
    `.wai/resources/ubiquitous-language/contexts/*.md` files incrementally

#### Scenario: Unknown template name rejected

- **WHEN** user runs `wai resource add skill <name> --template unknown-template`
- **THEN** the system rejects the command with an error
- **AND** lists the valid template names in the error message

#### Scenario: No template gives bare stub

- **WHEN** user runs `wai resource add skill <name>` without `--template`
- **THEN** the system creates the existing minimal stub (no regression)

#### Scenario: Templates are portable

- **WHEN** a template-generated SKILL.md is used in a different project
- **THEN** all project-specific values are provided via `$ARGUMENTS`, `$PROJECT`,
  or `$REPO_ROOT` at runtime — no hardcoded project names exist in the template

#### Scenario: Ubiquitous-language template preserves progressive disclosure

- **WHEN** user runs `wai resource add skill <name> --template ubiquitous-language`
- **THEN** the generated skill instructs the agent to read `.wai/resources/ubiquitous-language/README.md` first
- **AND** update only the relevant bounded-context files under `contexts/`
- **AND** avoid collapsing all terminology into one giant glossary file
