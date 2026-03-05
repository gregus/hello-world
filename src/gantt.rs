/// Gantt chart intermediate representation and parsing from mermaid-rs-renderer's Graph.

use anyhow::{bail, Result};
use mermaid_rs_renderer::ir::{GanttStatus, GanttTask};
use mermaid_rs_renderer::Graph;
use std::collections::HashMap;

// ── Public IR types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GanttData {
    pub title: String,
    /// Tasks in the order they appear in the diagram
    pub tasks: Vec<ResolvedTask>,
    /// Earliest start day across all tasks (days since 1970-01-01)
    pub start_day: i32,
    /// Latest end day across all tasks
    pub end_day: i32,
}

#[derive(Debug, Clone)]
pub struct ResolvedTask {
    pub id: String,
    pub label: String,
    pub section: Option<String>,
    /// Absolute start, in days since 1970-01-01
    pub start_day: i32,
    /// Absolute end (exclusive), in days since 1970-01-01
    pub end_day: i32,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Default,
    Done,
    Active,
    Crit,
    Milestone,
}

// ── Construction ─────────────────────────────────────────────────────────────

impl GanttData {
    /// Build `GanttData` from a parsed mermaid `Graph`.
    pub fn from_graph(graph: &Graph) -> Result<Self> {
        let title = graph
            .gantt_title
            .clone()
            .unwrap_or_else(|| "Gantt Chart".to_string());

        let raw_tasks = &graph.gantt_tasks;
        if raw_tasks.is_empty() {
            bail!("The Gantt diagram contains no tasks");
        }

        // Resolve tasks iteratively; tasks with `after` dependencies may need
        // the referenced task to be resolved first.
        let mut resolved: Vec<Option<ResolvedTask>> = vec![None; raw_tasks.len()];
        // Map id → index for fast look-ups.
        let id_to_idx: HashMap<&str, usize> = raw_tasks
            .iter()
            .enumerate()
            .map(|(i, t)| (t.id.as_str(), i))
            .collect();

        for _pass in 0..raw_tasks.len() + 1 {
            let mut progress = false;
            for (i, task) in raw_tasks.iter().enumerate() {
                if resolved[i].is_some() {
                    continue;
                }
                if let Some(rt) = try_resolve(task, i, raw_tasks, &resolved, &id_to_idx) {
                    resolved[i] = Some(rt);
                    progress = true;
                }
            }
            if !progress {
                break;
            }
        }

        let tasks: Vec<ResolvedTask> = resolved.into_iter().flatten().collect();

        if tasks.is_empty() {
            bail!("Could not resolve any Gantt tasks (check date/duration syntax and dependencies)");
        }

        let start_day = tasks.iter().map(|t| t.start_day).min().unwrap();
        let end_day = tasks.iter().map(|t| t.end_day).max().unwrap();

        Ok(GanttData {
            title,
            tasks,
            start_day,
            end_day,
        })
    }

    /// Returns section names in first-seen order (skips tasks with no section).
    pub fn sections(&self) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        self.tasks
            .iter()
            .filter_map(|t| t.section.as_ref())
            .filter(|s| seen.insert(s.clone()))
            .cloned()
            .collect()
    }
}

// ── Resolution helpers ────────────────────────────────────────────────────────

