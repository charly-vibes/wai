## ADDED Requirements

### Requirement: Plan Artifact Tags

The CLI SHALL support tagging plan artifacts with arbitrary labels for structured retrieval.

#### Scenario: Add plan with tags

- **WHEN** user runs `wai add plan "approach" --tags topic:ant-forager,pipeline:impl`
- **THEN** the plan file includes YAML frontmatter with the provided tags
- **AND** the file body follows the frontmatter block

#### Scenario: Plan tags are searchable

- **WHEN** a plan has been tagged with `topic:ant-forager`
- **AND** user runs `wai search "ant-forager" --tag topic:ant-forager --type plan`
- **THEN** the tagged plan appears in results
- **AND** untagged plans matching the text query but lacking the tag are excluded

### Requirement: Design Artifact Tags

The CLI SHALL support tagging design artifacts with arbitrary labels for structured retrieval.

#### Scenario: Add design with tags

- **WHEN** user runs `wai add design "architecture" --tags topic:ant-forager`
- **THEN** the design file includes YAML frontmatter with the provided tags
- **AND** the file body follows the frontmatter block

#### Scenario: Design tags are searchable

- **WHEN** a design has been tagged with `topic:ant-forager`
- **AND** user runs `wai search "ant-forager" --tag topic:ant-forager --type design`
- **THEN** the tagged design appears in results
