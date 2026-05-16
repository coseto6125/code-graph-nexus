//! `Imports` edge emission — `File` node → imported target.
//!
//! Walks each `LocalGraph.imports` (already populated by the 14 language
//! parsers), resolves each `RawImport` via the same `Resolver` used for
//! Calls/Accesses, and emits one `Imports` edge per
//! `(file_node, resolved_target)` pair.
//!
//! Spec: `docs/superpowers/specs/2026-05-17-imports-edge-emission.md` §2 —
//! resolver miss → don't emit (refuse to produce gitnexus-style cross-language
//! false positives like `.mjs → Path.java`).
//!
//! Resolution strategy (three steps, first-hit wins):
//!
//! 1. **Named-symbol lookup** — try `Resolver::resolve_symbol` with both
//!    `Callable` and `Type` kinds. Covers TS/JS/Python/Java/PHP/Rust where
//!    `RawImport.imported_name` already names the imported symbol
//!    (`from a import foo`, `import { foo } from './a'`, etc.).
//!
//! 2. **FQN last-segment** — if `imported_name` contains `.` (e.g. Kotlin
//!    `import com.x.Alpha` surfaces with `imported_name = "com.x.Alpha"`),
//!    retry resolve with the last dotted segment (`"Alpha"`).
//!
//! 3. **Module-style fallback** — if Steps 1+2 miss, treat the import as
//!    pointing at a whole file (Ruby `require_relative 'alpha'`, Go
//!    `import "x/pkg"`, C `#include "alpha.h"`). Use
//!    `Resolver::enumerate_candidates` to walk the same path-expansion
//!    rules Tier 2 uses, then emit a `File → File` edge to the first
//!    candidate that resolves to a known file. Confidence 0.9 (vs 1.0
//!    for named imports — fallback is slightly less specific).

use crate::resolution::index::ResolveTarget;
use crate::resolution::resolver::Resolver;
use graph_nexus_core::analyzer::types::LocalGraph;
use graph_nexus_core::graph::{Edge, RelType};
use graph_nexus_core::pool::StringPool;
use rustc_hash::{FxHashMap, FxHashSet};

