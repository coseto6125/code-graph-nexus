#!/usr/bin/env bash
# tools/gen-cli-ref.sh
# Generate per-version CLI reference cards from `gnx --help`.
#
# Usage:
#   gen-cli-ref.sh [gnx-binary] [output-dir]
#
# Defaults:
#   gnx-binary   = ./target/release/gnx if it exists, else `gnx` from PATH
#   output-dir   = docs/skills/gnx-onboard/_shared/cli
#
# Output layout:
#   <output-dir>/<version>/<cmd>.md            (one per top-level + selected sub-commands)
#   <output-dir>/manifest.json                  {"latest": "<ver>", "versions": [...]}

set -euo pipefail

# Default gnx binary
default_gnx() {
    if [[ -x ./target/release/gnx ]]; then echo "./target/release/gnx"
    elif command -v gnx >/dev/null; then echo "gnx"
    else echo ""; fi
}

GNX="${1:-$(default_gnx)}"
OUT="${2:-docs/skills/gnx-onboard/_shared/cli}"

[[ -n "$GNX" ]] || { echo "gen-cli-ref: no gnx binary found" >&2; exit 1; }

# Version: "gnx 0.1.5" → 0.1.5  (also matches "gnx 9.9.9-test")
VER=$("$GNX" --version | awk '{print $2}')
[[ -n "$VER" ]] || { echo "gen-cli-ref: could not determine gnx version" >&2; exit 1; }

mkdir -p "$OUT/$VER"

# Commands to capture: top-level + curated sub-commands actually used in guides.
# When new guides reference a new command, add it here.
declare -a TOPLEVEL=(find impact inspect cypher routes coverage diff rename)
declare -a SUB=(
    "admin:index"
    "admin:group"
    "admin:mcp"
    "group:find"
    "group:contracts"
    "group:impact"
)

for cmd in "${TOPLEVEL[@]}"; do
    out="$OUT/$VER/$cmd.md"
    "$GNX" "$cmd" --help > "$out" 2>/dev/null || { echo "warn: $cmd has no --help; skipped" >&2; rm -f "$out"; }
done

for entry in "${SUB[@]}"; do
    parent="${entry%%:*}"
    child="${entry##*:}"
    out="$OUT/$VER/${parent}-${child}.md"
    "$GNX" "$parent" "$child" --help > "$out" 2>/dev/null || { echo "warn: $parent $child has no --help; skipped" >&2; rm -f "$out"; }
done

# Build/update manifest.json
manifest="$OUT/manifest.json"
if [[ -f "$manifest" ]]; then
    versions=$(jq -r --arg v "$VER" '(.versions // []) + [$v] | unique' "$manifest")
else
    versions=$(jq -n --arg v "$VER" '[$v]')
fi
jq -n \
    --arg v "$VER" \
    --argjson vs "$versions" \
    --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    '{latest: $v, versions: $vs, generated_at: $ts}' \
    > "$manifest"

echo "gen-cli-ref: wrote $OUT/$VER/ + manifest"
