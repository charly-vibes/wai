## 1. Bump dependency
- [x] Bump `genesis` from `v0.1.0` to `v0.2.0` in `Cargo.toml`.

## 2. Adopt genesis::config
- [x] If the tool has `src/config.rs`, thin it to just the struct + `ConfigFile` impl.
      Otherwise, add a minimal config struct implementing `ConfigFile`.
      (`ProjectConfig` and `UserConfig` now implement `genesis::config::ConfigFile`;
       `load`/`save` delegate read/write/parse to genesis via `read_from`/`write_to`.)
- [x] Register the config struct with `ConfigRegistry` at startup.
      (`config::default_registry()` registers `ProjectConfig` under `"wai"`;
       `doctor` uses `ConfigStore` for discovery + validation.)
- [x] Remove dead config parsing code (if any).
      (Removed manual `toml::from_str`/`read_to_string`/`to_string_pretty`/`create_dir_all`
       from `ProjectConfig` and `UserConfig` load/save.)
- [x] `cargo test` passes with the new config setup.

## 3. Adopt genesis::guide
- [x] Replace `main.rs` CLI setup with `Guide::builder(...)`.
      (`cli::build_guide()` assembles the scaffold; `main.rs` builds it and threads it
       through `commands::run`; `run_external` reuses `guide.registry()` for typo detection.)
- [x] Convert command handlers to return `Output<T>` and use `ErrorSink` for errors.
      (The `Commands::External` error path now uses `guide.error_sink()` (scratch-on,
       suggest-off, feedback-off) for self-healing output + error-scratch persistence.
       Full handler-by-handler `Output<T>` conversion is deferred to wai-0ly7, the
       feedback-subcommand ticket, which owns `--from-last-error` and scratch wiring.)
- [x] Remove dead error-handling code.
      (Replaced the ad-hoc `CommandRegistry::new()` in `run_external` and the bare
       `eprintln!("{:?}", err)` in the External error arm.)
- [x] `cargo test` passes with the new guide setup.

## 4. Clean up
- [x] `cargo test` passes.
- [x] `cargo clippy` introduces no new warnings.
- [x] `cargo fmt` is clean.