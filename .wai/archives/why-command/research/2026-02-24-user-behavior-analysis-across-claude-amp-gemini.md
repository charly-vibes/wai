User behavior analysis across Claude, Amp, Gemini, OpenCode: patterns and wai improvement opportunities

## Data Sources Analyzed

- Claude conversations: 576 JSONL files, 2161 history entries
- Amp threads: 282 threads
- Gemini sessions: Dec 2025
- OpenCode prompt history: 40 entries
- Global AGENTS.md user workflow config

## Key Behavior Patterns

### 1. Session Start Friction
The most striking pattern: context loss and re-orientation consume significant effort.
- 233 /clear commands (fresh sessions constantly)
- 189 /rate-limit-options hits (forced breaks break flow)
- Common opening messages: 'work', 'start working', 'continue', 'what is the status?'
- Heavy reliance on handoffs + resume skills to bridge sessions
- User wants to say 'work' and have agent figure out everything from state

### 2. Delegation Mode
Majority of user messages are empty (approve tool) or 'y', 'ok', 'commit and push'.
'commit and push' used 100 times in Claude alone.
User sets goal then approves/steers, not micromanages.
wai should minimize friction for context capture, maximize autonomous behavior.

### 3. Multi-Tool Skill Duplication Pain
Amp: 26 skills, Claude: separate commands, Gemini: separate config.
User manually adapts same skills for each tool (seen in conversations).
wai resource management/projections built to solve this but not seamlessly automated.

### 4. Terse Intent-First Style
Short messages, assumes agent reads state from files.
Mixes Spanish/English. Uses abbreviations like 'mvh', 'mkr'.
Interrupts frequently to monitor, rarely repeats context.

### 5. Artifact Capture is Reactive
wai add research happens when agent suggests it, not user-initiated.
Session end neglect: many threads end 'commit and push' with no handoff.
Only 3 handoffs in wai .wai dir for months of work.

## Pain Points

A. Multi-tool discoverability: CLAUDE.md block done, but Gemini/Amp configs not auto-updated by wai init
B. No single session orientation command: wai status + bd ready + openspec list = 3 commands
C. Artifact capture too manual: session insights lost without explicit wai add
D. Skill projection requires manual steps: wai resources/agent-config not synced to tools automatically
E. No cross-project global view: user manages 10+ active projects

## Improvement Opportunities

HIGH VALUE:
1. 'wai prime' - single session orientation command showing: project/phase, last handoff summary, bd ready, openspec status, suggested next action
2. Multi-tool projection on init: detect and configure Amp, Gemini, Claude global, not just per-repo CLAUDE.md
3. 'wai close' - session end helper: auto-generates handoff, reminds about bd sync, shows uncommitted files
4. Smarter 'wai status' with last handoff summary, bd open issues, openspec % complete

MEDIUM VALUE:
5. 'wai capture' - distill session summary into artifact automatically
6. 'wai ls --all' global view of all repos with .wai/
7. Auto-suggest handoff when uncommitted changes detected at session end

LOWER VALUE:
8. Stream capture for quick lightweight artifacts
9. Tool health dashboard showing all tool configs in sync