/// Emit `Imports` edges from each `File` node to the resolved targets of
/// every `RawImport` in that file. Returns the number of edges appended.
pub fn emit_edges(
    local_graphs: &[LocalGraph],
    resolver: &Resolver<'_>,
    file_node_idx: &FxHashMap<String, u32>,
    string_pool: &mut StringPool,
    edges_out: &mut Vec<Edge>,
) -> usize {
    let reason_named = string_pool.add("post_process:imports");
    let reason_module = string_pool.add("post_process:imports:module");
    let mut emitted = 0usize;
    let mut dedupe: FxHashSet<(u32, u32)> = FxHashSet::default();

    // Pre-pass: build a basename → [(full_path, idx)] index so the
    // Step 3c/3d/3e fallbacks can do O(1) hash lookup + bucket-local
    // suffix filter instead of an O(N) linear scan across `file_node_idx`
    // per import. On `.sample_repo` (14k files) this shrinks Step 3 cost
    // from O(imports × files) ≈ 200M comparisons to O(imports × bucket)
    // where bucket size is typically < 10.
    let mut basename_idx: FxHashMap<&str, Vec<(&str, u32)>> = FxHashMap::default();
    for (path, &idx) in file_node_idx.iter() {
        let basename = std::path::Path::new(path.as_str())
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(path.as_str());
        basename_idx
            .entry(basename)
            .or_default()
            .push((path.as_str(), idx));
    }
    // Also index path components for Step 3e namespace/module-dir match.
    // Key = directory component name (e.g. "X" from `src/X/Alpha.cs`).
    // Value = Vec<(full_path, idx)>. Same O(1) lookup benefit.
    let mut dir_component_idx: FxHashMap<&str, Vec<(&str, u32)>> = FxHashMap::default();
    for (path, &idx) in file_node_idx.iter() {
        for component in path.split('/') {
            if !component.is_empty() && !component.contains('.') {
                dir_component_idx
                    .entry(component)
                    .or_default()
                    .push((path.as_str(), idx));
            }
        }
    }

    for local_graph in local_graphs {
        let path_str = local_graph.file_path.to_string_lossy().replace('\\', "/");
        let Some(&source_file_idx) = file_node_idx.get(&path_str) else {
            continue;
        };
        if local_graph.imports.is_empty() {
            continue;
        }

        for import in &local_graph.imports {
            let before = emitted;
            // Step 1: named-symbol lookup.
            emitted += try_named(
                resolver,
                local_graph,
                &import.imported_name,
                source_file_idx,
                reason_named,
                &mut dedupe,
                edges_out,
            );

            // Step 2: FQN last-segment retry (Kotlin / Java / PHP qualified imports).
            if emitted == before && import.imported_name.contains('.') {
                if let Some(last) = import.imported_name.rsplit('.').next() {
                    if !last.is_empty() && last != import.imported_name {
                        emitted += try_named(
                            resolver,
                            local_graph,
                            last,
                            source_file_idx,
                            reason_named,
                            &mut dedupe,
                            edges_out,
                        );
                    }
                }
            }

            // Step 3: module-style fallback (File → File). Strip leading
            // surrounding quotes / angle brackets common in C-family
            // `#include "alpha.h"` / `#include <alpha.h>` source strings.
            if emitted == before {
                let cleaned = import
                    .source
                    .trim_matches(|c: char| c == '"' || c == '\'' || c == '<' || c == '>');

                let probe = |spec: &str, file_node_idx: &FxHashMap<String, u32>| -> Option<u32> {
                    let mut hit: Option<u32> = None;
                    resolver.enumerate_candidates(&local_graph.file_path, spec, |candidate| {
                        let normalized = candidate.replace('\\', "/");
                        if let Some(&idx) = file_node_idx.get(&normalized) {
                            hit = Some(idx);
                            return false;
                        }
                        true
                    });
                    hit
                };

                // Step 3a: probe as-is (TypeScript `./a`, Python `.foo`, etc.
                // already encode relativity in the specifier).
                let mut target_file_idx = probe(cleaned, file_node_idx);

                // Step 3b: if still miss, retry with `./` prefix so the
                // resolver's relative-resolution branch joins caller dir
                // (Ruby `require_relative 'alpha'`, Go `import "x/pkg"`,
                // C `#include "alpha.h"` all surface here — none of them
                // prepend a `./` but they're all caller-dir-relative in
                // practice when the target lives in the same indexed tree).
                if target_file_idx.is_none() && !cleaned.starts_with('.') && !cleaned.is_empty() {
                    let with_dot = format!("./{}", cleaned);
                    target_file_idx = probe(&with_dot, file_node_idx);
                }

                // Step 3c: basename + suffix match. Handles C/C++
                // `#include "alpha.hpp"` where the header sits under a
                // search-path dir. O(1) hash lookup via basename_idx,
                // then suffix-filter within the small same-basename bucket.
                if target_file_idx.is_none() && !cleaned.is_empty() {
                    target_file_idx = suffix_match_single(cleaned, &basename_idx);
                }

                // Step 3d: caller-extension + last-segment match. Handles
                // Go `import "modulePath/pkg"` where the specifier carries
                // a `go.mod` module-name prefix that isn't a filesystem
                // path; the actual file is `pkg/<anything>.go`. Falls
                // back to `<last-segment>.<caller-ext>` basename lookup.
                if target_file_idx.is_none() && !cleaned.is_empty() {
                    if let Some(last) = cleaned.rsplit('/').next() {
                        if let Some(caller_ext) = local_graph.file_path.extension() {
                            let ext = caller_ext.to_string_lossy();
                            let candidate = format!("{}.{}", last, ext);
                            target_file_idx = suffix_match_single(&candidate, &basename_idx);
                        }
                    }
                }

                // Step 3f: Rust `use` path resolution. The Rust parser
                // stamps `use crate::a::b::Foo` as `source = "crate::a::b"`
                // (parent module path) + `imported_name = "Foo"` (already
                // split off). The resolver doesn't grok `::` as a path
                // separator nor `crate::` as a crate-root anchor, so this
                // step resolves manually.
                //
                // Cases handled:
                //   `use crate::a;`         → source="crate", name="a"      → suffix `a.rs`
                //   `use crate::a::Foo;`    → source="crate::a", name="Foo" → suffix `a.rs`
                //   `use crate::a::b::Foo;` → source="crate::a::b", name="Foo" → suffix `b.rs`
                //   `use std::io;`          → source="std", name="io"       → external, no match
                //   `use super::Foo;`       → source="super", name="Foo"    → skipped (needs caller dir walk)
                if target_file_idx.is_none() {
                    let raw = import.source.as_str();
                    let module_last: Option<String> = if raw == "crate" || raw == "self" {
                        // `use crate::Foo` / `use self::Foo` — imported_name IS
                        // the module-file basename.
                        Some(import.imported_name.clone())
                    } else if let Some(rest) = raw
                        .strip_prefix("crate::")
                        .or_else(|| raw.strip_prefix("self::"))
                    {
                        // Strip leading qualifier, take last `::` segment.
                        rest.rsplit("::").next().map(str::to_string)
                    } else if raw.starts_with("super") {
                        // super:: requires caller_dir walk; defer.
                        None
                    } else {
                        // Don't probe generic `a::b::c` forms — these are
                        // external crate imports (`use std::io::Read`,
                        // `use tokio::io::Interest`) whose last segment is
                        // a module name that coincidentally matches an
                        // unrelated internal file. Probing them caused a
                        // 15× over-extraction in the Rust corner of
                        // .sample_repo (2092 emit vs 137 valid).
                        None
                    };
                    if let Some(last) = module_last {
                        if let Some(caller_ext) = local_graph.file_path.extension() {
                            let candidate = format!("{}.{}", last, caller_ext.to_string_lossy());
                            target_file_idx = suffix_match_single(&candidate, &basename_idx);
                        }
                    }
                }

                // Step 3e: namespace/module-dir match. C# `using NS;`
                // names a namespace whose source lives under a `/NS/`
                // directory; Swift `import Module` similarly names a
                // module-dir. O(1) lookup via dir_component_idx, then
                // filter same-extension single-hit within the bucket.
                if target_file_idx.is_none() && !cleaned.is_empty() && !cleaned.contains('/') {
                    if let Some(caller_ext) = local_graph.file_path.extension() {
                        let ext_dot = format!(".{}", caller_ext.to_string_lossy());
                        if let Some(bucket) = dir_component_idx.get(cleaned) {
                            let mut hit: Option<u32> = None;
                            let mut multi = false;
                            for &(path, idx) in bucket {
                                if path.ends_with(&ext_dot) {
                                    if hit.is_some() {
                                        multi = true;
                                        break;
                                    }
                                    hit = Some(idx);
                                }
                            }
                            if !multi {
                                target_file_idx = hit;
                            }
                        }
                    }
                }

                if let Some(target) = target_file_idx {
                    if dedupe.insert((source_file_idx, target)) {
                        edges_out.push(Edge {
                            source: source_file_idx,
                            target,
                            rel_type: RelType::Imports,
                            confidence: 0.9,
                            reason: reason_module,
                        });
                        emitted += 1;
                    }
                }
            }
        }
    }

    emitted
}

