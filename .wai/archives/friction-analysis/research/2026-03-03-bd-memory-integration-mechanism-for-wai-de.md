## bd Memory Integration Mechanism for wai

### Decision: Shell out to bd CLI

The existing plugin system in src/plugin.rs already establishes the pattern:
- execute_hook() shells out via std::process::Command and captures stdout
- The beads plugin already calls 'bd stats', 'bd list --status=open' this way
- Graceful degradation is built in: execute_hook returns None on failure or empty output

This is the right approach for memory integration too. No new mechanism needed.

### Read: fetch_memories()

Add a helper in src/plugin.rs (or a thin beads.rs module):

```rust
pub fn fetch_memories(project_root: &Path) -> Option<String> {
    // 1. Check beads plugin is detected (.beads/ exists)
    // 2. Run: bd memories
    // 3. Return stdout, or None if unavailable/empty
}
```

Shell command: `bd memories` (no args = all memories)
- Returns all memories; callers inject the full list and let the LLM judge relevance
- Filtered reads: `bd memories <keyword>` for search integration

### Write: store_memory()

```rust
pub fn store_memory(project_root: &Path, text: &str) -> Result<()> {
    // 1. Check beads detected
    // 2. Run: bd remember "<text>"
    // 3. Honour context.safe (refuse in safe mode)
    // 4. Return Ok or error
}
```

Write operations must check context.safe and context.no_input, same as execute_passthrough().

### Why NOT direct Dolt SQL

- Couples wai to bd's internal schema and port config
- Port varies per project (memory: dolt-port-conflict)
- bd CLI is the stable public interface

### Why NOT a new plugin API

- The HookDef / execute_hook pattern already handles exactly this use case
- Adding a new abstraction layer for two functions would be over-engineering

### Graceful Degradation Contract

All callers of fetch_memories() and store_memory() must:
- Treat None/Err as 'bd not available' — skip the section, do not fail
- Never surface a bd error to the user unless they explicitly invoked a bd operation
- Log at debug level if needed

### MEMORIES_BUDGET constant

Add to src/plugin.rs or a shared constants module:
```rust
pub const MEMORIES_BUDGET: usize = 10_000; // chars
```

Memories are short KV entries; 10K chars accommodates ~50-100 memories comfortably.

### Placement

Both helpers live in src/plugin.rs alongside execute_hook() — they follow the same shell-out pattern and are beads-specific plugin utilities. No new file needed.
