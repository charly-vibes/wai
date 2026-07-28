## 1. Error scratch
- [ ] 1.1 Add a best-effort JSONL write in the `main.rs` error sink (before non-zero exit), gated by `error_scratch = true` config.
- [ ] 1.2 Handle read-only/no-cache-dir/temp fallback; never change exit code on scratch failure.
- [ ] 1.3 Rotation cap (last 100 lines).

## 2. Context bundle + redactor
- [ ] 2.1 Implement the context-bundle serializer (tool+version, argv, exit, footer, OS/arch, reduced git_remote, repo_state, repro_hash).
- [ ] 2.2 Implement the whole-body redactor (secret-value patterns, env values, home paths; value-not-key-substring matching).
- [ ] 2.3 Reduce `git_remote` to host/path by default (`--redact-remote`, default on).

## 3. Command surface
- [ ] 3.1 Add `Feedback` variant to `Commands` with `KIND` + all flags from proposal §What.
- [ ] 3.2 Interactive `KIND` prompt via cliclack when omitted.
- [ ] 3.3 `--dry-run` prints title/body/labels + exact `gh` line, exits 0.

## 4. gh invocation + fallback ladder
- [ ] 4.1 `gh issue create --repo … --title … --body-file - --label …` (body via stdin).
- [ ] 4.2 Fallback 1: `gh` missing → temp file + `open <prefilled-url>`.
- [ ] 4.3 Fallback 2: unauthed → same + `gh auth login` hint.
- [ ] 4.4 Fallback 3: missing labels → create without labels + ContextHint (§9 sync).
- [ ] 4.5 Fallback 4: no network → write `.<tool>/reports/<ts>.md` + retry hint.
- [ ] 4.6 Fallback 5: permission error → `--web` URL + rights hint.

## 5. Error-footer integration
- [ ] 5.1 When a non-zero exit has no `Suggestion::Fix`, print `Feedback: wai feedback bug --from-last-error`.
- [ ] 5.2 `prime` surfaces unsent reports in `.<tool>/reports/`.
- [ ] 5.3 `llm.txt` one-liner: `feedback  # file an issue against this tool's repo with context attached`.

## 6. Tests
- [ ] 6.1 e2e `--dry-run` asserts body + exact `gh` line.
- [ ] 6.2 A failed scratch write does not change the exit code (read-only cache dir).
- [ ] 6.3 Redactor strips a `https://<pat>@github.com/…` remote to host/path.
- [ ] 6.4 `monkey_type`/`keymap` survive redaction (value not key matching).

## 7. Donate to genesis
- [ ] 7.1 Once stable, move redactor + context-bundle + error-scratch into `genesis::feedback`.
- [ ] 7.2 wai re-imports from genesis; other tools adopt via their `adopt-genesis` proposals.
