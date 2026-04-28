## Context

The user wants `wai way` to recommend a ubiquitous-language definition workflow for AI-assisted development. The repository already has patterns for:

- canonical resources under `.wai/resources/`
- progressive disclosure for agent skills
- managed-block guidance that points agents at reusable knowledge before they start work

The design challenge is to preserve those patterns while avoiding a single glossary file large enough to consume valuable context window space.

## Goals / Non-Goals

### Goals
- Define a canonical home for ubiquitous-language artifacts inside `.wai/resources/`
- Keep the entrypoint small so agents can load terminology progressively
- Provide a built-in skill template that helps maintain the resource tree
- Surface the practice through `wai way` and the managed block

### Non-Goals
- Automatic extraction of terminology from code or conversations
- Semantic linting or CI enforcement in this change
- A new top-level command dedicated to ubiquitous language management

## Decisions

### Decision: Use a directory tree, not a single file

The canonical resource lives at `.wai/resources/ubiquitous-language/`, not at a single `ubiquitous-language.md` file.

Rationale:
- matches PARA resource organization
- avoids large always-loaded glossary files
- lets agents load a tiny index first, then only the bounded context they need

### Decision: Root index + shared terms + bounded-context files

The structure is intentionally simple:

```text
.wai/resources/ubiquitous-language/
├── README.md
├── shared.md
└── contexts/
    ├── <context>.md
    └── ...
```

- `README.md` is the navigation entrypoint and loading guide
- `shared.md` is reserved for truly cross-context terms only
- `contexts/*.md` hold detailed term definitions per bounded context

This keeps global context lean and pushes detail to the leaf files.

### Decision: Managed block references the directory, not every file

The managed block should not inline terminology. It should tell agents to:
1. inspect `.wai/resources/ubiquitous-language/`
2. read `README.md` first as the navigation entrypoint
3. open only the relevant bounded-context files before introducing or reusing terms
4. avoid loading every terminology file unless the task truly spans multiple bounded contexts

That preserves progressive disclosure and keeps the managed block concise.

### Decision: Skill template focuses on curation, not generation without review

The `ubiquitous-language` skill template should guide agents to:
- search existing artifacts and code
- identify candidate terms and contexts
- propose additions or normalizations
- update the right context files incrementally
- record anti-terms and synonyms

The template is a workflow scaffold, not an authority to invent definitions without user validation.

## Alternatives considered

### Single markdown file

Rejected because it is easy to create but encourages context bloat and mixes unrelated bounded contexts together.

### One file per term

Rejected for V1 because it would create excessive file fragmentation and make navigation harder than necessary.

### Root-level `UBIQUITOUS_LANGUAGE.md`

Rejected because wai already treats `.wai/resources/` as the canonical home for reusable project knowledge.

## Risks / Trade-offs

- Too little structure could lead to inconsistent term files
  - Mitigation: define canonical file roles and term-entry expectations in the spec
- Too much structure could make the feature feel heavyweight
  - Mitigation: keep V1 to three levels: index, shared, contexts
- Managed-block guidance could become noisy
  - Mitigation: mention the directory only when it exists and keep the instruction short

## Migration Plan

1. Add the new spec capability and cross-spec deltas
2. Implement the `wai way` check
3. Add the skill template and help text updates
4. Update managed block generation to reference the resource tree when present

## Open Questions

- None for the proposal stage; the requested structure and integration points are defined here.
