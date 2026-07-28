# Change: Add `feedback` subcommand

## Why

Friction observed in the field dies in chat sessions; maintainers never see
which errors have no self-healing fix, which docs/AIX gaps agents hit. The
`feedback` subcommand lets an agent (or human) file a well-contexted,
well-tagged issue against wai's upstream repo via `gh`, closing the loop:
error-with-no-fix → footer offers `feedback` → the fix becomes a
`Suggestion::Fix` → the footer stops offering `feedback` for that case.

Full design: the agent-issue-reporting playbook (genesis repo, `.wai` research)
. The verb is
`feedback`, not `report` — `report` is reserved in pretender and espectacular
(see playbook §0 verb audit).

## What Changes

- New `wai feedback [KIND]` subcommand (`bug`/`friction`/`docs-gap`/`aix-gap`/`idea`).
- Auto-gathered context bundle (tool+version, exact failing argv, exit code,
  `Suggestion` footer, OS/arch, reduced `git_remote`, `repo_state`, repro hash).
- Error-scratch JSONL written on every non-zero exit (best-effort, never
  shadows the real error) so `--from-last-error` works.
- Whole-body redactor (secret values, not key substrings; `git_remote` → host/path).
- `gh issue create` primary path + 5-step fallback ladder (missing/unauthed/
  labels/no-network/permission → prefilled URL).
- `--dry-run`, `--web`, `--json`, `--yes`, `--no-context`, `--redact-remote`.
- Error footer hook: non-zero exits with no `Suggestion::Fix` print
  `Feedback: wai feedback bug --from-last-error`.
- The redactor, context-bundle serializer, and error scratch live in
  `genesis::feedback` (per `genesis` foundation proposal); wai is the
  reference impl and first consumer.

## Impact

- Affected specs: new `feedback` capability under `cli-core` (delta).
- Affected code: new `commands/feedback.rs`; error-scratch write in the
  `main.rs` error sink; footer hook in `suggestions` (now from genesis).
- Blocked by: wai implements `feedback` first as the reference impl, then
  donates the redactor/context-bundle/error-scratch to `genesis::feedback`
  (genesis foundation tasks 5.x). The genesis foundation is NOT a blocker —
  wai lands first and is the donor.
- Labels: requires the charly-monorepo label-sync workflow (playbook §9).
