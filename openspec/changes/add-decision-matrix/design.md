# Design: Decision Matrix

## Directory Layout

One matrix per project, at a fixed location inside the existing designs
artifact directory:

```
.wai/projects/<project>/designs/matrix/
├── problem.md                     # A1: the decision being made (always in your face)
├── criteria/                      # rows: one definition file per criterion
│   ├── 01-impact.md
│   ├── 02-risk.md
│   └── 03-effort.md
├── approaches/                    # columns
│   ├── 01-status-quo/             # always first (convention + lint)
│   │   ├── _description.md        # `_` prefix = metadata, never a cell
│   │   ├── 01-impact/
│   │   │   ├── fact.md            # cell text: the aspect — facts, not judgment
│   │   │   └── yellow             # judgment: marker file, one of four
│   │   ├── 02-risk/
│   │   │   ├── fact.md
│   │   │   └── red
│   │   └── 03-effort/
│   │       └── fact.md
│   │       └── neutral            # neutral is a judgment, not "unassessed"
│   └── 02-event-sourcing/
│       ├── _description.md
│       └── ... (same criterion subdirs)
├── decision.md                    # current decision: selected approach + rationale + pointer
└── (matrix.html is generated on demand, never committed)
```

## Conventions

| Convention | Purpose |
|---|---|
| `problem.md` | A1: what decision are you trying to make? Required by `init`; rendered as the banner; re-anchored by editing it |
| Numeric prefixes (`01-`, `02-`) | Define row/column order; enforce status-quo-first |
| Criterion ID = filename | `02-risk.md` in `criteria/` matches `02-risk/` in every approach dir — the join key for rendering |
| `_` prefix | Metadata file (e.g., `_description.md`), skipped by the renderer |
| Fixed cell filenames | `fact.md` gives every cell a predictable path: `approaches/<a>/<c>/fact.md` |
| Marker file = judgment | Empty file named `neutral`/`green`/`yellow`/`red`. An encoding that **cannot be malformed**. Exactly one per filled cell |

## The Cell: Fact and Judgment Separated

Per the methodology: text is FACT, color is JUDGMENT, and judgment lives in
exactly one place.

- `fact.md` — plain markdown, no frontmatter. Says **how** the approach meets
  the criterion, not yes/no. Never contains the judgment.
- `neutral` / `green` / `yellow` / `red` — empty marker file.

**Cell completeness is defined by the fact, never the color:**

- No `fact.md` (or empty file) → **incomplete cell**. The renderer shows a
  gray "not yet assessed" placeholder — never a red chip, since red is a
  judgment and an unfinished cell has none.
- Non-empty `fact.md` + one marker → assessed (including `neutral`, which is
  the legitimate "just OK / clear" verdict from the methodology).
- Non-empty `fact.md` + zero or multiple markers → lint error.

Filling a cell is two trivial file operations. There is nothing to misalign.

## The API Is the Filesystem

Agents need **zero special tooling** for everyday editing:

| Operation | Filesystem action |
|---|---|
| Add approach | `cp -r approaches/01-status-quo approaches/03-cqrs`, rewrite `_description.md` |
| Fill a cell | Write one small file |
| Change judgment | Swap marker files (`rm red && touch yellow`) |
| Record a question | Write a line starting with `?` anywhere — `wai search "?"` finds it (methodology convention, zero implementation) |
| Record decision | `wai matrix decide` (multi-artifact operation → helper command) |

Git provides history, blame, and cell-sized diffs for free. No table merge
conflicts, ever — no cell is co-located with another.

## Commands

| Command | Behavior |
|---|---|
| `wai matrix init` | Scaffold `problem.md`, `criteria/`, `approaches/01-status-quo/`, `decision.md` template (does **not** count as decided — see Deciding) |
| `wai matrix criterion add <name>` | Add `criteria/NN-name.md` + cell dir in **every** approach (multi-op helper) |
| `wai matrix approach add <name>` | Add `approaches/NN-name/` with `_description.md` + empty cells for all criteria |
| `wai matrix decide <approach> <rationale>` | Write `decision.md` **and** scaffold a design doc in `designs/` (see Deciding) |
| `wai matrix lint` | Validate structure + methodology warnings (see Lints) |
| `wai matrix render` | Generate self-contained `matrix.html` for the human (see Rendering) |

Deliberately absent: `wai matrix set`. Filling a cell is writing one file and
touching one marker — sugar that hides trivial file I/O would only obscure the
model the human/agent pair is supposed to internalize. `init`, `criterion add`,
`approach add` exist only where one logical action spans many directories.

## Deciding

`wai matrix decide <approach> <rationale>`:

