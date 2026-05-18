//! Manually-constructed MCP tool for `gnx group` sub-subcommands.
//!
//! The root `gnx group` subcommand is `#[command(hide = true)]` in main.rs,
//! so `enumerate_tools` skips it. Without this hand-rolled tool, LLM clients
//! would have no path to invoke `gnx group <verb>` via MCP — and the
//! GroupAtTopLevel migration hint emitted by `--repo @<group>` rejection
//! would point at a verb that's MCP-unreachable.
//!
//! Mirrors the `peers.rs` pattern: one tool fronts every sub-subcommand via
//! a `subcmd` discriminator (`sync` / `status` / `contracts` / `impact` /
//! `search` / `find` / `coverage`). `spawn::peel_subcmd` lifts `subcmd` off
//! the JSON object and prepends it as the first argv token, yielding
//! `gnx group <subcmd> <name> [<query>|<pattern>] [--flags...]`.

use crate::schema::DerivedTool;
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;

/// Return the single `gnx_group` MCP tool fronting all group sub-subcommands.
pub fn group_tools() -> Vec<DerivedTool> {
    vec![tool_group()]
}

fn tool_group() -> DerivedTool {
    DerivedTool {
        name: "gnx_group".into(),
        subcommand: "group".into(),
        description: "Multi-repo group operations: extract contracts, query \
            cross-repo impact, search across all members. Pick `subcmd`; see \
            each arg's [tag] for which subcmd uses it. Groups are managed \
            via `gnx admin group add/remove`."
            .into(),
        schema: Arc::new(json!({
            "type": "object",
            "properties": {
                "subcmd": {
                    "type": "string",
                    "enum": ["sync", "status", "contracts", "impact", "search", "find", "coverage"],
                    "description": "Which group operation to run. Each subcmd uses a disjoint subset of the args below."
                },
                "name": {
                    "type": "string",
                    "description": "[all] Group name (must exist in registry; add members via `gnx admin group add <repo> <group>`)."
                },
                "query": {
                    "type": "string",
                    "description": "[search] BM25 query string."
                },
                "pattern": {
                    "type": "string",
                    "description": "[find] BM25 symbol pattern (name or fragment)."
                },
                "target": {
                    "type": "string",
                    "description": "[impact] Symbol name (function / method / file) to analyse."
                },
                "repo": {
                    "type": "string",
                    "description": "[impact] Member name within the group (dir_name or alias). [contracts] Filter by repo name."
                },
                "type": {
                    "type": "string",
                    "description": "[contracts] Filter by contract type: http|grpc|thrift|topic|lib|include|custom."
                },
                "unmatched": {
                    "type": "boolean",
                    "description": "[contracts] Show only unmatched contracts."
                },
                "direction": {
                    "type": "string",
                    "enum": ["upstream", "downstream"],
                    "description": "[impact] Traversal direction (callers vs callees). Default upstream."
                },
                "max_depth": {
                    "type": "integer",
                    "description": "[impact] Local-impact max graph traversal depth."
                },
                "cross_depth": {
                    "type": "integer",
                    "description": "[impact] Cross-repo hop depth (clamped to 1 in first wave)."
                },
                "min_confidence": {
                    "type": "number",
                    "description": "[impact] Minimum cross-link confidence to surface."
                },
                "timeout_ms": {
                    "type": "integer",
                    "description": "[impact] Local-impact wall-clock budget in ms."
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "[impact] Include test files in local traversal."
                },
                "limit": {
                    "type": "integer",
                    "description": "[search] Maximum results to return (merged mode only). Default 5."
                },
                "no_merge": {
                    "type": "boolean",
                    "description": "[search] Emit per-repo streams instead of merging (disables RRF)."
                },
                "exact_only": {
                    "type": "boolean",
                    "description": "[sync] Skip BM25 stage; exact match only."
                },
                "allow_stale": {
                    "type": "boolean",
                    "description": "[sync] Don't bail when per-repo index is stale."
                },
                "json": {
                    "type": "boolean",
                    "description": "[all] Emit JSON instead of text/TOON."
                },
                "verbose": {
                    "type": "boolean",
                    "description": "[sync] Show per-cross-link detail."
                }
            },
            "required": ["subcmd", "name"],
            "additionalProperties": false
        })),
        // Boolean bare flags (no following value).
        flag_args: HashSet::from_iter(
            [
                "unmatched",
                "include_tests",
                "no_merge",
                "exact_only",
                "allow_stale",
                "json",
                "verbose",
            ]
            .into_iter()
            .map(String::from),
        ),
        // Positional order: `name` (all subcmds), then `query` (search) OR
        // `pattern` (find). Listing both is safe — only one fires per call
        // since search/find are disjoint subcmds.
        positional_args: vec!["name".into(), "query".into(), "pattern".into()],
        prefix_args: Vec::new(),
        subcmd_arg: Some("subcmd".into()),
    }
}
