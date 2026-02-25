## 1. Update skill name validation

- [x] 1.1 Update `validate_skill_name` in `src/commands/resource.rs` to accept exactly
        one `/`, with neither segment empty, starting/ending with hyphen, or containing
        invalid characters
- [x] 1.2 Add tests: `issue/gather` (valid), `impl/run` (valid), `a/b/c` (invalid),
        `/gather` (invalid), `issue/` (invalid), flat `my-skill` (unchanged valid)

## 2. Update skill storage paths

- [x] 2.1 Update `wai resource add skill` to pass the full name through `Path::join`,
        so `issue/gather` resolves to `skills/issue/gather/SKILL.md`
- [x] 2.2 Verify `create_dir_all` creates intermediate category directory correctly
- [x] 2.3 Add integration test: create `issue/gather`, verify file at correct path

## 3. Update skill listing

- [x] 3.1 Update `read_dir` scanning in `src/commands/resource.rs` to recurse one level
        into subdirectories (detect entries that are directories, scan their contents)
- [x] 3.2 Use the two-segment path (`category/name`) as the display name
- [x] 3.3 Update JSON output to include `category` field when present
- [x] 3.4 Add integration test: create `issue/gather` and flat `plain-skill`, verify
        both appear in list output with correct names
- [x] 3.5 Add integration test: create flat skill `issue`, then attempt to create
        `issue/gather`; verify the system errors with a conflict message

## 4. Documentation

- [x] 4.1 Update `wai resource add skill --help` to show the hierarchical name format
        and the one-slash rule
