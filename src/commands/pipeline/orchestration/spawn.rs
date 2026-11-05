
use miette::Result;



// ─── spawn wrapper (ticket wai-tdnv) ─────────────────────────────────────────

/// Canonical directory for spawn checkpoints and logs, relative to the
/// workspace root. Created on demand by every spawn verb.
fn spawns_dir() -> Result<std::path::PathBuf> {
    let project_root = crate::commands::require_project()?;
    let dir = crate::config::wai_dir(&project_root)
        .join("pipeline-runs")
        .join("spawns");
    std::fs::create_dir_all(&dir)
        .map_err(|e| miette::miette!("failed to create spawns dir {}: {e}", dir.display()))?;
    Ok(dir)
}

/// Refuse a spawn name that could escape the checkpoint/log paths (the name
/// IS the basename of both files). Canon: retries are NEW named sessions with
/// a `-retry` suffix, never the same name.
fn validate_spawn_name(name: &str) -> Result<()> {
    if name.trim().is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name == "."
        || name == ".."
    {
        return Err(miette::miette!(
            "invalid spawn name {name:?} — must be a non-empty filename-safe \
             session name (retries are new names with a -retry suffix)"
        ));
    }
    Ok(())
}

/// Launch `timeout <budget>s pi <args...>` DETACHED: stdout/stderr stream to
/// the log file, stdin nulled, and the child is NOT waited on — wai returns
/// immediately and the supervisor is reparented when wai exits. Returns the
/// supervisor pid for the checkpoint.
fn launch_detached(
    args: &[String],
    budget_minutes: u64,
    log_path: &std::path::Path,
) -> Result<u32> {
    // Both stdout AND stderr stream to the log file: a piped-but-never-drained
    // stderr fills the OS pipe buffer at ~64KB and blocks the spawn forever
    // while status still reports running — the capture-then-drain-after-wait
    // pipe-deadlock class caught by ro5u on the wait primitive (wai-63h9).
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|e| miette::miette!("failed to open log {}: {e}", log_path.display()))?;
    let stderr_file = log_file
        .try_clone()
        .map_err(|e| miette::miette!("failed to dup log handle: {e}"))?;
    let child = std::process::Command::new("timeout")
        .arg(format!("{}s", budget_minutes.saturating_mul(60)))
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(log_file))
        .stderr(std::process::Stdio::from(stderr_file))
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                miette::miette!(
                    "`timeout` binary not found on PATH — the wall-clock budget \
                     (canon invariant 8: never blind-block) cannot be enforced; \
                     install coreutils or fix PATH"
                )
            } else {
                miette::miette!("failed to launch spawn: {e}")
            }
        })?;
    Ok(child.id())
}

/// Liveness probe for a supervisor pid. Linux (the supported platform):
/// `/proc/<pid>` existence — zero forks, zero external dependencies, precise.
/// Other platforms fall back to `kill -0` (needs procps/coreutils kill).
fn pid_alive(pid: u32) -> bool {
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new(&format!("/proc/{pid}")).exists()
    }
    #[cfg(not(target_os = "linux"))]
    {
        std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

/// Read the last chunk of a log file and return its text (the one-line probe
/// evidence source). Missing/empty logs degrade to "".
fn log_tail(log_path: &std::path::Path) -> String {
    use std::io::{Read, Seek};
    let Ok(mut f) = std::fs::File::open(log_path) else {
        return String::new();
    };
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    let start = len.saturating_sub(4096);
    if f.seek(std::io::SeekFrom::Start(start)).is_err() {
        return String::new();
    }
    // Read bytes, not a String: the seek boundary can split a multi-byte
    // UTF-8 char, and read_to_string would error (swallowed → empty probe).
    let mut bytes = Vec::new();
    let _ = f.read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).into_owned()
}

/// `wai pipeline spawn <name> <prompt> [--budget <minutes>]` — canonical
/// spawn: detached `pi -p -n <name> <prompt>` under a wall-clock budget,
/// streaming to the canonical log path, with a checkpoint written before
/// returning (the checkpoint is the resume/discoverability record).
pub fn cmd_spawn(name: &str, prompt: &str, budget_minutes: u64) -> Result<()> {
    validate_spawn_name(name)?;
    if budget_minutes == 0 {
        return Err(miette::miette!(
            "--budget must be at least 1 minute — a zero budget would kill \
             the spawn instantly"
        ));
    }
    let dir = spawns_dir()?;
    let checkpoint_path = dir.join(format!("{name}.json"));
    if checkpoint_path.exists() {
        return Err(miette::miette!(
            "spawn {name} is already checkpointed — probe it with \
             'wai pipeline spawn-status {name}' or use a new -retry name \
             (canon invariant 6: retries are new named sessions)"
        ));
    }
    let log_path = dir.join(format!("{name}.log"));
    let args = vec![
        "pi".to_string(),
        "-p".to_string(),
        "-n".to_string(),
        name.to_string(),
        prompt.to_string(),
    ];
    let pid = launch_detached(&args, budget_minutes, &log_path)?;
    let checkpoint = crate::pipeline_spawn::SpawnCheckpoint {
        session: name.to_string(),
        pid,
        log_path: log_path.display().to_string(),
        prompt: prompt.to_string(),
        budget_minutes,
        started_at: chrono::Utc::now().to_rfc3339(),
    };
    checkpoint
        .save(&checkpoint_path)
        .map_err(|e| miette::miette!("failed to write checkpoint: {e}"))?;
    println!(
        "spawn {name}: launched (pid {pid}) — log {} — budget {budget_minutes}m",
        log_path.display()
    );
    println!("→ probe: wai pipeline spawn-status {name}");
    Ok(())
}

