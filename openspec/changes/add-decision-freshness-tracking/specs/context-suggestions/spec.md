# Spec delta: context-suggestions

## ADDED Requirements

### Requirement: wai status surfaces stale decision artifacts

When the freshness scanner finds stale artifacts, `wai status` SHALL include
suggestions naming the stale artifact(s) and prompting re-evaluation. At most
3 stale artifacts MUST be listed inline; if more exist, a single summary line
SHALL read "N decision artifacts are stale — run `wai artifacts stale` for
details." Untracked artifacts (tracks declared but no sidecar) MUST NOT be
surfaced in `wai status`.

#### Scenario: One stale artifact surfaces in status

- **GIVEN** one tracked artifact is stale (its tracked file has changed)
- **WHEN** `wai status` runs
- **THEN** suggestions include a line naming the artifact and the changed path

#### Scenario: Many stale artifacts collapse to count

- **GIVEN** four or more tracked artifacts are stale
- **WHEN** `wai status` runs
- **THEN** suggestions include a single line with the count and a pointer to `wai artifacts stale`
- **AND** individual artifact names are NOT listed inline

#### Scenario: No stale artifacts produce no suggestion

- **GIVEN** all tracked artifacts are current
- **WHEN** `wai status` runs
- **THEN** no stale-artifact suggestion appears in output
