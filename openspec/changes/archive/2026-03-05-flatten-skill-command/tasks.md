## 1. CLI: add `wai add skill` variant (`src/cli.rs`)

- [x] 1.1 Add `Skill { name: String, template: Option<String> }` variant to `AddCommands` enum
- [x] 1.2 Copy doc-comment and `#[arg]` attributes from `ResourceAddCommands::Skill`
  verbatim (name validation rules, template list, examples)

## 2. Command dispatch (`src/commands/mod.rs`, `src/commands/add.rs`)

- [x] 2.1 Add `AddCommands::Skill` arm in the `add` command dispatch — call the same
  handler as `ResourceAddCommands::Skill`
- [x] 2.2 Extract the skill-add handler into a shared function if it isn't already
  (avoids duplicating logic between the two dispatch sites)

## 3. Deprecation warning for `wai resource add skill` (`src/commands/resource.rs`)

- [x] 3.1 At the top of the `ResourceAddCommands::Skill` handler, emit:
  `eprintln!("⚠ 'wai resource add skill' is deprecated. Use: wai add skill <name>");`
- [x] 3.2 After the warning, delegate to the shared handler from task 2.2

## 4. Update `valid_patterns` in `src/commands/mod.rs`

- [x] 4.1 Add `("add", "skill")` to the `valid_patterns` list so typo detection
  recognises the new shape
- [x] 4.2 Keep `("resource", "add")` — the deprecated path must still be recognised

## 5. Update `wai init` managed block template

- [x] 5.1 Add `wai add skill <name>    # Scaffold a new agent skill` to the Quick
  Reference section, alongside the other `wai add` commands
- [x] 5.2 Verify the template renders correctly with `wai init --dry-run` (or
  equivalent test)

## 6. Tests

- [x] 6.1 Integration test: `wai add skill my-skill` creates
  `.wai/resources/skills/my-skill.md` (or appropriate path) — mirrors the
  existing `wai resource add skill` test
- [x] 6.2 Integration test: `wai add skill issue/gather --template gather` creates
  the file with template content
- [x] 6.3 Integration test: `wai resource add skill my-skill` still works and
  prints the deprecation warning to stderr
- [x] 6.4 Unit test: deprecation warning text matches the specified format exactly
