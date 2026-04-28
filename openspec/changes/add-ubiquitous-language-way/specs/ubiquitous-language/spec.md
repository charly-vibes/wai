## ADDED Requirements

### Requirement: Progressive-Disclosure Ubiquitous Language Resource

The system SHALL treat `.wai/resources/ubiquitous-language/` as the canonical home for repository ubiquitous-language artifacts.

The resource tree SHALL use progressive disclosure so agents can load a small navigation file first and then open only the bounded-context files relevant to the current task.

The canonical structure is:

```text
.wai/resources/ubiquitous-language/
├── README.md
├── shared.md              # optional
└── contexts/
    ├── <context>.md
    └── ...
```

- `README.md` is the navigation entrypoint and loading guide
- `shared.md` is optional and contains only truly cross-context terms
- `contexts/*.md` contain detailed term definitions scoped to a bounded context

A fully configured tree for ongoing use SHALL contain `README.md` and at least one bounded-context file under `contexts/`. `shared.md` MAY exist in addition to those files, but it does not replace bounded-context files.

#### Scenario: Canonical tree exists

- **WHEN** a repository adopts ubiquitous-language resources with wai
- **THEN** the canonical location is `.wai/resources/ubiquitous-language/`
- **AND** the root contains `README.md`
- **AND** the tree may contain `shared.md`
- **AND** bounded-context term files live under `contexts/`

#### Scenario: Agent loads terminology progressively

- **WHEN** an agent needs domain terminology for a task
- **THEN** it reads `.wai/resources/ubiquitous-language/README.md` first
- **AND** it opens only the bounded-context files relevant to the task instead of loading every term file

### Requirement: Root Index Defines Navigation And Loading Rules

The root `README.md` SHALL act as a lightweight index rather than a full glossary dump.

It SHALL:
- describe the purpose and scope of the ubiquitous-language tree
- list the available bounded contexts
- point to `shared.md` for cross-context terms
- instruct readers to open only the context files relevant to the current task

#### Scenario: Root index remains lightweight

- **WHEN** `.wai/resources/ubiquitous-language/README.md` is present
- **THEN** it describes how the tree is organized
- **AND** it lists available bounded-context files under `contexts/`
- **AND** it does not need to duplicate every term definition from those files

### Requirement: Bounded-Context Files Capture Terms Explicitly

Each bounded-context file under `contexts/` SHALL record terms in a structured, reviewable format suitable for both humans and agents.

The bounded context is defined primarily by the file location itself (for example, `contexts/orders.md` defines the `orders` bounded context). Individual term entries MAY repeat the context name, but they are not required to do so if the file-level context is already clear.

Each term entry SHALL include:
- the preferred term
- a concise definition
- discouraged synonyms, anti-terms, or deprecated variants when relevant
- related terms when helpful

#### Scenario: Context file records a term

- **WHEN** a term is added to `contexts/orders.md`
- **THEN** the file identifies `orders` as the bounded context
- **AND** each term entry includes the preferred term and definition
- **AND** the file may also include anti-terms and related terms

#### Scenario: Shared-only tree is not yet fully configured

- **WHEN** `.wai/resources/ubiquitous-language/README.md` and `shared.md` exist
- **AND** no bounded-context files exist under `contexts/`
- **THEN** the tree is considered a valid skeleton
- **AND** it is not yet considered fully configured for ongoing use

### Requirement: Ubiquitous-Language Skill Workflow

The system SHALL support a reusable skill workflow for maintaining ubiquitous-language resources incrementally.

The workflow SHALL direct agents to:
- search existing artifacts and code for domain terminology
- identify the appropriate bounded context for each term
- update `README.md`, `shared.md`, or the relevant `contexts/*.md` file as needed
- prefer incremental edits to the resource tree over creating a single monolithic glossary file

#### Scenario: Agent updates an existing context file

- **WHEN** an agent is asked to add or normalize domain terminology
- **THEN** it checks the root index first
- **AND** it updates the relevant bounded-context file when one already exists
- **AND** it avoids moving unrelated terminology into one large catch-all file
