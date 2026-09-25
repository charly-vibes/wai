# Decision Matrix

The decision matrix is wai's representation of the central artifact from the
[Design in Practice](https://www.infoq.com/presentations/design-in-practice/)
methodology (Rich Hickey, *"Design in Practice"*; Alex Miller,
*"Design in Practice in Practice"*). Where `wai add design` records the
*conclusion*, the matrix captures the *deliberation*: the approaches
considered, the criteria that mattered, the facts gathered, and the judgment
made on each intersection.

## One matrix per project

The matrix lives at a fixed location inside the project's design area:

```
.wai/projects/<project>/designs/matrix/
├── problem.md                     # A1: the decision being deliberated
├── criteria/                      # rows: one definition file per criterion
│   ├── 01-impact.md
│   └── 02-risk.md
├── approaches/                    # columns
│   ├── 01-status-quo/             # always first
│   │   ├── _description.md
│   │   ├── 01-impact/
│   │   │   ├── fact.md            # the aspect — facts, not judgment
│   │   │   └── red                # judgment marker
│   │   └── 02-risk/
│   │       └── fact.md            # no marker yet → not assessed
│   └── 02-event-sourcing/
│       └── ...
├── decision.md                    # current decision (written by `wai matrix decide`)
└── matrix.html                    # generated on demand, never committed
```

It is a **standing surface**, re-anchored by editing `problem.md` as new
decisions arise, and it grows over the project's lifetime — new approaches
are *born* in it, new criteria appear as deliberation makes them salient.

## A1: start with the problem

`problem.md` holds the answer to the methodology's first question — *what
decision are you trying to make?* It is rendered as the banner in
`matrix.html` and shown by `wai status` during the design phase. Re-anchor
the matrix for a new decision by rewriting it.

## The cell: fact and judgment separated

Each cell holds exactly two things:

- **`fact.md`** — the aspect: *how* the approach meets the criterion. Facts,
  not judgment. "Audit trail exists by construction", not "this is good".
- **One marker file** — empty, named `neutral`, `green`, `yellow`, or `red`.
  The judgment lives in exactly one place, and the encoding *cannot be
  malformed*: there is no syntax to break.

**Completeness is defined by the fact, never the color.** A cell with a
missing or empty `fact.md` is *not yet assessed* — the renderer shows a gray
placeholder, never a judgment color. `neutral` is a legitimate judgment
("clear, nothing special"), distinct from incomplete.

Filling a cell is two trivial file operations — this is the primary editing
path, for humans and agents alike:

```bash
# fill a cell
echo "Audit trail exists by construction." > approaches/02-event-sourcing/01-impact/fact.md
touch approaches/02-event-sourcing/01-impact/green
# change a judgment
rm approaches/02-event-sourcing/01-impact/green
touch approaches/02-event-sourcing/01-impact/yellow
```

The `?` convention: write a line starting with `?` anywhere in the matrix to
record an open question — `wai search "?"` finds them. Keep criteria
*statements*; questions stay searchable.

## Commands

The CLI exists only where one logical action spans many directories:

| Command | Behavior |
|---|---|
| `wai matrix init "<problem>"` | Scaffold `problem.md`, `criteria/`, `approaches/01-status-quo/`, `decision.md` template |
| `wai matrix criterion add <name>` | Add `criteria/NN-name.md` + a cell dir in **every** approach |
| `wai matrix approach add <name>` | Add `approaches/NN-name/` with `_description.md` + empty cells for all criteria |
| `wai matrix decide <approach> "<rationale>"` | Write `decision.md` + scaffold a design doc with a decision-time snapshot |
| `wai matrix lint` | Structural errors + methodology warnings |
| `wai matrix render` | Generate self-contained `matrix.html` |

There is deliberately **no `wai matrix set`**: filling a cell is writing one
file and touching one marker — sugar over trivial file I/O would only obscure
the model.

## Linting: structure is enforced, methodology is taught

`wai matrix lint` reports structural **errors** (non-zero exit):
rectangularity, exactly one known marker per filled cell, no empty `fact.md`,
status-quo-first.

Methodology **warnings never block** — each maps 1:1 to a tip from the source
talks: all-green column ("are you rationalizing?"), undistinguished columns
(likely a missing criterion), judgment-in-text, link-only cells, criteria
phrased as questions, status quo with no red cell (Miller: *show what's wrong
with today*), deciding with unfilled cells, empty `problem.md`, and stale
decisions (matrix edited after deciding).

## The design → plan gate

Leaving the design phase requires the matrix to be **decided and current**:
`decision.md` names an existing approach, and no file under `approaches/` or
`criteria/` is newer than the decision timestamp recorded *inside*
`decision.md` (content timestamps survive clone/rebase; mtimes don't).
Edited-after-decision blocks the transition with a self-healing message;
ambiguous timestamps (e.g. a fresh clone, where everything shares the
checkout time) downgrade to a warning, never a false block. Projects without
a matrix are never gated.

## Deciding

`wai matrix decide "02-event-sourcing" "Best audit story at acceptable ops
cost."` does two things:

1. Writes `decision.md` — approach, rationale, UTC timestamp, design-doc
   pointer. A scaffolded `decision.md` names no approach, so a bare `init`
   never passes the gate.
2. Scaffolds `designs/<date>-<slug>.md` with standard frontmatter and a
   **decision-time snapshot** of the winning column's facts. The matrix keeps
   growing; the design doc records what was true *when decided*.

The human/agent then fleshes out the design doc's narrative — rationale,
trade-offs — with the matrix one `wai search` away.

## Adoption

The matrix is **opt-in**: nothing exists until `wai matrix init`. Projects
that don't use it behave exactly as before — no gating, no status output, no
handoff sections.
