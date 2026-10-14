# devenv.sh — single source of truth for wai's toolchain (spike for DDL-irv).
# Verify: devenv shell -- just gates
# Versions below mirror versions.ddl.toml; that file remains the source of
# truth — update both in the same commit.
{ pkgs, ... }: {
  packages = with pkgs; [
    just # task runner (justfile is the QA entry point)
    nodejs # host for openspec (@fission-ai/openspec, npm)
  ];

  # Fleet tools NOT duplicated here (per DDL-irv §8.1): pretender 0.7.0,
  # dulce-de-leche 0.7.0, bd 1.3.0 are crates.io/GitHub-release pins that
  # nixpkgs does not track at pinned versions — ddl provisions them
  # (cargo install / pinned release download). Revisit if the granularity
  # question (per-repo devenv vs one fleet flake in ddl) produces real
  # cross-repo pin duplication pain.
}
