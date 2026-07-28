# cli-core spec delta: feedback

## ADDED Requirements

### Requirement: feedback subcommand

wai SHALL provide a `feedback` subcommand that files a structured issue
against wai's upstream repo via the `gh` CLI.

#### Scenario: agent files a bug with last error

- **WHEN** `wai feedback bug --from-last-error --yes` is run after a non-zero exit
- **THEN** wai SHALL read the last error scratch line
- **AND** SHALL assemble a body with the exact failing argv, exit code, suggestion footer, and context bundle
- **AND** SHALL redact the whole body per the privacy rules
- **AND** SHALL invoke `gh issue create` against `github.com/charly-vibes/wai` with labels `agent-reported`, `bug`, `has-repro`.

#### Scenario: gh is absent

- **WHEN** `gh` is not on PATH
- **THEN** wai SHALL print the body to a temp file and emit a `Suggestion::Fix` offering to `open <prefilled-url>`.

#### Scenario: credential in git_remote

- **WHEN** the origin URL contains embedded credentials
- **THEN** the context bundle SHALL reduce it to `host/path` (dropping `user:pass@` and query)
- **AND** SHALL NOT include the raw credential in the filed issue.

#### Scenario: error with no self-healing fix

- **WHEN** wai exits non-zero and no `Suggestion::Fix` is available
- **THEN** the error footer SHALL print `Feedback: wai feedback bug --from-last-error`.
