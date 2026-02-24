#!/usr/bin/env bash
# Test that wai why works with the claude-cli backend.
# Run this OUTSIDE a Claude Code session (or it will fail with the nesting guard).
# Usage: ./scripts/test-claude-cli-backend.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WAI="${WAI:-$REPO_ROOT/target/debug/wai}"

if ! command -v claude &>/dev/null; then
  echo "SKIP: claude binary not found on PATH"
  exit 0
fi

echo "=== Building wai ==="
cargo build -q --manifest-path "$REPO_ROOT/Cargo.toml"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

cd "$TMP"
"$WAI" init --name test-ws >/dev/null
"$WAI" new project myproj >/dev/null
"$WAI" add research "TOML was chosen for human-readability and broad language support." \
  --project myproj >/dev/null

echo "=== Configuring claude-cli backend ==="
# Replace the [why] section wai init writes (strip it and append a fresh one)
python3 -c "
import sys
content = open('.wai/config.toml').read()
base = content.split('[why]')[0].rstrip()
print(base + '\n[why]\nllm = \"claude-cli\"\nprivacy_notice_shown = true\n')
" > .wai/config.toml.tmp && mv .wai/config.toml.tmp .wai/config.toml

echo "=== Running: wai why 'why TOML' ==="
echo ""
"$WAI" why "why TOML"
echo ""
echo "=== PASS: claude-cli backend worked ==="
