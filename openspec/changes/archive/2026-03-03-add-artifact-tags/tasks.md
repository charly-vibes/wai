## 1. Add --tags to wai add plan and wai add design

- [x] 1.1 Add `tags: Option<String>` field to `AddCommands::Plan` in `src/cli.rs`
- [x] 1.2 Add `tags: Option<String>` field to `AddCommands::Design` in `src/cli.rs`
- [x] 1.3 Copy the frontmatter-writing block from the `Research` arm in
        `src/commands/add.rs` to both the `Plan` and `Design` arms
- [x] 1.4 Add unit tests for `wai add plan --tags` and `wai add design --tags`;
        verify frontmatter is present in the created file

## 2. Extend wai search with tag and latest filtering

- [x] 2.1 Add `--tag <value>` flag to `SearchCommands` in `src/cli.rs`
        (accept multiple uses or comma-separated)
- [x] 2.2 Add `--latest` bool flag to `SearchCommands` in `src/cli.rs`
- [x] 2.3 Update `src/commands/search.rs` to parse YAML frontmatter from each candidate
        file and apply tag filter before returning matches
- [x] 2.4 Implement `--latest`: after filtering, return only the candidate with the
        lexicographically greatest date prefix in its filename
- [x] 2.5 Confirm `--type` flag already handles `plan` and `design` types; fix if not
- [x] 2.6 Add tests for `--tag`, `--latest`, and combined `--tag --type --latest`
- [x] 2.7 Add test that malformed/absent frontmatter does not abort a `--tag` search

## 3. Documentation

- [x] 3.1 Update `wai add plan --help` and `wai add design --help` to document `--tags`
- [x] 3.2 Update `wai search --help` to document `--tag` and `--latest` flags