fn try_resolve(
    task: &GanttTask,
    idx: usize,
    raw_tasks: &[GanttTask],
    resolved: &[Option<ResolvedTask>],
    id_to_idx: &HashMap<&str, usize>,
) -> Option<ResolvedTask> {
    let duration_days = task
        .duration
        .as_deref()
        .and_then(parse_duration)
        .unwrap_or(1);

    let start_day: i32 = if let Some(date_str) = &task.start {
        // Explicit start date
        parse_date(date_str)?
    } else if let Some(after_id) = &task.after {
        // Starts after another task finishes
        let dep_idx = *id_to_idx.get(after_id.trim())?;
        resolved[dep_idx].as_ref()?.end_day
    } else {
        // Implicit: follows previous task
        if idx == 0 {
            0 // No previous task; start at epoch (relative zero)
        } else {
            // Find the last resolved task before this one
            (0..idx)
                .rev()
                .find_map(|j| resolved[j].as_ref().map(|r| r.end_day))?
        }
    };

    let status = match task.status {
        Some(GanttStatus::Done) => TaskStatus::Done,
        Some(GanttStatus::Active) => TaskStatus::Active,
        Some(GanttStatus::Crit) => TaskStatus::Crit,
        Some(GanttStatus::Milestone) => TaskStatus::Milestone,
        None => TaskStatus::Default,
    };

    let end_day = match status {
        TaskStatus::Milestone => start_day + 1, // milestones are one day wide (displayed as a marker)
        _ => start_day + duration_days,
    };

    Some(ResolvedTask {
        id: task.id.clone(),
        label: task.label.clone(),
        section: task.section.clone(),
        start_day,
        end_day,
        status,
    })
}

// ── Date / duration parsing ───────────────────────────────────────────────────

/// Parse an ISO-style date ("YYYY-MM-DD", "YYYY/MM/DD", "YYYY.MM.DD") into
/// days since 1970-01-01.
pub fn parse_date(s: &str) -> Option<i32> {
    let s = s.trim();
    let sep = if s.contains('-') {
        '-'
    } else if s.contains('/') {
        '/'
    } else if s.contains('.') {
        '.'
    } else {
        return None;
    };

    let mut parts = s.splitn(3, sep);
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;

    if month < 1 || month > 12 || day < 1 || day > 31 {
        return None;
    }

    Some(days_from_civil(year, month, day))
}

/// Parse a Mermaid duration string (e.g. "5d", "2w", "1m") into days.
pub fn parse_duration(s: &str) -> Option<i32> {
    let s = s.trim();
    if s.len() < 2 {
        return None;
    }
    let (num_str, unit) = s.split_at(s.len() - 1);
    let n: i32 = num_str.trim().parse().ok()?;
    match unit.to_ascii_lowercase().as_str() {
        "d" => Some(n),
        "w" => Some(n * 7),
        "m" => Some(n * 30),
        "y" => Some(n * 365),
        "h" => Some((n / 24).max(1)),
        _ => None,
    }
}

// ── Calendar arithmetic (Zeller / civil calendar) ────────────────────────────

/// Days since 1970-01-01 (Unix epoch) for the given civil date.
pub fn days_from_civil(year: i32, month: u32, day: u32) -> i32 {
    let m = month as i32;
    let y = if m <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    // days since epoch 0000-03-01; adjust for Unix epoch (1970-01-01 = day 719468)
    era * 146097 + doe - 719468
}

/// Convert days-since-Unix-epoch back to (year, month, day).
pub fn civil_from_days(days: i32) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

/// Format a day number as "YYYY-MM-DD".
pub fn format_day(day: i32) -> String {
    let (y, m, d) = civil_from_days(day);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_dates() {
        for (y, m, d) in [(2024, 1, 1), (2000, 3, 1), (1970, 1, 1), (2025, 12, 31)] {
            let days = days_from_civil(y, m, d);
            let (ry, rm, rd) = civil_from_days(days);
            assert_eq!((y as i32, m, d), (ry, rm, rd));
        }
    }

    #[test]
    fn parse_date_formats() {
        assert_eq!(parse_date("2024-01-01"), parse_date("2024/01/01"));
        assert_eq!(parse_date("2024-01-01"), parse_date("2024.01.01"));
    }

    #[test]
    fn parse_duration_units() {
        assert_eq!(parse_duration("7d"), Some(7));
        assert_eq!(parse_duration("2w"), Some(14));
        assert_eq!(parse_duration("1m"), Some(30));
    }
}
