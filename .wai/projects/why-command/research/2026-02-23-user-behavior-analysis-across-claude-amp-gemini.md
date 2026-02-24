User behavior analysis across Claude, Amp, Gemini, OpenCode: patterns and wai improvement opportunities

## Data Sources Analyzed

- Claude conversations: 576 JSONL files (~56 in wai project), 2161 history entries
- Amp threads: 282 threads (up to 473 messages each)
- Gemini sessions: ~15+ sessions (Dec 2025)
- OpenCode prompt history: 40 entries
- Goose LLM logs: 10 logs
- Global AGENTS.md: user workflow configuration

## User Behavior Patterns

### 1. Session Start Friction (HIGH IMPACT)

The most striking pattern: **context loss and re-orientation consume significant effort.**

Evidence:
- 233 /clear commands in Claude (fresh sessions constantly)
- 189 /rate-limit-options hits (forced breaks break flow)
- Common opening messages: 'work', 'start working', 'continue', 'what's the status?', 'how do we continue?'
- Heavy reliance on handoffs + resume skills to bridge sessions
- 'bd prime' concept in CLAUDE.md suggests user craves a single orientation command

The user wants to say 'work' and have the agent figure out everything from state. Currently requires: wai status + bd ready + openspec list + reading last handoff.

### 2. Tool Approval as Primary Interaction Mode

In Amp threads, the majority of user messages are:
- Empty strings (approve tool use)
- 'y', 'yes', 'ok' (confirmations)
- 'commit and push' (100 times in Claude history alone)
- 'fix all', 'continue'

This means **the user is in delegation mode, not direction mode.** They set a goal, then approve/steer rather than guide step-by-step. wai should support this: minimal friction to capture context, maximum autonomous behavior.

### 3. Multi-Tool Skill Duplication Pain

User manually manages skills across tools:
- Amp: 26 skills in ~/.config/amp/skills/
- Claude: commands in .claude/commands/ or .claude/skills/
- Gemini: separate config
- Global AGENTS.md covers goose/amp/claude/gemini but each tool needs tool-specific wiring

Evidence from conversations:
- 'add https://github.com/humanlayer/.../commit.md ... but addap them to this repo' (adapting same skill for each tool)
- 'add a .agents/commands to have them work across tools'  
- wai resource management (agent-config projections) was built to solve this but user still manually copies

### 4. Terse, Intent-First Communication

User messages are short and assume agent has context:
- 'fix all', '1 2 and 3', 'mvh', 'mkr' (abbreviations)
- Mixes Spanish/English fluidly
- Rarely repeats context - expects agent to read state from files
- Interrupts frequently ([Request interrupted by user for tool use]) - monitors but doesn't micromanage

### 5. Spec-Driven but Research-Heavy

User invests heavily in specs before coding:
- openspec for big changes
- wai for reasoning capture
- Frequently asks agents to 'research the codebase' before implementing
- 'design-practice', 'plan-review', 'research-review' are configured Amp skills

But **artifact capture is reactive, not proactive** - happens when explicitly prompted, not automatically.

### 6. Session End Neglect

The CLAUDE.md has a '🚨 SESSION CLOSE PROTOCOL 🚨' checklist but it's procedural. Evidence of incomplete captures:
- Many threads end with 'commit and push' without wai handoff
- Handoffs created irregularly (only 3 in wai .wai dir)
- 'wai add research' is often the agent's initiative, not user's

## Pain Points for wai Specifically

### A. Discoverability Gap (Already Partially Addressed)
User explicitly requested: 'wai init should create something in the repo to make llms know that they need to use it in the project'
- CLAUDE.md managed block: done
- AGENTS.md injection: done
- But other tools (Gemini ~/.gemini/, Amp ~/.config/amp/) don't get auto-configured

### B. Single-Command Session Orientation Missing
When user types 'work' they want to see:
1. What project is active + current phase
2. Last handoff summary
3. Next ready issues (bd ready output)
4. Active openspec changes

'wai status' gives partial answer but not integrated view.

### C. Artifact Capture Too Manual  
User runs sessions without capturing insights unless explicitly prompted. 
The AI stream journaling pattern (AGENTS.md) shows user values low-friction note capture.
wai could offer: end-of-session capture prompts, or a 'wai capture' that distills session into an artifact.

### D. Skill Projection Not Seamless
wai resources/agent-config/ has skills defined, but projection to Claude/Amp/Gemini requires manual steps. Should be: 'wai project sync' → updates all tool configs automatically.

### E. Cross-Project Status
User manages 10+ active projects across 5+ GitHub orgs. Switching projects requires re-learning state. wai status is per-repo; a global view would help.

## Improvement Opportunities for wai

### HIGH VALUE
1. **'wai prime' command** - Session orientation in one shot:
   - Current project + phase
   - Recent handoff summary (last 5 lines)  
   - bd ready output
   - Active openspec changes
   - Suggests next action
   
2. **Multi-tool projection on init** - wai init should detect and configure:
   - ~/.config/amp/settings.json → add wai skill
   - ~/.gemini/settings.json → add wai context
   - ~/.claude/CLAUDE.md (global) → awareness block
   - Not just per-repo CLAUDE.md
   
3. **'wai close' command** - Session end helper:
   - Auto-generates handoff
   - Reminds to run bd sync --from-main
   - Shows uncommitted files
   - One command instead of manual checklist

4. **Smarter 'wai status'** - Currently shows project/phase. Add:
   - Last handoff date + first line summary
   - Integration with bd (open issues count)
   - Integration with openspec (tasks % complete)
   - Single 'what's the situation' command

### MEDIUM VALUE
5. **'wai capture' command** - Distill a session summary:
   - Takes a description of what happened
   - Creates research artifact automatically
   - Suggests whether it's research/design/plan type

6. **Global wai view** - 'wai ls --all' showing all repos with .wai/, their phases, and open issues

7. **Handoff auto-suggest** - When wai detects uncommitted changes + end of work pattern, prompt: 'Should I create a handoff?'

### LOWER VALUE
8. **Stream capture** - Like the AI journaling pattern: quick 'wai note "thought"' that creates a lightweight artifact
9. **Tool health dashboard** - wai doctor shows all tool configs are in sync (Amp skills match Claude commands match wai resources)
