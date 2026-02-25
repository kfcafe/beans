//! File locking for concurrent agents.
//!
//! When `file_locking` is enabled in config, agents lock files they work on
//! to prevent concurrent writes. Locks are stored as JSON files in `.beans/locks/`.
//!
//! Lock lifecycle:
//! - Pre-emptive: `bn run` locks files listed in the bean's `paths` field on spawn.
//! - On-write: The pi extension locks files on first write (safety net).
//! - Release: Locks are released when the agent finishes or is killed.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Information stored in each lock file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub bean_id: String,
    pub pid: u32,
    pub file_path: String,
    pub locked_at: i64,
}

/// A lock with its file system path.
#[derive(Debug)]
pub struct ActiveLock {
    pub info: LockInfo,
    pub lock_path: PathBuf,
}

// ---------------------------------------------------------------------------
// Lock directory
// ---------------------------------------------------------------------------

/// Return the locks directory, creating it if needed.
pub fn lock_dir(beans_dir: &Path) -> Result<PathBuf> {
    let dir = beans_dir.join("locks");
    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create locks directory: {}", dir.display()))?;
    Ok(dir)
}

/// Hash a file path to a lock filename.
fn lock_filename(file_path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(file_path.as_bytes());
    let hash = hasher.finalize();
    format!("{:x}.lock", hash)
}

/// Full path to the lock file for a given file path.
fn lock_file_path(beans_dir: &Path, file_path: &str) -> Result<PathBuf> {
    let dir = lock_dir(beans_dir)?;
    Ok(dir.join(lock_filename(file_path)))
}

// ---------------------------------------------------------------------------
// Lock operations
// ---------------------------------------------------------------------------

/// Acquire a lock on a file for a bean agent.
///
/// Returns `Ok(true)` if the lock was acquired, `Ok(false)` if already locked
/// by another live process. Stale locks (dead PID) are automatically cleaned.
pub fn acquire(beans_dir: &Path, bean_id: &str, pid: u32, file_path: &str) -> Result<bool> {
    let lock_path = lock_file_path(beans_dir, file_path)?;

    // Check for existing lock
    if lock_path.exists() {
        match read_lock(&lock_path) {
            Some(existing) => {
                if is_process_alive(existing.pid) {
                    // Lock is held by a live process
                    return Ok(false);
                }
                // Stale lock — remove it
                let _ = fs::remove_file(&lock_path);
            }
            None => {
                // Corrupt lock file — remove it
                let _ = fs::remove_file(&lock_path);
            }
        }
    }

    let info = LockInfo {
        bean_id: bean_id.to_string(),
        pid,
        file_path: file_path.to_string(),
        locked_at: chrono::Utc::now().timestamp(),
    };

    let content = serde_json::to_string_pretty(&info)
        .context("Failed to serialize lock info")?;

    fs::write(&lock_path, content)
        .with_context(|| format!("Failed to write lock file: {}", lock_path.display()))?;

    Ok(true)
}

/// Release a lock on a file.
///
/// Safe to call even if the lock doesn't exist or is held by another process.
pub fn release(beans_dir: &Path, file_path: &str) -> Result<()> {
    let lock_path = lock_file_path(beans_dir, file_path)?;
    match fs::remove_file(&lock_path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("Failed to remove lock: {}", lock_path.display())),
    }
}

/// Release all locks held by a specific bean.
pub fn release_all_for_bean(beans_dir: &Path, bean_id: &str) -> Result<u32> {
    let mut released = 0;
    for lock in list_locks(beans_dir)? {
        if lock.info.bean_id == bean_id {
            let _ = fs::remove_file(&lock.lock_path);
            released += 1;
        }
    }
    Ok(released)
}

/// Release all locks held by a specific PID.
pub fn release_all_for_pid(beans_dir: &Path, pid: u32) -> Result<u32> {
    let mut released = 0;
    for lock in list_locks(beans_dir)? {
        if lock.info.pid == pid {
            let _ = fs::remove_file(&lock.lock_path);
            released += 1;
        }
    }
    Ok(released)
}

/// Force-clear all locks.
pub fn clear_all(beans_dir: &Path) -> Result<u32> {
    let mut cleared = 0;
    for lock in list_locks(beans_dir)? {
        let _ = fs::remove_file(&lock.lock_path);
        cleared += 1;
    }
    Ok(cleared)
}

/// List all active locks.
pub fn list_locks(beans_dir: &Path) -> Result<Vec<ActiveLock>> {
    let dir = lock_dir(beans_dir)?;
    let mut locks = Vec::new();

    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(locks),
        Err(e) => return Err(e).context("Failed to read locks directory"),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("lock") {
            continue;
        }

        if let Some(info) = read_lock(&path) {
            locks.push(ActiveLock {
                info,
                lock_path: path,
            });
        }
    }

    Ok(locks)
}

