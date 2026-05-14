//! Output emission: consolidates the toon/json branching previously
//! duplicated across every command.

use gnx_core::GnxError;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Toon,
    Json,
    Text,
}

impl OutputFormat {
    pub fn parse(s: Option<&str>) -> Self {
        match s {
            Some("json") => OutputFormat::Json,
            Some("text") => OutputFormat::Text,
            _ => OutputFormat::Toon, // default
        }
    }
}

/// Print `value` to stdout in the requested format. For `Text`, callers must
/// have already produced human-readable lines and packed them as JSON strings
/// inside `value["results"]` (or `value` itself if a string array). This keeps
/// commands free of inline branching while preserving their custom text output.
pub fn emit(value: &Value, format: OutputFormat) -> Result<(), GnxError> {
    match format {
        OutputFormat::Toon => {
            let bytes = serde_json::to_vec(value)
                .map_err(|e| GnxError::Output(format!("json serialize: {e}")))?;
            let output = _etoon::toon::encode(&bytes)
                .map_err(|e| GnxError::Output(format!("toon encode: {e}")))?;
            println!("{}", output);
        }
        OutputFormat::Json => {
            let s = serde_json::to_string(value)
                .map_err(|e| GnxError::Output(format!("json serialize: {e}")))?;
            println!("{}", s);
        }
        OutputFormat::Text => {
            // If the value has a "results" array of strings, print each line.
            // Otherwise fall back to pretty JSON.
            if let Some(results) = value.get("results").and_then(|v| v.as_array()) {
                for r in results {
                    if let Some(s) = r.as_str() {
                        println!("{}", s);
                    }
                }
            } else {
                let s = serde_json::to_string_pretty(value)
                    .map_err(|e| GnxError::Output(format!("json pretty: {e}")))?;
                println!("{}", s);
            }
        }
    }
    Ok(())
}
