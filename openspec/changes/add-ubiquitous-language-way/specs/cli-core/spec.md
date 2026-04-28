## ADDED Requirements

### Requirement: Ubiquitous-Language Skill Template Alias

The `wai add skill` command SHALL expose `ubiquitous-language` as a valid built-in template name.

#### Scenario: Add skill with ubiquitous-language template

- **WHEN** user runs `wai add skill <name> --template ubiquitous-language`
- **THEN** the system creates the skill file pre-populated with the ubiquitous-language template
- **AND** the generated skill is equivalent to using `wai resource add skill <name> --template ubiquitous-language`