/// Check if a file is currently locked.
///
/// Returns the lock info if locked by a live process, None otherwise.
/// Automatically cleans stale locks.
pub fn check_lock(beans_dir: &Path, file_path: &str) -> Result<Option<LockInfo>> {
    let lock_path = lock_file_path(beans_dir, file_path)?;

    if !lock_path.exists() {
        return Ok(None);
    }

    match read_lock(&lock_path) {
        Some(info) => {
            if is_process_alive(info.pid) {
                Ok(Some(info))
            } else {
                // Stale — clean it up
                let _ = fs::remove_file(&lock_path);
                Ok(None)
            }
        }
        None => {
            // Corrupt — clean it up
            let _ = fs::remove_file(&lock_path);
            Ok(None)
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn read_lock(path: &Path) -> Option<LockInfo> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn is_process_alive(pid: u32) -> bool {
    // signal 0 checks existence without actually signaling
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_beans_dir() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let beans_dir = dir.path().join(".beans");
        fs::create_dir_all(&beans_dir).unwrap();
        (dir, beans_dir)
    }

    #[test]
    fn acquire_and_release() {
        let (_dir, beans_dir) = temp_beans_dir();
        let pid = std::process::id();

        let acquired = acquire(&beans_dir, "1.1", pid, "/tmp/test.rs").unwrap();
        assert!(acquired);

        // Second acquire by same PID succeeds (we don't block ourselves in bn)
        // but in practice the pi extension handles re-entrancy
        let info = check_lock(&beans_dir, "/tmp/test.rs").unwrap();
        assert!(info.is_some());
        assert_eq!(info.unwrap().bean_id, "1.1");

        release(&beans_dir, "/tmp/test.rs").unwrap();

        let info = check_lock(&beans_dir, "/tmp/test.rs").unwrap();
        assert!(info.is_none());
    }

    #[test]
    fn release_nonexistent_is_ok() {
        let (_dir, beans_dir) = temp_beans_dir();
        release(&beans_dir, "/tmp/nonexistent.rs").unwrap();
    }

    #[test]
    fn release_all_for_bean_works() {
        let (_dir, beans_dir) = temp_beans_dir();
        let pid = std::process::id();

        acquire(&beans_dir, "2.1", pid, "/tmp/a.rs").unwrap();
        acquire(&beans_dir, "2.1", pid, "/tmp/b.rs").unwrap();
        acquire(&beans_dir, "2.2", pid, "/tmp/c.rs").unwrap();

        let released = release_all_for_bean(&beans_dir, "2.1").unwrap();
        assert_eq!(released, 2);

        // c.rs should still be locked
        assert!(check_lock(&beans_dir, "/tmp/c.rs").unwrap().is_some());
        assert!(check_lock(&beans_dir, "/tmp/a.rs").unwrap().is_none());
    }

    #[test]
    fn list_locks_returns_all() {
        let (_dir, beans_dir) = temp_beans_dir();
        let pid = std::process::id();

        acquire(&beans_dir, "3.1", pid, "/tmp/x.rs").unwrap();
        acquire(&beans_dir, "3.2", pid, "/tmp/y.rs").unwrap();

        let locks = list_locks(&beans_dir).unwrap();
        assert_eq!(locks.len(), 2);
    }

    #[test]
    fn clear_all_removes_everything() {
        let (_dir, beans_dir) = temp_beans_dir();
        let pid = std::process::id();

        acquire(&beans_dir, "4.1", pid, "/tmp/p.rs").unwrap();
        acquire(&beans_dir, "4.2", pid, "/tmp/q.rs").unwrap();

        let cleared = clear_all(&beans_dir).unwrap();
        assert_eq!(cleared, 2);

        let locks = list_locks(&beans_dir).unwrap();
        assert!(locks.is_empty());
    }

    #[test]
    fn stale_lock_is_cleaned() {
        let (_dir, beans_dir) = temp_beans_dir();

        // Write a lock with a dead PID
        let lock_path = lock_file_path(&beans_dir, "/tmp/stale.rs").unwrap();
        let info = LockInfo {
            bean_id: "5.1".to_string(),
            pid: 999_999_999, // almost certainly dead
            file_path: "/tmp/stale.rs".to_string(),
            locked_at: 0,
        };
        fs::write(&lock_path, serde_json::to_string(&info).unwrap()).unwrap();

        // check_lock should clean it
        let result = check_lock(&beans_dir, "/tmp/stale.rs").unwrap();
        assert!(result.is_none());
        assert!(!lock_path.exists());
    }

    #[test]
    fn acquire_cleans_stale_and_succeeds() {
        let (_dir, beans_dir) = temp_beans_dir();

        // Plant a stale lock
        let lock_path = lock_file_path(&beans_dir, "/tmp/stale2.rs").unwrap();
        let info = LockInfo {
            bean_id: "6.1".to_string(),
            pid: 999_999_999,
            file_path: "/tmp/stale2.rs".to_string(),
            locked_at: 0,
        };
        fs::write(&lock_path, serde_json::to_string(&info).unwrap()).unwrap();

        // Acquire should clean the stale lock and succeed
        let acquired = acquire(&beans_dir, "6.2", std::process::id(), "/tmp/stale2.rs").unwrap();
        assert!(acquired);
    }
}
