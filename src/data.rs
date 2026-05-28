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
    ensure_data_dir()?;
    LockGuard::acquire(&data_dir())
}

pub fn lock_file_path() -> PathBuf {
    data_dir().join("LOCK")
}

/// Remove LOCK file if present. Missing file is not treated as an error.
pub fn remove_lock_file_if_exists() -> std::io::Result<()> {
    match fs::remove_file(lock_file_path()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// Return the path of the most recent day file that is NOT today.
/// Used to seed the project list when no projectlist file exists or
/// when projectlist contains stale/test entries.
pub fn find_most_recent_dayfile() -> Option<PathBuf> {
    let dir = data_dir();
    let today = today_string();
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name != &today && is_dayfile_name(name))
        .collect();
    names.sort();
    names.pop().map(|name| dir.join(name))
}

/// Returns true if the name looks like a YYYY-MM-DD day file.
fn is_dayfile_name(name: &str) -> bool {
    name.len() == 10
        && name.as_bytes()[4] == b'-'
        && name.as_bytes()[7] == b'-'
        && name[..4].chars().all(|c| c.is_ascii_digit())
        && name[5..7].chars().all(|c| c.is_ascii_digit())
        && name[8..].chars().all(|c| c.is_ascii_digit())
}

pub fn data_dir() -> PathBuf {
    // Respect TIMETRACKDIR and TIMEXDIR env vars per BLUEPRINT spec.
    // Also accept TITRAXDIR for compatibility with older documentation.
    if let Some(path) = env_dir("TIMETRACKDIR") {
        return path;
    }
    if let Some(path) = env_dir("TIMEXDIR") {
        return path;
    }
    if let Some(path) = env_dir("TITRAXDIR") {
        return path;
    }
    let mut home = home_dir();
    home.push(".TimeTracker");
    home
}

fn env_dir(var: &str) -> Option<PathBuf> {
    let raw = std::env::var(var).ok()?;
    let val = raw.trim();
    if val.is_empty() {
        return None;
    }
    Some(expand_tilde(val))
}

fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        return home_dir();
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return home_dir().join(rest);
    }
    PathBuf::from(path)
}

pub fn ensure_data_dir() -> std::io::Result<()> {
    fs::create_dir_all(data_dir())
}

pub fn today_file() -> PathBuf {
    data_dir().join(today_string())
}

pub fn day_file_for(date: &str) -> PathBuf {
    data_dir().join(date)
}

pub fn projectlist_file() -> PathBuf {
    data_dir().join("projectlist")
}

/// Read a file as a String, tolerating non-UTF-8 content.
/// Tries UTF-8 first; falls back to ISO-8859-1 (Latin-1).
/// Latin-1 byte values 0x00–0xFF map 1:1 to Unicode U+0000–U+00FF,
/// so the conversion always succeeds and roundtrips correctly for
/// day files written by the original C titrax on Norwegian systems.
fn read_file_lenient(path: &Path) -> Option<String> {
    if let Ok(s) = fs::read_to_string(path) {
        return Some(s);
    }
    let bytes = fs::read(path).ok()?;
    Some(bytes.iter().map(|&b| b as char).collect())
}

/// Read the master project list — one project name per line, preserving order.
/// Lines starting with `#` and blank lines are ignored.
pub fn read_projectlist(path: &Path) -> Vec<String> {
    let text = match read_file_lenient(path) {
        Some(t) => t,
        None => return Vec::new(),
    };
    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect()
}

/// Write the master project list — one project name per line.
pub fn write_projectlist(path: &Path, projects: &[Project]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = projects
        .iter()
        .map(|p| format!("{}\n", p.name))
        .collect::<String>();
    fs::write(path, content)
}

pub fn today_string() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Read a day-file. Lines starting with `#` are comments and are skipped.
/// Handles both old titrax format (` 2:00 Name`) and new format (`02:00 Name`).
/// Uses lenient reading to tolerate ISO-8859-1 files from original titrax.
pub fn read_dayfile(path: &Path) -> Vec<Project> {
    let text = match read_file_lenient(path) {
        Some(t) => t,
        None => return Vec::new(),
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
        projects.push(Project { name, minutes });
    }
    projects
}

/// Write a day-file. Only projects with time > 0 are written,
/// matching the original titrax behaviour and keeping day files clean.
pub fn write_dayfile(path: &Path, projects: &[Project]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut content = format!("# TimeTracker log saved at {}\n", timestamp);
    for p in projects.iter().filter(|p| p.minutes > 0) {
        content.push_str(&format!("{} {}\n", format_hhmm(p.minutes), p.name));
    }
    fs::write(path, content)
}

