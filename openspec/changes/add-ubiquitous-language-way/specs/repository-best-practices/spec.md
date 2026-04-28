## ADDED Requirements

### Requirement: Ubiquitous Language Context

The system SHALL provide a repository best-practice check for a canonical, machine-readable source of domain terminology so humans and agents use the same language.

**Intent:** Provide a canonical, machine-readable source of domain terminology so humans and agents use the same language.
**Success Criteria:** A progressively disclosed ubiquitous-language resource tree exists with a lightweight index and bounded-context term files.

`wai way` SHALL check for ubiquitous-language resources in `.wai/resources/ubiquitous-language/` and recommend that location when missing. A repository adopting this practice SHALL use a lightweight root index plus bounded-context files rather than a single monolithic glossary.

A fully configured tree consists of:
- `.wai/resources/ubiquitous-language/README.md`
- optional `.wai/resources/ubiquitous-language/shared.md`
- one or more bounded-context files under `.wai/resources/ubiquitous-language/contexts/`

A skeleton tree may contain only:
- `.wai/resources/ubiquitous-language/README.md`
- optional `.wai/resources/ubiquitous-language/shared.md`

#### Scenario: Fully configured ubiquitous-language tree

- **WHEN** `wai way` runs and `.wai/resources/ubiquitous-language/README.md` exists
- **AND** `.wai/resources/ubiquitous-language/contexts/` contains at least one `.md` file
- **THEN** it reports pass
- **AND** the capability name in output is "Ubiquitous language context"

#### Scenario: Skeleton tree present

- **WHEN** `wai way` runs and `.wai/resources/ubiquitous-language/` exists
- **AND** `README.md` exists
- **AND** no bounded-context files exist yet
- **THEN** it reports info status
- **AND** suggests adding bounded-context files under `contexts/`

#### Scenario: Shared-only early-stage tree

- **WHEN** `wai way` runs and `.wai/resources/ubiquitous-language/README.md` exists
- **AND** `.wai/resources/ubiquitous-language/shared.md` exists
- **AND** no bounded-context files exist yet
- **THEN** it reports info status
- **AND** explains that the tree is a valid starting point but not yet fully configured
- **AND** suggests adding bounded-context files under `contexts/`

#### Scenario: No ubiquitous-language tree

- **WHEN** `wai way` runs and `.wai/resources/ubiquitous-language/README.md` does not exist
- **THEN** it reports info status
- **AND** suggests creating `.wai/resources/ubiquitous-language/` with a lightweight index plus context files

#### Scenario: Malformed tree without root index

- **WHEN** `wai way` runs and `.wai/resources/ubiquitous-language/` exists
- **AND** one or more files exist under `contexts/`
- **AND** `README.md` does not exist
- **THEN** it reports info status
- **AND** explains that `README.md` is required as the root index for progressive disclosure
