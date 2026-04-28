# Artifacts Context

## Artifact

**Definition:** A dated Markdown file that captures reasoning at a specific point in time. Types include `research`, `design`, `plan`, `handoff`, and `review`. Stored under `.wai/projects/<name>/`. Created with `wai add <type>`.

**Anti-terms:** Do not use "document", "note", or "file" — "artifact" is the canonical term.

**Related:** Phase, Frontmatter, Artifact lock, Addendum

---

## Frontmatter

**Definition:** YAML metadata at the top of an artifact file (between `---` markers). Contains fields like `title`, `date`, `tags`, and `project`. Used by `wai search` for filtering and by the oracle for context gathering.

**Anti-terms:** Do not call it "header" or "metadata block".

**Related:** Artifact

---

## Artifact lock

**Definition:** A SHA-256 hash sidecar (`.lock` file) written alongside an artifact to freeze its content. Created when a pipeline step with `lock = true` advances, or manually via `wai pipeline lock`. Verified by `wai pipeline verify` and `wai doctor`.

**Anti-terms:** Do not say "frozen artifact" or "sealed artifact" — "locked artifact" is the correct term.

**Related:** Artifact, Pipeline gate, Addendum

---

## Addendum

**Definition:** A correction artifact that references a locked artifact via `--corrects=<path>`. Preserves the original artifact's integrity while recording what changed and why.

**Anti-terms:** Do not edit a locked artifact directly — create an addendum instead.

**Related:** Artifact lock

---

## Review artifact

**Definition:** An artifact that records validation results against another artifact. Created with `wai add review --reviews <target>`. Includes an optional verdict (`pass`/`fail`/`needs-work`), severity counts, and the producing skill name. Used by pipeline procedural gates.

**Anti-terms:** Do not call it a "review document" or "audit" — "review artifact" is the canonical term.

**Related:** Artifact, Pipeline gate