/// Suffix-match `candidate` against the basename index. Lookup is O(1)
/// on the bucket (typically < 10 entries per basename); only paths that
/// share the candidate's basename get a full-suffix `ends_with` check.
/// Returns `Some(idx)` iff exactly one path equals `candidate` or ends
/// with `"/<candidate>"`; multi-hit returns `None` (single-hit constraint
/// keeps cross-language ambiguity defused — same rule as resolver Tier-3).
fn suffix_match_single(
    candidate: &str,
    basename_idx: &FxHashMap<&str, Vec<(&str, u32)>>,
) -> Option<u32> {
    let cand_basename = std::path::Path::new(candidate)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(candidate);
    let bucket = basename_idx.get(cand_basename)?;
    let needle = format!("/{}", candidate);
    let mut hit: Option<u32> = None;
    for &(path, idx) in bucket {
        if path == candidate || path.ends_with(&needle) {
            if hit.is_some() {
                return None;
            }
            hit = Some(idx);
        }
    }
    hit
}

fn try_named(
    resolver: &Resolver<'_>,
    local_graph: &LocalGraph,
    name: &str,
    source_file_idx: u32,
    reason: graph_nexus_core::pool::StrRef,
    dedupe: &mut FxHashSet<(u32, u32)>,
    edges_out: &mut Vec<Edge>,
) -> usize {
    let mut emitted = 0usize;
    for target_kind in [ResolveTarget::Callable, ResolveTarget::Type] {
        let targets = resolver.resolve_symbol(
            &local_graph.file_path,
            name,
            &local_graph.imports,
            target_kind,
        );
        for (target_id, confidence) in targets {
            if !dedupe.insert((source_file_idx, target_id)) {
                continue;
            }
            edges_out.push(Edge {
                source: source_file_idx,
                target: target_id,
                rel_type: RelType::Imports,
                confidence,
                reason,
            });
            emitted += 1;
        }
    }
    emitted
}
