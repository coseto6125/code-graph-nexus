//! Manually-constructed MCP tools for `gnx peers` sub-subcommands.
//!
//! The general `enumerate_tools` path walks one level of clap subcommands,
//! so `gnx peers` appears as a single opaque tool. These three tools expand
//! the `peers` namespace into distinct MCP-callable entries that map to
//! `gnx peers status`, `gnx peers log`, and `gnx peers say` respectively.
//! Dispatch uses `DerivedTool::prefix_args` to insert the sub-subcommand
//! name between the top-level subcommand and the JSON-derived argv.

use crate::schema::DerivedTool;
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;

/// Return the three peer MCP tools: status, log, say.
pub fn peer_tools() -> Vec<DerivedTool> {
    vec![tool_status(), tool_log(), tool_say()]
}

fn tool_status() -> DerivedTool {
    DerivedTool {
        name: "gnx_peers_status".into(),
        subcommand: "peers".into(),
        description: "List alive peer sessions for the current repo.".into(),
        schema: Arc::new(json!({
            "type": "object",
            "properties": {
                "repo": {
                    "type": "string",
                    "description": "Path to the repo root (optional; defaults to cwd)"
                }
            },
            "required": [],
            "additionalProperties": false
        })),
        flag_args: HashSet::new(),
        positional_args: Vec::new(),
        prefix_args: vec!["status".into()],
    }
}

fn tool_log() -> DerivedTool {
    DerivedTool {
        name: "gnx_peers_log".into(),
        subcommand: "peers".into(),
        description: "Tail this session's Ƀ message log (optionally filtered by peer / direction)."
            .into(),
        schema: Arc::new(json!({
            "type": "object",
            "properties": {
                "peer": {
                    "type": "string",
                    "description": "Show only messages to/from this peer session ID"
                },
                "direction": {
                    "type": "string",
                    "description": "Filter by direction: 'in' or 'out'"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of messages to return (default 50)"
                },
                "repo": {
                    "type": "string",
                    "description": "Path to the repo root (optional; defaults to cwd)"
                }
            },
            "required": [],
            "additionalProperties": false
        })),
        flag_args: HashSet::new(),
        positional_args: Vec::new(),
        prefix_args: vec!["log".into()],
    }
}

fn tool_say() -> DerivedTool {
    DerivedTool {
        name: "gnx_peers_say".into(),
        subcommand: "peers".into(),
        description: "Ƀ Send a message to all peers or a specific peer (fire-and-forget).".into(),
        schema: Arc::new(json!({
            "type": "object",
            "properties": {
                "body": {
                    "type": "string",
                    "description": "Message body to send"
                },
                "to": {
                    "type": "string",
                    "description": "Target peer session ID (omit to broadcast)"
                },
                "reply": {
                    "type": "string",
                    "description": "msg_id this message is replying to"
                },
                "repo": {
                    "type": "string",
                    "description": "Path to the repo root (optional; defaults to cwd)"
                }
            },
            "required": ["body"],
            "additionalProperties": false
        })),
        flag_args: HashSet::new(),
        positional_args: vec!["body".into()],
        prefix_args: vec!["say".into()],
    }
}
