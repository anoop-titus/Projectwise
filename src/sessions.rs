use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveDateTime, Utc, Datelike};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use crate::registry::RegistryManager;

/// Returns the path to the Claude directory (~/.claude)
fn claude_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".claude")
}

/// Returns the path to the sessions log file (~/.claude/sessions.log)
fn sessions_log_path() -> PathBuf {
    claude_dir().join("sessions.log")
}

/// Log a session for the given folder.
/// Appends a line in the format: "2026-03-24T09:15:00Z {folder}\n"
pub fn log_session(folder: &str) -> Result<()> {
    let now = Utc::now();
    let timestamp = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let line = format!("{} {}\n", timestamp, folder);

    // Ensure the Claude directory exists
    let claude = claude_dir();
    if !claude.exists() {
        fs::create_dir_all(&claude)?;
    }

    // Append to the log file
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(sessions_log_path())?;
    file.write_all(line.as_bytes())?;
    Ok(())
}

/// Read all sessions from the log file.
/// Returns a vector of (date, folder) where date is the date part of the timestamp.
pub fn read_sessions() -> Result<Vec<(NaiveDate, String)>> {
    let path = sessions_log_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(&path)?;
    let reader = BufReader::new(file);
    let mut sessions = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() != 2 {
            continue;
        }
        let timestamp_str = parts[0];
        let folder = parts[1].trim();

        // Parse the timestamp (we only need the date part)
        let dt = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%SZ")
            .with_context(|| format!("invalid timestamp: {timestamp_str}"))?;
        let date = dt.date();
        sessions.push((date, folder.to_string()));
    }

    Ok(sessions)
}

/// Aggregate sessions by day, returning a HashMap from date to count of sessions on that day.
pub fn aggregate_by_day() -> Result<HashMap<NaiveDate, u32>> {
    let sessions = read_sessions()?;
    let mut counts = HashMap::new();
    for (date, _) in sessions {
        *counts.entry(date).or_insert(0) += 1;
    }
    Ok(counts)
}

/// Return the top `n` projects by session count.
pub fn top_projects(n: usize) -> Result<Vec<(String, u64)>> {
    let sessions = read_sessions()?;
    let mut counts: HashMap<String, u64> = HashMap::new();
    for (_, folder) in sessions {
        *counts.entry(folder).or_insert(0) += 1;
    }

    let mut vec: Vec<(String, u64)> = counts.into_iter().collect();
    vec.sort_by(|a, b| b.1.cmp(&a.1)); // descending by count
    vec.truncate(n);
    Ok(vec)
}

/// Return activity by weekday as an array of 7 u64, where index 0 is Sunday, 1 is Monday, ..., 6 is Saturday.
pub fn activity_by_weekday() -> Result<[u64; 7]> {
    let sessions = read_sessions()?;
    let mut counts = [0u64; 7];
    for (date, _) in sessions {
        let weekday = date.weekday().num_days_from_sunday(); // 0=Sunday, 6=Saturday
        counts[weekday as usize] += 1;
    }
    Ok(counts)
}

/// Return the session counts for the last `n` days, from oldest to newest.
/// For example, if n=30, returns a vector of 30 elements: [count 30 days ago, ..., count yesterday, count today].
pub fn last_n_days(n: u64) -> Result<Vec<u64>> {
    let sessions = read_sessions()?;
    let today = Utc::now().date_naive();

    // Initialize a vector of zeros for the last n days (today is the last element)
    let mut counts = vec![0u64; n as usize];

    for (date, _) in sessions {
        if date >= today - chrono::Duration::days(n as i64 - 1) && date <= today {
            let diff = today - date;
            let index = diff.num_days() as usize; // 0 = today, n-1 = n days ago
            // We want oldest first, so reverse: index_from_start = n - 1 - index
            let rev_index = (n as usize - 1) - index;
            if rev_index < counts.len() {
                counts[rev_index] += 1;
            }
        }
    }

    Ok(counts)
}

/// Backfill the sessions log with one entry per project using the project's last_accessed date.
/// Only runs if the log file is empty or does not exist.
#[allow(dead_code)]
pub fn backfill_if_empty(mgr: &RegistryManager) -> Result<()> {
    let log_path = sessions_log_path();
    let needs_backfill = !log_path.exists() || {
        let metadata = fs::metadata(&log_path)?;
        metadata.len() == 0
    };

    if !needs_backfill {
        return Ok(());
    }

    // Ensure the Claude directory exists
    let claude = claude_dir();
    if !claude.exists() {
        fs::create_dir_all(&claude)?;
    }

    let projects = mgr.list_sorted(crate::models::ListMode::Quick)?;

    let mut lines = String::new();
    for p in projects {
        let timestamp = p.last_accessed.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        lines.push_str(&format!("{} {}\n", timestamp, p.folder_name));
    }

    fs::write(log_path, lines.as_bytes())?;
    Ok(())
}