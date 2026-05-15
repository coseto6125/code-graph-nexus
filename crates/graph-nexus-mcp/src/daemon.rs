//! Daemon-mode dispatch. Engine mmap'd once at server startup; refreshed
//! via mtime-remap before every dispatch.
//!
//! Why mtime-remap: `gnx analyze` writes graph.bin via atomic
//! write-tmp + rename (see crates/graph-nexus-core/src/registry/io.rs:33).
//! This swaps the dentry but our existing mmap holds the unlinked old
//! inode. Without explicit re-load, daemon serves stale data forever.

use anyhow::{Context, Result};
use std::path::Path;
use std::time::SystemTime;

/// True iff the file at `path` has been replaced since `loaded_at`.
/// Returns Err if the file is missing or unreadable (caller decides
/// whether to abort or retry).
pub fn needs_remap(path: &Path, loaded_at: SystemTime) -> Result<bool> {
    let meta = std::fs::metadata(path)
        .with_context(|| format!("stat {path:?} for mtime-remap check"))?;
    let mtime = meta
        .modified()
        .with_context(|| format!("modified() for {path:?}"))?;
    Ok(mtime > loaded_at)
}
