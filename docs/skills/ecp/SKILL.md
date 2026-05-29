---
name: ecp
description: Use for symbol-level code analysis, blast-radius impact, cross-repo API contracts, AST-aware rename, route map. Defer to grep for string literals, config keys, vendored / generated code, and fs layout.
---

# EgentCodePlexus (ecp) — Structural Analysis Entry

The entry point for structural code analysis, impact assessment, and cross-repo contract verification. Directives below = when to reach for ecp and when to distrust its answer; Quick Reference = which command for which task.

---

## 🧭 Layer 1: Core Principles

The *what* (which command for which task) lives in @ECP.md and the Quick Reference below. These directives cover the part that's harder to get right: **when to reach for ecp, and when to distrust its answer.**

### Directive 1: ecp-first reflex (full rule in @ECP.md §"The reflex")
Code-structure queries go to ecp before grep or an Explore agent — definitions, callers, blast radius, traces, "fan out and read these files." The only open question here is *which verb*: `find` for definitions, `impact` for callers / blast radius, `inspect` for full context, `routes` / `contracts` for API surfaces, `cypher` for anything else. This holds for ecp's own codebase too.

### Directive 2: Blast Radius before Refactor — and it's a lower bound
Before modifying a function or class, run `ecp impact` to see who calls it (HIGH / CRITICAL risk → confirm with the user). The caller set is a **lower bound**: a bare call to a common name can be suppressed by the resolver's ambiguity cap. A suspiciously low caller count → `grep` the call sites to cross-check before trusting it.

### Directive 3: `found:false` is two-valued — read the `result` field
ecp auto-refreshes the index; you rarely run `ecp admin index` manually. But a `found:false` can mean "doesn't exist" OR "graph is a warm-attach, HEAD not yet indexed, symbol not indexed yet". **Tell:** a `result` field in the payload or an `l2.warm-attach` / `note:` line on stderr → the answer is provisional. Rerun or `ecp admin index --force --repo .` before concluding it's gone. For genuine misses, try `ecp find <fragment> --mode fuzzy`. See [`guides/troubleshooting.md`](./guides/troubleshooting.md).

### Directive 4: Surprising output has a root cause; and grep is right for text
Before concluding "ecp is broken", verify against source (read the definition, fresh reindex, grep cross-check) — doc-comment inference ≠ verification. And use grep / Read for genuinely non-code text: string literals, error messages, config keys, vendored / generated code, fs layout. ecp parses code, not text.

---

## ⚡ Quick Reference (command × use-case)

### Symbol lookup
| Command | Use for |
|---|---|
| `ecp find <name>` | Exact symbol match (default) |
| `ecp find <n> --mode fuzzy` | Substring match for partial names |
| `ecp find <n> --mode bm25` | BM25-ranked, bucketed top-K |
| `ecp find <n> --kind function,method` | Filter by symbol kind |
| `ecp inspect --name <n>` | Full context: signature + body + edges + callers |

### Impact / blast radius

`ecp impact` has three **mutually exclusive** modes — pick by what you have:

**Symbol mode** (you know the symbol name):
| Command | Use for |
|---|---|
| `ecp impact <name>` | Upstream callers + risk_level (default depth 5, direction `up`) |
| `ecp impact <n> --direction down --depth N` | Custom traversal (`up` / `down` / `both`) |

**Baseline mode** (no symbol — derive from git diff):
| Command | Use for |
|---|---|
| `ecp impact --baseline origin/main` | All symbols changed between baseline and HEAD |
| `ecp review --baseline origin/main` | Post-edit audit: impact + route drift + egress, in one pass (the flag is easy to miss) |

**Literal mode** (path-string sink lookup):
| Command | Use for |
|---|---|
| `ecp impact --literal session_meta.json` | Exact read/write sites for that path string, classified (`sink:read` / `sink:write` / `sink:join` / `sink:free` / …). For split-brain bugs, query each suspected literal separately (`meta.json`, `session_meta.json`) |
| `ecp impact --literal-coherence` | Auto-detect likely filename split-brain pairs across all PathLiteral nodes (similar names, same extension, nearby dirs, read-only vs write-only) |

**Related (edge-level, not symbol-level)**:
| Command | Use for |
|---|---|
| `ecp diff` | Edge-level resolver delta (binding tier-degradation, route / contract changes) |

### Architecture / cross-cutting
| Command | Use for |
|---|---|
| `ecp summary` | Repo health + frameworks + blind spots |
| `ecp routes <path>` | HTTP route → handler + caller chain |
| `ecp contracts` | Cross-repo API contracts |
| `ecp tool-map` | External HTTP / DB / Redis / queue calls |
| `ecp shape-check` | HTTP consumer ↔ Route response shape drift |
| `ecp processes` | List execution-flow Process nodes (Leiden community + BFS detection at index time) |
| `ecp processes trace <pat>` | Dump full Function / Method step sequence for a matching Process — cleaner than `impact --direction down` when you want the actual execution order |
| `ecp review` | Full audit (impact + summary + tool-map + shape-check + diff) |
| `ecp rename <old> <new>` | AST-aware multi-file rename |
| `ecp admin doctor [check] [--fix]` | Environment health check (skills / index / host / config / registry / version); `--fix` repairs fixable items |

### Multi-repo / groups (cross-repo scope — single-repo flows don't use these)
Run in order: `sync` → `contracts` → `impact`.
| Command | Use for |
|---|---|
| `ecp group sync <name>` | Build cross-links + extract contracts for the group |
| `ecp group status <name>` | Check staleness of group members |
| `ecp group contracts <name> [--unmatched]` | Inspect contract registry; `--unmatched` finds orphaned consumers |
| `ecp group impact <name> --target <symbol> --repo <provider>` | Cross-repo blast radius — which other repos call this symbol |
| `ecp group find <name>` | Search across all group members |
| `ecp contracts --repo @all` | Registry-wide contract view (no group needed) |

### Cypher escape hatch
| Command | Use for |
|---|---|
| `ecp cypher "<query>"` | Ad-hoc `MATCH ... RETURN ...` when no command fits |

### Schema introspection (graph-loadless)
| Command | Output |
|---|---|
| `ecp schema blindspots` | Per-lang BlindSpot coverage; disambiguates "no dispatch in diff" vs "parser doesn't detect it" |
| `ecp schema reltypes` | All 20 RelType edges + LLM-utility category + heuristic flag |
| `ecp schema node-kinds` | All 29 NodeKind variants + same-name distinctions (Struct vs Class, Trait vs Interface) |
| `ecp schema graph-version` | rkyv `graph.bin` format version + bump history |

All `schema` commands default to `--format json` (agent-consumable); pass `--format text` for a human table.

---

## 📚 On-Demand References

- [`guides/troubleshooting.md`](./guides/troubleshooting.md) — `found:false`, index staleness, resolver misses, and the four output-trust tells.
- `_shared/cli/` — Per-command flag references (`inspect`, `impact`, `cypher`, `group`, `processes`, …).
- `_shared/refs/` — Conceptual background (Cypher syntax, repo resolution).
