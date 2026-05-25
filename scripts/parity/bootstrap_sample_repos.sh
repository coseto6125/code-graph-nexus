#!/usr/bin/env bash
# Bootstrap Wave 1 language sample repos into .sample_repo/<lang>/
# Run from repo root: bash scripts/parity/bootstrap_sample_repos.sh
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SAMPLE_DIR="${REPO_ROOT}/.sample_repo"
mkdir -p "${SAMPLE_DIR}"

# Emit progress messages to stderr so stdout can be captured for stats
log() { echo "[bootstrap] $*" >&2; }

clone_if_missing() {
    local lang="$1"; shift
    local dest="${SAMPLE_DIR}/${lang}"
    if [[ -d "${dest}/.git" ]]; then
        log "${lang}: already cloned — skipping"
        return 0
    fi
    # URL is conventionally the last arg of each caller; the old `$1`
    # form after `shift` would print the first flag (e.g. `--depth`)
    # instead, so logs read "lua: cloning --depth ..." rather than the
    # actual repo URL.
    local url="${*: -1}"
    log "${lang}: cloning ${url} ..."
    git clone "$@" "${dest}"
    log "${lang}: done"
}

# ── Wave 1 repos ──────────────────────────────────────────────────────────────

mkdir -p "${SAMPLE_DIR}/C/deps" "${SAMPLE_DIR}/Cpp/tests/thirdparty"

JEMALLOC_DEST="${SAMPLE_DIR}/C/deps/jemalloc"
if [[ -d "${JEMALLOC_DEST}/.git" ]]; then
    log "C/deps/jemalloc: already cloned — skipping"
else
    log "C/deps/jemalloc: cloning jemalloc ..."
    git clone \
        --depth 1 \
        https://github.com/jemalloc/jemalloc.git \
        "${JEMALLOC_DEST}"
    log "C/deps/jemalloc: done"
fi

DOCTEST_DEST="${SAMPLE_DIR}/Cpp/tests/thirdparty/doctest"
if [[ -d "${DOCTEST_DEST}/.git" ]]; then
    log "Cpp/tests/thirdparty/doctest: already cloned — skipping"
else
    log "Cpp/tests/thirdparty/doctest: cloning doctest ..."
    git clone \
        --depth 1 \
        https://github.com/doctest/doctest.git \
        "${DOCTEST_DEST}"
    log "Cpp/tests/thirdparty/doctest: done"
fi

if [[ -f "${DOCTEST_DEST}/doctest/doctest.h" ]]; then
    cp "${DOCTEST_DEST}/doctest/doctest.h" "${DOCTEST_DEST}/doctest.h"
fi

# Pinned to the commit captured in baselines.md so re-clones reproduce the
# documented fixture instead of drifting with upstream master.

clone_if_missing lua \
    --filter=blob:none \
    --no-checkout \
    https://github.com/kikito/middleclass.git
git -C "${SAMPLE_DIR}/lua" checkout 359f0e2742f51ca77801b513ec91eb9029de8de4

clone_if_missing solidity \
    --filter=blob:none \
    --no-checkout \
    https://github.com/OpenZeppelin/openzeppelin-contracts.git
git -C "${SAMPLE_DIR}/solidity" checkout cd05883078060e0cd8a7bd36636944570dbe1722

clone_if_missing bash \
    --filter=blob:none \
    --no-checkout \
    https://github.com/Bash-it/bash-it.git
git -C "${SAMPLE_DIR}/bash" checkout 854c4ee02d033dbdd22183fb164ff84373c851aa

clone_if_missing zig \
    --filter=blob:none \
    --no-checkout \
    https://github.com/karlseguin/http.zig.git
git -C "${SAMPLE_DIR}/zig" checkout 569bba10f22afd4ea1815416b546a8065905f820

clone_if_missing crystal \
    --filter=blob:none \
    --no-checkout \
    https://github.com/kemalcr/kemal.git
git -C "${SAMPLE_DIR}/crystal" checkout 5023c21195cff9b0fca700bc582911b179c2add5

clone_if_missing dockerfile \
    --filter=blob:none \
    --no-checkout \
    https://github.com/docker-library/postgres.git
git -C "${SAMPLE_DIR}/dockerfile" checkout 2353f0380c24944616282f94544cec2d462e1e2a

# Move (aptos-core is huge — sparse checkout, only aptos-move/framework/)
MOVE_DEST="${SAMPLE_DIR}/move"
if [[ -d "${MOVE_DEST}/.git" ]]; then
    log "move: already cloned — skipping"
else
    log "move: sparse-cloning aptos-core (aptos-move/framework/ only) ..."
    git clone \
        --filter=blob:none \
        --sparse \
        --no-checkout \
        https://github.com/aptos-labs/aptos-core.git \
        "${MOVE_DEST}"
    git -C "${MOVE_DEST}" sparse-checkout set aptos-move/framework
    git -C "${MOVE_DEST}" checkout 47de220ebaa3e70bc5547911045971961395c233
    log "move: done"
fi

# ── Entry-point fixtures ──────────────────────────────────────────────────────
# The Wave 1 Go / Swift clones above are libraries (no `func main()` / `@main`),
# so `ecp cypher` returns 0 EntryPoint nodes for these langs and the
# `readme_verifier.py` (ext-scoping) drift report flags them as ☐ on cells the
# parser actually supports. These two clones add a Go binary + a Swift example
# repo with @main so the verifier sees real EntryPoint emissions.

# Go binary: mvdan/sh provides cmd/shfmt/main.go and cmd/gosh/main.go.
# Sparse checkout to the `cmd/` directory keeps disk footprint small.
GO_BIN_DEST="${SAMPLE_DIR}/Go-binary"
if [[ -d "${GO_BIN_DEST}/.git" ]]; then
    log "Go-binary: already cloned — skipping"
else
    log "Go-binary: sparse-cloning mvdan/sh (cmd/ only) ..."
    git clone \
        --depth 1 \
        --filter=blob:none \
        --sparse \
        https://github.com/mvdan/sh.git \
        "${GO_BIN_DEST}"
    git -C "${GO_BIN_DEST}" sparse-checkout set cmd
    log "Go-binary: done"
fi

# Swift CLI examples: apple/swift-argument-parser ships 8+ examples each with
# `@main`. Full clone (no sparse) because the entire Examples/ tree is small.
SWIFT_CLI_DEST="${SAMPLE_DIR}/Swift-cli"
if [[ -d "${SWIFT_CLI_DEST}/.git" ]]; then
    log "Swift-cli: already cloned — skipping"
else
    log "Swift-cli: cloning apple/swift-argument-parser ..."
    git clone \
        --depth 1 \
        https://github.com/apple/swift-argument-parser.git \
        "${SWIFT_CLI_DEST}"
    log "Swift-cli: done"
fi

# ── Disk usage summary ────────────────────────────────────────────────────────
log "Disk usage:"
du -sh "${SAMPLE_DIR}"/* 2>/dev/null || true
