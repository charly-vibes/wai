// Library interface for the `wai pipeline spawn` wrapper (ticket wai-tdnv,
// openspec change add-orchestrator-spawn-wrapper).
//! Purpose: pure spawn-checkpoint logic for the canonical subagent spawn
//! wrapper — checkpoint records, liveness classification, and one-line status
//! formatting. Responsibilities: serialize/deserialize the checkpoint file
//! that makes every spawn resumable and discoverable (canon Invariant 2),
//! classify a spawn as running/exited/budget-exhausted from pid liveness plus
//! a log-tail probe (canon Invariant 7: progress must be pokeable), and format
//! exactly one status line per spawn. Rationale: October 2026 measured 178
//! blocking `pi -p` orchestration calls (~48h) with several wedged until
//! 3600–7200s bash timeouts because spawn safety was prose, not tooling; the
//! wrapper encodes canon Invariants 2/6/7/8 as a capability. Declared in both
//! the lib and the bin (pattern: `pipeline_wait`, `matrix`, `plugin`) so
//! integration tests reach it via `wai::pipeline_spawn` while the process
//! supervision (`cmd_spawn`, `cmd_spawn_status`, `cmd_spawn_resume`) stays
//! bin-side in `src/commands/pipeline/orchestration.rs`.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// The checkpoint record written at spawn time. This file IS the
/// resume/discoverability record: session name, supervisor pid, log path,
/// prompt, wall-clock budget, and start timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpawnCheckpoint {
    /// pi session name (the `-n` value; the spawn name verbatim).
    pub session: String,
    /// pid of the supervising `timeout` process that wraps the spawn.
    pub pid: u32,
    /// Absolute path of the log file the spawn streams into.
    pub log_path: String,
    /// The prompt/brief text handed to the subagent (verbatim — resume must
    /// be able to prove the brief was NOT re-prompted).
    pub prompt: String,
    /// Wall-clock budget in minutes enforced by the `timeout` wrapper.
    pub budget_minutes: u64,
    /// Spawn start time, RFC3339 (Utc).
    pub started_at: String,
}

impl SpawnCheckpoint {
    /// Write the checkpoint as pretty JSON. The parent creates the directory.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Load a checkpoint written by `save`.
    pub fn load(path: &Path) -> std::io::Result<SpawnCheckpoint> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// Classified state of a checkpointed spawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnStatus {
    /// Supervisor pid alive and within the wall-clock budget.
    Running,
    /// Supervisor pid is gone — the spawn terminated (normally, errored, or
    /// was budget-killed). The log tail's last non-empty line is the probe.
    Exited { last_line: String },
    /// Pid alive but the wall-clock budget has elapsed — the `timeout`
    /// wrapper should kill it imminently; report the overrun so the
    /// orchestrator knows the spawn is wedged, not healthy.
    BudgetExhausted { budget_minutes: u64 },
}

/// Classify a spawn from its inputs. Precedence: a dead pid ALWAYS means
/// Exited (terminal beats a stale deadline — the budget kill also ends the
/// pid); an alive pid past its deadline is BudgetExhausted; anything else is
/// Running. An unparseable `started_at` degrades to Running (defensive: a
/// corrupt timestamp must never misreport a live spawn as wedged).
pub fn classify(
    pid_alive: bool,
    budget_minutes: u64,
    started_at_rfc3339: &str,
    now: chrono::DateTime<chrono::Utc>,
    log_tail: &str,
) -> SpawnStatus {
    if !pid_alive {
        return SpawnStatus::Exited {
            last_line: last_nonempty_line(log_tail).to_string(),
        };
    }
    if let Some(started) = chrono::DateTime::parse_from_rfc3339(started_at_rfc3339)
        .ok()
        .map(|d| d.with_timezone(&chrono::Utc))
        && now - started >= chrono::Duration::minutes(budget_minutes.max(1) as i64)
    {
        return SpawnStatus::BudgetExhausted { budget_minutes };
    }
    SpawnStatus::Running
}

/// The last non-empty line of the log tail (the one-line probe evidence for
/// an exited spawn). Empty output degrades to a marker, never an empty string.
pub fn last_nonempty_line(tail: &str) -> &str {
    tail.lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("no output yet")
}

/// Format the ONE status line for a spawn. Every branch names the spawn and
/// the log path (canon Invariant 7: one command answers what is running,
/// done or stuck, where is the log). Never multi-line.
pub fn status_line(checkpoint: &SpawnCheckpoint, status: &SpawnStatus) -> String {
    let name = &checkpoint.session;
    let log = &checkpoint.log_path;
    match status {
        SpawnStatus::Running => {
            format!(
                "{name}: running — log {log} (budget {}m)",
                checkpoint.budget_minutes
            )
        }
        SpawnStatus::Exited { last_line } => {
            format!("{name}: exited — log {log} — last output: {last_line}")
        }
        SpawnStatus::BudgetExhausted { budget_minutes } => {
            format!("{name}: BUDGET EXHAUSTED ({budget_minutes}m) — log {log}")
        }
    }
}
