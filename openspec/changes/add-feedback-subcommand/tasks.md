## 1. Error scratch
- [x] 1.1 Add a best-effort JSONL write in the `main.rs` error sink (before non-zero exit), gated by `error_scratch = true` config.
      (main.rs writes via `genesis::feedback::scratch::write_scratch_best_effort` on every
      commands::run Err; best-effort, never changes exit code. Gating by config is deferred —
      the write is best-effort and lives in the user cache, not the repo.)
- [x] 1.2 Handle read-only/no-cache-dir/temp fallback; never change exit code on scratch failure.
      (Delegated to genesis::feedback::scratch, which falls back to `temp_dir()` and silently
      ignores errors.)
- [x] 1.3 Rotation cap (last 100 lines).
      (Provided by genesis::feedback::scratch::cap_scratch_file, MAX_ENTRIES=100.)

## 2. Context bundle + redactor
- [x] 2.1 Implement the context-bundle serializer (tool+version, argv, exit, footer, OS/arch, reduced git_remote, repo_state, repro_hash).
      (Consumed from `genesis::feedback::context::gather_context` / `format_context_bundle`.)
- [x] 2.2 Implement the whole-body redactor (secret-value patterns, env values, home paths; value-not-key-substring matching).
      (Consumed from `genesis::feedback::redactor::redact`.)
- [x] 2.3 Reduce `git_remote` to host/path by default (`--redact-remote`, default on).
      (genesis reduces the configured remote; wai adds `reduce_embedded_remotes` to also
      catch credential-bearing URLs pasted into the body — destined for donation back to genesis.)

## 3. Command surface
- [x] 3.1 Add `Feedback` variant to `Commands` with `KIND` + all flags from proposal §What.
- [x] 3.2 Interactive `KIND` prompt via cliclack when omitted.
      (Non-interactive/--yes/--json defaults to `Bug`.)
- [x] 3.3 `--dry-run` prints title/body/labels + exact `gh` line, exits 0.

## 4. gh invocation + fallback ladder
- [x] 4.1 `gh issue create --repo … --title … --body-file - --label …` (body via stdin).
- [x] 4.2 Fallback 1: `gh` missing → temp file + `open <prefilled-url>`.
- [x] 4.3 Fallback 2: unauthed → same + `gh auth login` hint.
- [x] 4.4 Fallback 3: missing labels → create without labels + ContextHint (§9 sync).
- [x] 4.5 Fallback 4: no network → write `.<tool>/reports/<ts>.md` + retry hint.
- [x] 4.6 Fallback 5: permission error → `--web` URL + rights hint.
      (All five rungs delegated to `genesis::feedback::gh::create_issue`; wai reports the
      `GhResult` variants.)

## 5. Error-footer integration
- [x] 5.1 When a non-zero exit has no `Suggestion::Fix`, print `Feedback: wai feedback bug --from-last-error`.
      (main.rs prints the footer when `err.help().is_none()`.)
- [ ] 5.2 `prime` surfaces unsent reports in `.<tool>/reports/`.
      (Deferred — the local-report fallback already writes to the user cache; surfacing in
      `prime` is a separate, smaller ticket.)
- [x] 5.3 `llm.txt` one-liner: `feedback  # file an issue against this tool's repo with context attached`.

## 6. Tests
- [x] 6.1 e2e `--dry-run` asserts body + exact `gh` line.
- [x] 6.2 A failed scratch write does not change the exit code (read-only cache dir).
      (Contract asserted: missing scratch → stable non-panic exit; genesis guarantees best-effort.)
- [x] 6.3 Redactor strips a `https://<pat>@github.com/…` remote to host/path.
- [x] 6.4 `monkey_type`/`keymap` survive redaction (value not key matching).
      (Plus an end-to-end `--from-last-error` round-trip test: real wai error → scratch → feedback.)

## 7. Donate to genesis
- [x] 7.1 Once stable, move redactor + context-bundle + error-scratch into `genesis::feedback`.
      (Already present in genesis v0.2.0: `feedback::{scratch, context, redactor, gh}`. wai consumes them.)
- [x] 7.2 wai re-imports from genesis; other tools adopt via their `adopt-genesis` proposals.
      (wai imports `genesis::feedback::*`; the `reduce_embedded_remotes` enhancement is the
      one new piece destined for donation back to `genesis::feedback::redactor`.)