pub fn parse_hhmm(s: &str) -> u32 {
    let mut parts = s.splitn(2, ':');
    let h: u32 = parts.next().unwrap_or("0").trim().parse().unwrap_or(0);
    let m: u32 = parts.next().unwrap_or("0").trim().parse().unwrap_or(0);
    h * 60 + m
}

pub fn format_hhmm(minutes: u32) -> String {
    format!("{:2}:{:02}", minutes / 60, minutes % 60)
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
            assert!(
                lock_path.exists(),
                "LOCK file must exist while guard is held"
            );
        }
        assert!(
            !lock_path.exists(),
            "LOCK file must be removed after guard drops"
        );
    }

    #[test]
    fn test_lock_guard_exclusive() {
        let dir = std::env::temp_dir().join("titrax-test-lock-excl");
        fs::create_dir_all(&dir).unwrap();

        let _guard = LockGuard::acquire(&dir).expect("first acquire must succeed");
        let second = LockGuard::acquire(&dir);
        assert!(
            second.is_err(),
            "second acquire must fail while lock is held"
        );
    }

    #[test]
    fn test_parse_hhmm_roundtrip() {
        assert_eq!(parse_hhmm("01:30"), 90);
        assert_eq!(parse_hhmm(" 1:30"), 90);
        assert_eq!(format_hhmm(90), " 1:30");
        assert_eq!(parse_hhmm("00:00"), 0);
        assert_eq!(parse_hhmm(" 0:00"), 0);
        assert_eq!(format_hhmm(0), " 0:00");
    }

    #[test]
    fn test_read_dayfile_old_titrax_format() {
        // Old titrax writes lines with a leading space and single-digit hours,
        // e.g. " 2:00 Authentisering" and " 0:53 IT IS datasenter".
        // Verify that read_dayfile handles this correctly.
        let dir = std::env::temp_dir().join("titrax-test-dayfile");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("2026-05-21");
        fs::write(
            &path,
            "# TIMETRACKER log saved at Fri May 22 00:00:42 2026\n\
             # End of day\n\
              2:00 Authentisering\n\
              3:02 Basis team\n\
              2:02 IT IS Annet\n\
              0:53 IT IS datasenter\n",
        )
        .unwrap();
        let projects = read_dayfile(&path);
        assert_eq!(projects.len(), 4);
        assert_eq!(projects[0].name, "Authentisering");
        assert_eq!(projects[0].minutes, 120);
        assert_eq!(projects[1].name, "Basis team");
        assert_eq!(projects[1].minutes, 182);
        assert_eq!(projects[2].name, "IT IS Annet");
        assert_eq!(projects[2].minutes, 122);
        assert_eq!(projects[3].name, "IT IS datasenter");
        assert_eq!(projects[3].minutes, 53);
    }

    #[test]
    fn test_force_flag_removes_stale_lock() {
        let dir = std::env::temp_dir().join("titrax-test-force-flag");
        fs::create_dir_all(&dir).unwrap();
        let lock_path = dir.join("LOCK");

        // Simulate a stale lock file left by a crashed previous instance.
        fs::write(&lock_path, "stale").unwrap();
        assert!(
            lock_path.exists(),
            "stale LOCK file must exist before force-remove"
        );

        // Simulate what --force does: remove the stale lock file.
        fs::remove_file(&lock_path).expect("force-remove of stale lock must succeed");

        // After removal, acquiring the lock must succeed.
        let guard =
            LockGuard::acquire(&dir).expect("acquire must succeed after stale lock removed");
        assert!(
            lock_path.exists(),
            "LOCK file must exist while guard is held"
        );
        drop(guard);
        assert!(
            !lock_path.exists(),
            "LOCK file must be removed after guard drops"
        );
    }

    #[test]
    fn test_titraxdir_env_is_supported() {
        let tmp = std::env::temp_dir().join("titrax-test-titraxdir-env");
        std::env::set_var("TITRAXDIR", tmp.to_string_lossy().as_ref());
        std::env::remove_var("TIMETRACKDIR");
        std::env::remove_var("TIMEXDIR");

        assert_eq!(data_dir(), tmp);

        std::env::remove_var("TITRAXDIR");
    }

    #[test]
    fn test_tilde_in_env_expands_to_home() {
        std::env::set_var("TIMETRACKDIR", "~/.TimeTracker-test-tilde");
        std::env::remove_var("TIMEXDIR");
        std::env::remove_var("TITRAXDIR");

        let expected = home_dir().join(".TimeTracker-test-tilde");
        assert_eq!(data_dir(), expected);

        std::env::remove_var("TIMETRACKDIR");
    }
}
