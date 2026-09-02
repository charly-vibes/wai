# Cartography — micro trace: `wai why` (network path)

**Entry zoom level:** micro (functionality trace)
**Scope covered:** `src/commands/why/{mod,context}.rs` + `src/llm.rs` — wai's only network I/O path

## Entry point

`pub fn run(query: String, no_llm: bool, json: bool, verbose: u8)` — `src/commands/why/mod.rs:438`

## Call chain

```
 1. why::run(query)                    commands/why/mod.rs:438
 2. json = json || current_context().json   [pure]
 3. no_llm? → search::run(SearchArgs)  commands/search.rs        [degraded path, no network]
 4. gather_context()                   why/context.rs:117
      read_artifacts + sort by mtime                              [I/O: fs walk]
      truncate_context (MAX_CONTEXT_CHARS)                        [pure]
      gather_git_file_context (file queries)                      [I/O: git subprocess]
      gather_meta / fetch_memories                                [I/O: fs]
 5. ProjectConfig::load → llm_config()                            [I/O: fs]
 6. detect_backend(&why_cfg)            llm.rs:448
      ClaudeCliClient   → std::process::Command::new("claude")    [llm.rs:225]
      OllamaClient      → Command::new("ollama")                  [llm.rs:314]
      ExternalApiClient → reqwest::blocking::Client               [llm.rs:134]  ← network
 7. privacy_notice_needed → show_privacy_notice (once)            [I/O: fs marker write, mod.rs:546]
 8. build_prompt(&ctx)                                            [pure]
 9. backend.complete(&prompt)                                     [I/O: network or subprocess — PURITY ENDS]
10. AGENT_SENTINEL short-circuit (agent session)                  [mod.rs:549]
11. parse_response()                    why/parsing.rs            [pure]
12. format_json / format_terminal; verbose stats (cost estimate llm.rs:522)
13. error path → llm_error_hint → fallback to search::run         [mod.rs:565-583]
```

## Data flow

```
query string
  → GatheredContext{ artifacts (recent-first, truncated), git_context, meta, memories }
  → prompt (built from context)
  → LLM backend (claude-cli subprocess | ollama subprocess | external API via reqwest::blocking)
  → raw_response
  → parsed answer (parsing.rs)
  → JSON envelope or terminal text (+cost/time stats on verbose)
fallback: any backend absence/failure → search::run over the same artifact store
```

## Structural observations

- Single purity boundary: steps 2–8 are fs/pure; step 9 is the only network/subprocess hop; parsing after it is pure again
- All I/O mechanisms live in `llm.rs` — `why/` never imports reqwest; `llm.rs` is the sole `reqwest` importer in `src/`
- Three distinct degradation ladders: no backend (→search), backend error (→search or error by `FallbackMode`), empty context (early exit)
- Backend selection is config-driven (`[llm]` in `.wai/config.toml`) with alias resolution (`resolve_model_alias`, `llm.rs:56`)
- Cost accounting is local estimation (`estimate_cost`, `llm.rs:522`) — no billing API calls

## Adjacent levels offered

Meso map of `why/` (`cartography-meso-why`) or micro trace of the agent-session sentinel path.
