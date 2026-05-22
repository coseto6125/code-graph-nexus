# ecp Cold-Ingest Pipeline Optimization

**Date:** 2026-05-22
**Status:** Shipped — depends on `perf/ecp-multilayer-opt` (PR #333)
**Scope:** Cold reindex hot-path fixes; complements warm-query roadmap.

## 1. Context

The multi-layer warm-query PR (#333, `perf/ecp-multilayer-opt`) cut warm
queries from ~50ms → ~27ms on ecp self-scan. This roadmap targets the
cold reindex pipeline (`ecp admin index --repo <path> --force`).

**Benchmark target:** `.sample_repo` (14-lang polyglot, 16814 files,
262,194 nodes, 49 MB graph.bin). The eywa hook records a 2.7s baseline
(no embeddings) and 449ms warm reindex — this branch's instrumented
baseline matches: **3.13s on a fresh cache**.

ECP_PROF on a fresh L2 build:

| Phase | Time | Share |
|---|---|---|
| step1 scan files (16814) | 0.02s | 0.6% |
| step2 init_providers | 0.14s | 4.5% |
| **step3a parse_only** | **1.60s** | **51%** |
| step3b cache_puts (13912) | 0.34s | 11% |
| **step4 build_global_graph** | **0.62s** | **20%** |
| step5 write graph.bin (49 MB) | 0.05s | 1.6% |
| step6 tantivy index | 0.32s | 10% |
| orchestrator publish | 0.04s | 1% |
| **TOTAL** | **3.13s** | 100% |

Step 4 sub-breakdown (`total_build: 0.516s`):

| Sub-pass | Time |
|---|---|
| pass1_register | 0.119s |
| **pass3_community (Leiden)** | **0.190s** |
| pass2_imports_resolve | 0.036s |
| class_membership | 0.052s |
| function_meta | 0.024s |
| imports_edges | 0.022s |
| pass15_routes / pass16_fetch_shape / pass17_entry_points / pass18 / blind_spots / csr_assembly | 0.073s combined |

## 2. Pivot from original scope

The first draft of this roadmap targeted `add_graph` (par_iter), path_aliases
clone, and the file-node loop. **Real profile invalidates all three**:

- `add_graph` is `Vec::push` — 0.001s for all 1389 files.
- `parse_configs` is 0.040s on the full sample_repo.
- File-node loop lives inside `build` (0.575s) but is a small fraction.

A first commit (`ad54073b instrument(build): …`) landed phase-split prof
prints behind `ECP_PROF=1` so future investigation works against ground
truth instead of guesses.

## 3. Locked design decisions

- **Branched off `perf/ecp-multilayer-opt`**, not `main`. Cold-ingest
  touches `admin/index.rs` + `orchestrator.rs` which both share files with
  PR #333's commits. Rebase to main once #333 merges.
- **Target the deferrable phases first**, not the inherently-serial work.
  - `cache_puts` (0.34s) and `tantivy` (0.32s) can both run AFTER
    `graph.bin` is durable on disk and the orchestrator's `BuildResult`
    is returned to the caller. They don't block correctness of any query.
  - These two alone account for **21% of cold reindex wall time**.
- **No schema bump.** All changes are inside the build pipeline.
- **Detached threads, not async runtime.** The build orchestrator is
  sync code with no Tokio context. `std::thread::spawn` matches the
  existing `write_head_sha_sidecar_with_sha` pattern.

## 4. Status table

| # | Fix | Severity | Status | Commit | Evidence |
|---|---|---|---|---|---|
| **CI-INST** | ECP_PROF phase timings | tooling | shipped | ad54073b | orchestrator + step4 phase splits |
| CI-A | Defer cache_puts to background | 🔴 -11% | shipped | 4b706e5e | dedicated thread (NOT rayon) avoids pass2 contention |
| CI-B | Defer tantivy index to background | 🔴 -10% | shipped | (next) | dispatched in orchestrator AFTER rename — stale-path race resolved |
| CI-C | parse_only investigation | 🟡 51% | **deferred** | — | already rayon; sub-improvements need provider-level work |
| CI-D | pass3 community Leiden | 🟡 6% | **deferred** | — | algorithmic — graph community detection itself, not parallelism |

**Final benchmark (.sample_repo, 16814 files, 262k nodes, 3-run median):**

| Run | Time | Δ |
|---|---|---|
| Baseline (main) | 3.13s | — |
| After CI-A | 2.58s | -17.6% |
| After CI-A + CI-B | **2.28s** | **-27.2%** |

Tantivy index continues to be built in background after the user-visible
"l2.built" line. A `ecp find --mode bm25` query issued within ~300ms of
the rebuild will fall back to substring scan (already the documented
no-tantivy path), then ranked BM25 kicks back in once the background
finishes.

## 5. Deferred (with rationale)

- **Original C1 par_iter add_graph** — `add_graph` is 0.001s. Not worth.
- **Original C3 Arc path_aliases** — `parse_configs` 0.040s; cloning a
  small struct per-worker is bounded.
- **Original C7 file-node loop batch** — lives inside `build` 0.575s
  with multiple other passes; isolating is hard, gain is sub-ms.
- **Original C2 enclosing_class_heritage O(N_file²)** — Pass 2 wall is
  0.036s total. Fix when a single 200+ node file appears.
- **Original C4 post-process serial island** — class_membership 0.052s,
  overrides + schema_field_mirrors + event_topic_mirrors all sub-10ms.
  Parallel coordination cost likely exceeds savings.
- **Original C5 CSR sort 2x** — csr_assembly is 0.011s. Sub-PR concern.

## 6. Acceptance

- CI-A + CI-B ship in two commits; CI-C / CI-D land as separate commits
  if research yields actionable findings, else stay logged as future work.
- `cargo test --workspace --tests` green.
- `cargo clippy --workspace --tests` clean.
- Cold reindex benchmark on `.sample_repo`:
  - Target: total wall time ≤ 2.5s (from 3.13s) — ~20% reduction
  - User-visible "ready to query" time ≤ 2.5s (graph.bin durable)
- A "Things to highlight" section gets added at end-of-PR.

## 7. Things to highlight (post-implementation)

- **The original roadmap was wrong about where the time lived.** First-pass
  scope (C1 par_iter `add_graph`, C3 Arc `path_aliases`, C7 file-node loop)
  was based on misread profile output that confused step4 total (0.04s on
  the small ecp self-scan corpus) with the orchestrator's full publish
  wall. Once `.sample_repo` was instrumented, the real distribution
  emerged and the roadmap was rewritten in the next commit. Lesson:
  match the benchmark target to the real workload before scoping work —
  ecp self-scan (10k files) was misleading; `.sample_repo` (17k files,
  262k nodes) was the right target per the eywa hook's "14343 files, 2.7s
  baseline" reference.
- **CI-A's first draft used rayon for the background writes** and caused
  `pass2_imports_resolve` to jump from 36ms → 225ms (6×) due to global
  thread pool contention. The fix was a single dedicated `std::thread`
  doing sequential `atomic_write_bytes_no_fsync` — kernel-buffered writes
  saturate a single thread at disk speed without poaching CPU from the
  foreground. The commit body documents this trap so future maintainers
  don't "re-parallelize" the background writer.
- **CI-B's first draft caused a stale-path race**. Dispatching tantivy
  from inside `run_analyzer_for_paths` left the background thread writing
  to `building/` while the orchestrator renamed `building/ → publish_dir`
  underneath it. Tantivy failed with "Failed to acquire Lockfile … NotFound"
  and the `review_first_run_builds_v2_index_then_loads_it` test flaked.
  Fix: move the dispatch into the orchestrator, AFTER the rename returns.
  `run_analyzer_for_paths` now returns `(node_count, ZeroCopyGraph)` so the
  orchestrator owns the graph and can transfer it to the thread by move.
- **`run_analyzer_for_paths` signature change is internal**. The only
  call site is `build_inside_locked`; the test
  `build_orchestrator::tests::lock_is_released_on_drop` is `#[ignore]`'d
  per its module doc, so no fixtures needed updating.
- **CI-C parse_only stays deferred even though it's 51% of total**. The
  loop is already rayon-parallel; meaningful gains require provider-level
  work (e.g. mmap source bytes instead of `fs::read_to_string`, profile
  individual `Provider::parse_file` implementations). That's a multi-PR
  scope and the existing wall is already inside the project budget.
- **CI-D Leiden community detection stays deferred**. The 0.19s `pass3`
  wall comes from the graph-algorithmic work itself (community detection
  on 262k-node graph), not from a missing parallelism opportunity. The
  algorithm's intermediate state has cross-iteration dependencies that
  resist embarrassingly-parallel decomposition without changing the
  community-assignment semantics.
