# Competitive landscape

[繁體中文 (Traditional Chinese)](./readme_i18n/competitive-landscape_zh-TW.md)

Last surveyed: 2026-05-22.

The "Rust + tree-sitter + MCP + code-intelligence-graph-for-LLM-agents" space is
extremely crowded as of mid-2026 — ≥15 similar projects pushed in the last
3 months alone. This document captures where `ecp` (egent-code-plexus) sits
relative to the field, where it overlaps, and where its real differentiation
lies. Updated when significant new entrants appear or existing ones shift focus.

## Closest matches (Rust-native, graph-first, MCP-wired)

| Project | Stars | License | Overlap with ecp | Where it diverges |
|---|---|---|---|---|
| [onur-gokyildiz-bhi/codescope](https://github.com/onur-gokyildiz-bhi/codescope) | 21 | MIT | Rust-native, MCP, "graph-first not embeddings-first", ms-level traversal, 57 languages, 9 agent integrations, rkyv in Cargo.lock | SurrealDB primary backend, LSP mode + Web UI + daemon, no Process abstraction, no community detection |
| [cmillstead/codesight-mcp](https://github.com/cmillstead/codesight-mcp) | — | — | 66 languages via tree-sitter, 34 MCP tools, impact analysis | Focus on retrieval not graph-algorithm layer |
| [postrv/narsil-mcp](https://github.com/postrv/narsil-mcp) | — | — | 32 langs, 90 MCP tools, call graph, security scanning | No community / Process abstraction |
| [basidiocarp/rhizome](https://github.com/basidiocarp/rhizome) | — | — | Tree-sitter + LSP dual backend, sub-ms parse | Closer to LSP wrapper than graph storage |
| [kuberstar/qartez-mcp](https://github.com/kuberstar/qartez-mcp) | — | — | 37 langs (tree-sitter + regex fallback), project map, symbol search, impact analysis | No community / Process abstraction |
| [greysquirr3l/coraline](https://github.com/greysquirr3l/coraline) | 10 | Apache-2.0 | 28 langs, MCP, claims sub-second indexing | SQLite backend, no community detection |
| [shaharia-lab/code-navigator](https://github.com/shaharia-lab/code-navigator) | 5 | MIT | "Compressed graph" for AI agents, impact analysis | Very early stage |
| [Jakedismo/codegraph-rust](https://github.com/Jakedismo/codegraph-rust) | 754 | unclear (README claims MIT/Apache, no LICENSE file) | 14 crates, Rust + MCP | 5 months stale (last push 2025-12-20), SurrealDB, focuses on agent framework (Rig + LATS + Reflexion), no community detection |

## Adjacent (overlap but different positioning)

| Project | What it does |
|---|---|
| [github/stack-graphs](https://github.com/github/stack-graphs) | 877★. GitHub's official Rust tree-sitter cross-file symbol resolver. Resolves cross-references only — no community detection or Process abstraction. |
| [probelabs/probe](https://github.com/probelabs/probe) | ripgrep-speed + tree-sitter AST, semantic search. No graph storage layer. |
| [faxioman/code-sage](https://github.com/faxioman/code-sage) | BM25 + vector + tree-sitter chunking. Semantic search, not graph. |
| [flupkede/codesearch](https://github.com/flupkede/codesearch) | Hybrid vector + BM25 + tree-sitter chunking. |
| [rustkit-ai/semtree](https://github.com/rustkit-ai/semtree) | Tree-sitter + embeddings + RAG multi-backend. |
| [hankh95/nusy-codegraph](https://github.com/hankh95/nusy-codegraph) | Arrow-native code object storage. Different storage angle worth tracking. |

## What is unique to ecp

The field largely converged on the same baseline stack: **Rust + tree-sitter +
MCP + sub-second-indexing-claim**. What separates ecp:

| Field commodity | ecp's specific bet |
|---|---|
| Tree-sitter parse across 28-66 languages | **Leiden community detection → `NodeKind::Process` semantic abstraction**. Surfaces "execution flows" to the LLM, not just callee / caller. Nobody else in the surveyed field does this. |
| Impact analysis (callers / callees) | **Deterministic seeded output**. Same corpus + same seed → bit-identical `graph.bin`. Pinned via `LeidenConfig::default().seed = 0xc0de` and XorShift64 RNG. Enables reproducible A/B oracle comparison vs `ref-gitnexus`. |
| MCP tool packaging | **Zero-copy rkyv mmap as primary store**. Codescope uses rkyv too, but as a transitive dep on top of SurrealDB. We use it as the canonical on-disk format — no DB engine in the query path. |
| BM25 / vector hybrid | **Cypher subset** for graph queries. Few in this field expose graph-query semantics; most expose JSON tool-call wrappers. |

## What we should not copy

Decisions in the surveyed field that would push ecp's per-query latency or
cold-ingest target in the wrong direction:

- **SurrealDB / SQLite as primary backend** (codescope, Jakedismo, coraline) —
  query path goes through a DB engine. Conflicts with our <30 ms / query and
  <5 s cold-ingest targets.
- **AI / RAG / embedding pipeline integrated into core** (Jakedismo, semtree,
  code-sage) — couples ecp to LLM call latency and vendor APIs. Our core value
  is deterministic, not fuzzy.
- **LSP as default backend** (rhizome) — LSP cold-start (3-10 s per server)
  alone exceeds our cold-ingest target. LSP can only ever be an opt-in
  enrichment layer.
- **Massive MCP tool count** (narsil 90 tools, codesight 34, codescope 32) —
  tool count ≠ tool quality. Every tool is a documented contract the consuming
  LLM has to read; surface area should grow only with demonstrated demand.

## What we could borrow (small, specific, opt-in)

| From | Idea | Cost |
|---|---|---|
| codescope | `codescope insight` style per-repo + hourly MCP tool usage telemetry — gives the user visibility into which tools agents actually call | Low (pure telemetry, no algorithm change) |
| codescope, Jakedismo | LSP bridge **as an opt-in feature** — for cases tree-sitter can't resolve (C++ templates, Java generics) | Medium. Must be feature-gated so default path keeps cold-ingest budget. |
| nusy-codegraph | Arrow-native storage angle — pair-equivalent of rkyv but with cross-language ecosystem reach (Python pandas can mmap-read directly). Relevant only if Python wheel binding becomes a user demand. | High; defer. |
| codescope, coraline | Public sub-second-indexing benchmark **as standard comparison** — same corpus, side-by-side numbers. | Low engineering, moderate marketing follow-through. |

## Implications for roadmap

The differentiation lives in **the algorithm layer producing semantic
abstractions** (Leiden → Process), not in tool count, language count, or
agent-integration breadth. Continuing to deepen that axis has higher ROI than
chasing parity on LSP / embedding / agent-framework features that are already
table-stakes in the field.
