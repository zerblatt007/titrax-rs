use std::fs;
use std::path::{Path, PathBuf};

use crate::app::Project;

/// RAII lock guard — removes the lock file on drop.
/// Satisfies PROJECT_RULES.md rule 4: lock file must never be left behind.
pub struct LockGuard {
    path: PathBuf,
}

impl LockGuard {
    /// Attempt to acquire the lock file exclusively.
    /// Returns `Err` if the lock file already exists (another instance running).
    pub fn acquire(data_dir: &Path) -> std::io::Result<Self> {
        let path = data_dir.join("LOCK");
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        Ok(Self { path })
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Create the data directory and acquire the lock file.
/// Returns `Err` if the lock is already held by another process.
pub fn acquire_lock() -> std::io::Result<LockGuard> {
    let dir = data_dir();
    ensure_data_dir()?;
    LockGuard::acquire(&dir)
}

pub fn data_dir() -> PathBuf {
    // Respect TIMETRACKDIR and TIMEXDIR env vars per BLUEPRINT spec
    if let Ok(val) = std::env::var("TIMETRACKDIR") {
        return PathBuf::from(val);
    }
    if let Ok(val) = std::env::var("TIMEXDIR") {
        return PathBuf::from(val);
    }
    let mut home = home_dir();
    home.push(".TimeTracker");
    home
}

pub fn ensure_data_dir() -> std::io::Result<()> {
    fs::create_dir_all(data_dir())
}

pub fn today_file() -> PathBuf {
    data_dir().join(today_string())
}

pub fn today_string() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Read a day-file. Lines starting with `#` are comments and are skipped.
/// Format: `HH:MM ProjectName`
pub fn read_dayfile(path: &Path) -> Vec<Project> {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut projects = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, ' ');
        let time_part = parts.next().unwrap_or("00:00");
        let name = parts.next().unwrap_or("").trim().to_string();
        if name.is_empty() {
            continue;
        }
        let minutes = parse_hhmm(time_part);
        projects.push(Project {
            name,
            minutes,
            marked: false,
        });
    }
    projects
}

/// Write a day-file with a header comment for backward compatibility.
pub fn write_dayfile(path: &Path, projects: &[Project]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut content = format!("# TIMETRACKER log saved at {}\n", timestamp);
    content.push_str("# Rust/GTK4 rewrite\n");
    for p in projects {
        content.push_str(&format!("{} {}\n", format_hhmm(p.minutes), p.name));
    }
    fs::write(path, content)
}

pub fn parse_hhmm(s: &str) -> u32 {
    let mut parts = s.splitn(2, ':');
    let h: u32 = parts.next().unwrap_or("0").parse().unwrap_or(0);
    let m: u32 = parts.next().unwrap_or("0").parse().unwrap_or(0);
    h * 60 + m
}

pub fn format_hhmm(minutes: u32) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_lock_guard_creates_and_removes_file() {
        let dir = std::env::temp_dir().join("titrax-test-lock");
        fs::create_dir_all(&dir).unwrap();
        let lock_path = dir.join("LOCK");

        {
            let _guard = LockGuard::acquire(&dir).expect("should acquire lock");
            assert!(lock_path.exists(), "LOCK file must exist while guard is held");
        }
        assert!(!lock_path.exists(), "LOCK file must be removed after guard drops");
    }

    #[test]
    fn test_lock_guard_exclusive() {
        let dir = std::env::temp_dir().join("titrax-test-lock-excl");
        fs::create_dir_all(&dir).unwrap();

        let _guard = LockGuard::acquire(&dir).expect("first acquire must succeed");
        let second = LockGuard::acquire(&dir);
        assert!(second.is_err(), "second acquire must fail while lock is held");
    }

    #[test]
    fn test_parse_hhmm_roundtrip() {
        assert_eq!(parse_hhmm("01:30"), 90);
        assert_eq!(format_hhmm(90), "01:30");
        assert_eq!(parse_hhmm("00:00"), 0);
        assert_eq!(format_hhmm(0), "00:00");
    }
}
