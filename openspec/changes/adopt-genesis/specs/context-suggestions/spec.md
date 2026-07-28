# context-suggestions spec delta: adopt genesis suggestions

## MODIFIED Requirements

### Requirement: Suggestion Output Blocks

The `Suggestion` enum and its variants SHALL be sourced from `genesis::suggestions` (re-exported by wai) — `DidYouMean`/`WrongOrder`/`ContextHint`/`Fix` — rather than defined locally in `src/suggestions.rs`. wai SHALL NOT maintain a local copy of the enum after adoption.

#### Scenario: typo suggestion still works after adoption

- **WHEN** `wai statos` (a typo of `status`) is run
- **THEN** wai SHALL emit "Did you mean 'status'?" via `genesis::suggestions`
- **AND** SHALL NOT depend on any local `Suggestion` enum definition.

#### Scenario: wai registers its command list

- **WHEN** wai initializes its suggestion engine
- **THEN** it SHALL register its valid-command list with
  `genesis::suggestions::SuggestionEngine` so typo detection works without
  wai reimplementing the Jaro/Levenshtein matching.