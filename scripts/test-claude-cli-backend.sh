#!/usr/bin/env bash
# Test that wai why works with the claude-cli backend.
# Run this OUTSIDE a Claude Code session (or it will fail with the nesting guard).
# Usage: ./scripts/test-claude-cli-backend.sh

set -euo pipefail

WAI="${WAI:-./target/debug/wai}"

if ! command -v claude &>/dev/null; then
  echo "SKIP: claude binary not found on PATH"
  exit 0
fi

echo "=== Building wai ==="
cargo build -q

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

echo "=== Setting up workspace ==="
"$WAI" --help >/dev/null  # sanity check

cd "$TMP"
"$WAI" init --name test-ws >/dev/null
"$WAI" new project myproj >/dev/null
"$WAI" add research "TOML was chosen for human-readability and broad language support." \
  --project myproj >/dev/null

echo "=== Configuring claude-cli backend ==="
# Write [why] config with claude-cli
cat >> .wai/config.toml << 'EOF'
[why]
llm = "claude-cli"
privacy_notice_shown = true
EOF

echo "=== Running: wai why 'why TOML' ==="
echo ""
"$WAI" why "why TOML"
echo ""
echo "=== PASS: claude-cli backend worked ==="