/// `wai pipeline spawn-status [name]` — one line per checkpointed spawn:
/// running / exited / budget-exhausted + log path, classified from pid
/// liveness plus a log-tail probe. Never opens session JSONLs.
pub fn cmd_spawn_status(name: Option<&str>) -> Result<()> {
    let dir = spawns_dir()?;
    let mut paths: Vec<std::path::PathBuf> = match name {
        Some(n) => {
            validate_spawn_name(n)?;
            let p = dir.join(format!("{n}.json"));
            if !p.exists() {
                return Err(miette::miette!(
                    "no checkpoint for spawn {n} — nothing was spawned under \
                     that name in this workspace"
                ));
            }
            vec![p]
        }
        None => {
            let mut found: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
                .map_err(|e| miette::miette!("failed to read spawns dir: {e}"))?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "json"))
                .collect();
            found.sort();
            found
        }
    };
    paths.sort();
    if paths.is_empty() {
        println!("no spawns checkpointed in {}", dir.display());
        return Ok(());
    }
    let now = chrono::Utc::now();
    for path in &paths {
        let checkpoint = crate::pipeline_spawn::SpawnCheckpoint::load(path)
            .map_err(|e| miette::miette!("failed to read checkpoint {}: {e}", path.display()))?;
        let alive = pid_alive(checkpoint.pid);
        let status = crate::pipeline_spawn::classify(
            alive,
            checkpoint.budget_minutes,
            &checkpoint.started_at,
            now,
            &log_tail(std::path::Path::new(&checkpoint.log_path)),
        );
        println!(
            "{}",
            crate::pipeline_spawn::status_line(&checkpoint, &status)
        );
    }
    Ok(())
}

/// `wai pipeline spawn-resume <name> [followup]` — canon invariant 6: an
/// interrupted-but-healthy spawn resumes via `pi -c -p -n "<name>-resume"`
/// against the existing named session, detached with a fresh checkpoint.
/// Never re-prompts the full brief. A wedged spawn is a new -retry session,
/// not a resume.
pub fn cmd_spawn_resume(name: &str, followup: Option<&str>) -> Result<()> {
    validate_spawn_name(name)?;
    let dir = spawns_dir()?;
    let checkpoint_path = dir.join(format!("{name}.json"));
    if !checkpoint_path.exists() {
        return Err(miette::miette!(
            "no checkpoint for spawn {name} — resume requires the checkpoint \
             record; if this is a wedged spawn, retry as a new named session \
             with a -retry suffix instead (canon invariant 6)"
        ));
    }
    let parent = crate::pipeline_spawn::SpawnCheckpoint::load(&checkpoint_path)
        .map_err(|e| miette::miette!("failed to read checkpoint: {e}"))?;
    let resume_session = format!("{name}-resume");
    let resume_checkpoint_path = dir.join(format!("{resume_session}.json"));
    if resume_checkpoint_path.exists() {
        return Err(miette::miette!(
            "spawn {resume_session} is already checkpointed — probe it with \
             'wai pipeline spawn-status {resume_session}'; a second resume \
             after another interruption needs manual triage"
        ));
    }
    let log_path = dir.join(format!("{resume_session}.log"));
    let mut args = vec![
        "pi".to_string(),
        "-c".to_string(),
        "-p".to_string(),
        "-n".to_string(),
        resume_session.clone(),
    ];
    if let Some(followup) = followup {
        args.push(followup.to_string());
    }
    let pid = launch_detached(&args, parent.budget_minutes, &log_path)?;
    let checkpoint = crate::pipeline_spawn::SpawnCheckpoint {
        session: resume_session.clone(),
        pid,
        log_path: log_path.display().to_string(),
        // Record the PARENT's prompt: the resume did NOT re-prompt it, and
        // the checkpoint must be able to prove that.
        prompt: parent.prompt,
        budget_minutes: parent.budget_minutes,
        started_at: chrono::Utc::now().to_rfc3339(),
    };
    checkpoint
        .save(&resume_checkpoint_path)
        .map_err(|e| miette::miette!("failed to write resume checkpoint: {e}"))?;
    let resumed = checkpoint.session;
    println!(
        "spawn {resumed}: resumed (pid {pid}) — log {} — budget {}m",
        log_path.display(),
        checkpoint.budget_minutes
    );
    println!("→ probe: wai pipeline spawn-status {resumed}");
    Ok(())
}
