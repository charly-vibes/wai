## ADDED Requirements

### Requirement: Ubiquitous Language Guidance In Managed Block

The generated WAI block SHALL mention ubiquitous-language resources when `.wai/resources/ubiquitous-language/README.md` exists.

The guidance SHALL tell agents to:
- consult `.wai/resources/ubiquitous-language/`
- read `README.md` first as the navigation entrypoint
- open only the bounded-context files relevant to the current task before introducing or reusing domain terms
- avoid broad loading of every terminology file unless the task truly spans multiple bounded contexts

#### Scenario: Ubiquitous-language tree present

- **WHEN** `wai init` or `wai reflect` runs
- **AND** `.wai/resources/ubiquitous-language/README.md` exists
- **THEN** the generated WAI block includes a note pointing agents to `.wai/resources/ubiquitous-language/`
- **AND** the note instructs them to read the root index first and then only relevant bounded-context files

#### Scenario: Ubiquitous-language tree absent

- **WHEN** `wai init` or `wai reflect` runs
- **AND** `.wai/resources/ubiquitous-language/README.md` does not exist
- **THEN** no ubiquitous-language note is included in the generated WAI block

#### Scenario: Partial tree without root index

- **WHEN** `wai init` or `wai reflect` runs
- **AND** `.wai/resources/ubiquitous-language/README.md` does not exist
- **AND** other files exist under `.wai/resources/ubiquitous-language/`
- **THEN** no ubiquitous-language note is included in the generated WAI block because the navigation entrypoint is missing
