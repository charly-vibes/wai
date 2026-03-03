## ADDED Requirements

### Requirement: Hierarchical Skill Names

Skill names SHALL support a single `/` separator to enable category/action grouping
that maps directly to Claude Code's invocation path convention (`/category:action`).

#### Scenario: Create hierarchical skill

- **WHEN** user runs `wai resource add skill issue/gather`
- **THEN** the system creates `.wai/resources/agent-config/skills/issue/gather/SKILL.md`
- **AND** the skill is listed with name `issue/gather`

#### Scenario: Flat names unchanged

- **WHEN** user runs `wai resource add skill my-skill`
- **THEN** the system creates `.wai/resources/agent-config/skills/my-skill/SKILL.md`
- **AND** existing flat naming behavior is preserved

#### Scenario: Invalid hierarchical names rejected

- **WHEN** user provides a skill name with two or more slashes (e.g., `a/b/c`)
- **THEN** the system rejects it with a clear error: only one `/` is allowed

#### Scenario: Empty segments rejected

- **WHEN** user provides a name with an empty segment (e.g., `/gather` or `issue/`)
- **THEN** the system rejects it with an error explaining both segments must be non-empty

#### Scenario: Category grouping in list output

- **WHEN** user runs `wai resource list skills`
- **AND** hierarchical skills exist (e.g., `issue/gather`, `issue/create`)
- **THEN** skills are displayed with their full hierarchical name (e.g., `issue/gather`)
- **AND** the JSON output includes a `category` field for hierarchical skills

#### Scenario: Frontmatter name for hierarchical skill

- **WHEN** user runs `wai resource add skill issue/gather`
- **THEN** the generated SKILL.md frontmatter contains `name: issue/gather`
  (the full hierarchical name, not just the action segment)
- **AND** this name is what `add-claude-code-projection` reads for frontmatter translation

#### Scenario: Flat-name collision with category directory rejected

- **WHEN** a flat skill named `issue` already exists at `skills/issue/SKILL.md`
- **AND** user runs `wai resource add skill issue/gather`
- **THEN** the system errors with a message explaining that `issue` is already a flat skill
  and cannot also be used as a category name
- **AND** suggests renaming the existing flat skill before creating the category
