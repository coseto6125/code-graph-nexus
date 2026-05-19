//! Shared fixtures for MCP integration tests.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

fn stub_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub fn stub_guard() -> MutexGuard<'static, ()> {
    stub_test_lock().lock().unwrap()
}

/// Write an executable shell script (stub `cgn`) into `dir` and return its path.
pub fn write_stub(dir: &Path, script: &str) -> PathBuf {
    let stub = dir.join("cgn");
    std::fs::write(&stub, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = std::fs::metadata(&stub).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&stub, perms).unwrap();
    }
    stub
}