1. Validates the approach directory exists.
2. Writes `decision.md`: selected approach, rationale, **decision UTC
   timestamp**, pointer to the design doc. Minimal and machine-readable — the
   current-answer marker read by the phase gate and `wai status`. A `decision.md`
   that does not name an existing approach directory is **not decided** (this is
   what the freshly-scaffolded template produces, so a bare `init` never passes
   the phase gate).
3. Scaffolds a design doc in `designs/<date>-<slug>.md` with standard
   frontmatter (tags, `tracks:`) and a body template: rationale, trade-offs,
   link to the matrix directory, and a **decision-time snapshot** of the
   winning column's aspect texts. The human/agent fleshes out the narrative.

The snapshot matters because the matrix keeps growing: the design doc records
what was true *when decided*; the matrix shows *now*. Handoff links both.

## Renderer

Pure function from the directory tree → self-contained HTML (inline CSS, no
JS). Deterministic: same input, byte-identical output (property used by
snapshot tests, not a contract for committed artifacts — the output is never
committed).

1. `problem.md` rendered as the banner, above everything else.
2. `criteria/*.md` sorted → ordered rows; definitions become the legend.
3. `approaches/*/` sorted → ordered columns; `01-status-quo` asserted first.
4. Per (approach × criterion): non-empty `fact.md` → cell text; marker →
   chip (`neutral` = no chip). Missing/empty `fact.md` → gray "not yet
   assessed" placeholder — never a judgment color.
5. `decision.md` rendered below the banner, cross-checked against an actual
   approach directory.
6. Assessment key (the four colors and their meanings) baked in.

The HTML is a view. It is never edited and never committed; staleness is
irrelevant because it is always regenerable.

## Lints

**Structural — errors (non-zero exit):**

- Rectangularity: criterion subdirs identical across all approach dirs
- Exactly one marker per filled cell; markers empty; color ∈ the four values
- No empty `fact.md` files
- `01-status-quo` present as first column

**Methodology warnings — never block** (each maps 1:1 to a tip from the
source talk; they teach the methodology, they do not nanny):

- **All-green column** — "are you rationalizing?"
- **Undistinguished columns** — two approaches with identical cell texts
  across all criteria → likely missing a criterion
- **Judgment in text** — subjective words ("good", "bad", "better", "worse",
  ...) as the substance of `fact.md`; judgment belongs in the marker
- **Link-only cells** — a bare URL is "seeing nothing"; links may supplement
  text, not replace it
- **Criteria phrased as questions** — `?` must stay searchable for real open
  questions
- **Status quo without red** — Miller, 18:06: the status quo column must show
  what's wrong with today; no red cell in column 01 means something important
  was missed
- **Decided with unfilled cells** — a decision recorded while the chosen
  column has empty cells (warn; the methodology expects rough columns early,
  but deciding on blanks is shopping, not deliberating)
- **Stale decision** — see Phase gate below (mtime vs. decision timestamp;
  ties are ambiguous, so inconclusive staleness is a warning, never a false
  block, e.g. after a fresh clone)
- **Empty `problem.md`** — "what decision are you trying to make?"

## Workflow Integration

- **`wai status`** — during the design phase: show the current problem
  statement, completion count (`Matrix: 5/6 cells filled`), and suggest the
  next unfilled cell.
- **Phase gate `design → plan`:** requires the matrix to be **decided and
  current**: `decision.md` names an approach directory that exists, and no
  file under `approaches/` or `criteria/` is strictly newer than the decision
  timestamp recorded inside `decision.md` (content timestamp written by
  `decide` — mtimes are unreliable across git clone/rebase). A matrix edited
  after the last decision means deliberation resumed and is not finished.
  Failure messages are self-healing: point at `wai matrix decide` or the
  offending files; ambiguous-timestamp staleness downgrades to a warning,
  never a false block.
- **`wai close` handoff** — include the current problem statement, the
  selected approach with rationale, and a link to both the matrix directory
  and the design doc: context recovery gets the reasoning, not just the
  conclusion.
- **PARA** — the matrix lives under the project's `designs/` area,
  consistent with existing artifact organization; `wai search` finds its
  files with no new indexing.

## Example Session

```bash
wai matrix init                                  # prompted for problem.md (A1)
wai matrix approach add "event-sourcing"
wai matrix criterion add "operational-cost"      # rows appear as they become salient
# agent/human fill cells by writing files directly:
#   write approaches/01-status-quo/03-operational-cost/fact.md
#   touch approaches/01-status-quo/03-operational-cost/yellow
wai search "?"                                   # find open questions
wai matrix lint                                  # structural + methodology checks
wai matrix render && open designs/matrix/matrix.html   # at-a-glance view
wai matrix decide "02-event-sourcing" "Best audit story at acceptable ops cost."
# → decision.md written + designs/<date>-event-sourcing.md scaffolded
wai phase next                                   # gate passes: decision is current
```
