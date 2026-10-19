//! Purpose: regression tests for the canonical subagent spawn wrapper
//! (ticket wai-tdnv tooling layer, openspec change
//! add-orchestrator-spawn-wrapper).
//! Responsibilities: pin the contract that (1) the epic-orchestrator template
//! requires `wai pipeline spawn` for subagent spawns and names improvised
//! manual `pi -p ... &` incantations as a violation, (2) spawning through the
//! wrapper returns immediately, launches the named-session invocation
//! (`pi -p -n <name> <prompt>`) under a wall-clock budget, streams output to a
//! known log path, and writes a checkpoint file recording session/pid/log/
//! budget, (3) status classifies spawns (running/exited/budget-exhausted) from
//! pid liveness plus a one-line log-tail probe, and (4) resume launches the
//! canon Invariant 6 continuation (`pi -c -p -n "<name>-resume"`) against the
//! existing named session with a fresh checkpoint — never re-prompting the
//! full brief. Rationale: October 2026 measured 178 blocking `pi -p` calls
//! (~48h) with several wedged until 3600–7200s bash timeouts because the
//! invariants were prose, not tooling (canon Invariant 10).

mod common;

use common::*;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use wai::pipeline_spawn;

/// Install a stub `pi` on PATH that records its argv to a file (one line per
/// invocation, space-joined) before behaving per env knobs:
///   PI_STUB_SLEEP  seconds to sleep after recording (default 30)
/// The env knobs are inherited by the wai child and passed through to the
/// spawned pi stub, so each invocation is observable without a real pi.
fn install_stub_pi(dir: &std::path::Path, args_log: &std::path::Path) -> PathBuf {
    let bin_dir = dir.join("fake-bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let script_path = bin_dir.join("pi");
    if script_path.exists() {
        // Idempotent: rewriting a script a still-running stub is executing
        // fails with ETXTBSY, and the content is invocation-independent (the
        // args log comes in via env, behavior via PI_STUB_SLEEP).
        return bin_dir;
    }
    let log = args_log.display();
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\necho 'stub pi started'\nsleep \"${{PI_STUB_SLEEP:-30}}\"\n"
    );
    fs::write(&script_path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        let _ = fs::set_permissions(&script_path, perms);
    }
    bin_dir
}

/// Spawn command builder with the stub pi prepended to PATH.
fn spawn_cmd(dir: &std::path::Path, args_log: &std::path::Path) -> assert_cmd::Command {
    let bin_dir = install_stub_pi(dir, args_log);
    let mut cmd = hermetic_wai_cmd(dir);
    let path = std::env::var("PATH").unwrap_or_default();
    cmd.env("PATH", format!("{}:{path}", bin_dir.display()));
    cmd.env("PI_STUB_ARGS_LOG", args_log);
    cmd
}

/// Wait until the predicate holds (stub argv flush is async w.r.t. the wai
/// process exiting). Polls the test-side condition, never the spawn itself.
fn wait_until<F: Fn() -> bool>(timeout: Duration, pred: F) -> bool {
    let start = Instant::now();
    loop {
        if pred() {
            return true;
        }
        if start.elapsed() > timeout {
            return false;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Read the argv log (one invocation per line, space-joined argv).
fn read_args_log(args_log: &std::path::Path) -> String {
    fs::read_to_string(args_log).unwrap_or_default()
}

/// Is the given pid alive? (kill -0 semantics via /proc on Linux.)
fn pid_alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

// ---------------------------------------------------------------------------
// 1.1 Template: spawn step requires the wrapper, names manual incantations
// as a violation.
// ---------------------------------------------------------------------------

#[test]
fn orchestrator_template_requires_spawn_wrapper() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let init = hermetic_wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("init should run");
    assert!(init.status.success(), "init epic-orchestrator failed");

    let template = tmp
        .path()
        .join(".wai/resources/pipelines/epic-orchestrator.toml");
    let content =
        fs::read_to_string(&template).expect("epic-orchestrator template should exist after init");

    let spawn_pos = content
        .find("id = \"spawn\"")
        .expect("template must have a spawn step");
    let next_step = content[spawn_pos..]
        .find("[[steps]]")
        .map(|i| spawn_pos + i)
        .unwrap_or(content.len());
    let spawn_section = &content[spawn_pos..next_step];

    assert!(
        spawn_section.contains("wai pipeline spawn"),
        "spawn step must require `wai pipeline spawn` as the spawn path; \
         spawn section:\n{spawn_section}"
    );
    assert!(
        spawn_section.contains("pi -p") && spawn_section.contains("incantation"),
        "spawn step must name improvised manual `pi -p ... &` incantations \
         as a violation; spawn section:\n{spawn_section}"
    );
}

// ---------------------------------------------------------------------------
// 1.2 Wrapper spawn: returns immediately, checkpoint written, named-session
// invocation observed, output in the log file.
// ---------------------------------------------------------------------------

#[test]
fn spawn_wrapper_writes_checkpoint() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    let args_log = tmp.path().join("pi-args.log");

    let out = spawn_cmd(tmp.path(), &args_log)
        .args([
            "pipeline",
            "spawn",
            "subagent:test:run",
            "hello world brief",
        ])
        .env("PI_STUB_SLEEP", "30")
        .output()
        .expect("spawn should run");
    assert!(
        out.status.success(),
        "spawn failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);

    // Checkpoint file exists at the canonical path with the required record.
    let checkpoint_path = tmp
        .path()
        .join(".wai/pipeline-runs/spawns/subagent:test:run.json");
    assert!(
        checkpoint_path.exists(),
        "checkpoint must exist at {checkpoint_path:?}; stdout:\n{stdout}"
    );
    let checkpoint =
        pipeline_spawn::SpawnCheckpoint::load(&checkpoint_path).expect("checkpoint must parse");
    assert_eq!(checkpoint.session, "subagent:test:run");
    assert!(checkpoint.pid > 0, "checkpoint must record the pid");
    assert_eq!(checkpoint.budget_minutes, 30, "default budget is 30m");
    assert!(
        checkpoint
            .log_path
            .ends_with("spawns/subagent:test:run.log"),
        "log path must be the canonical spawns/ path, got {}",
        checkpoint.log_path
    );

    // The stub received the named-session invocation.
    assert!(
        wait_until(Duration::from_secs(10), || {
            read_args_log(&args_log).contains("-n subagent:test:run")
        }),
        "stub pi must be invoked with -n subagent:test:run; argv log:\n{}",
        read_args_log(&args_log)
    );
    let argv = read_args_log(&args_log);
    assert!(
        argv.contains("-p") && argv.contains("hello world brief"),
        "stub pi must receive the prompt; argv log:\n{argv}"
    );

    // Output streams to the known log path.
    let log_path = tmp
        .path()
        .join(".wai/pipeline-runs/spawns/subagent:test:run.log");
    assert!(
        log_path.exists(),
        "log file must exist at the canonical path"
    );

    // The spawn returned immediately (not blocking on the 30s stub sleep):
    // the output command above already returned well under 30s — implicitly
    // proven by reaching this point without the harness timing out; also the
    // checkpoint pid should still be alive (stub sleeping).
    assert!(
        wait_until(Duration::from_secs(5), || pid_alive(checkpoint.pid)),
        "spawned process should be running (pid {})",
        checkpoint.pid
    );
}

// ---------------------------------------------------------------------------
// 1.3 Status: one line per spawn, classified from pid liveness + log probe;
// pure classification pinned for running/exited/budget-exhausted.
// ---------------------------------------------------------------------------

#[test]
fn spawn_status_classifies_liveness() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    let args_log = tmp.path().join("pi-args.log");

    // Running: stub sleeps 30s (kept alive through the running probe).
    let out = spawn_cmd(tmp.path(), &args_log)
        .args(["pipeline", "spawn", "subagent:test:probe", "probe prompt"])
        .env("PI_STUB_SLEEP", "30")
        .output()
        .expect("spawn should run");
    assert!(
        out.status.success(),
        "spawn failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let status_out = spawn_cmd(tmp.path(), &args_log)
        .args(["pipeline", "spawn-status"])
        .output()
        .expect("status should run");
    assert!(status_out.status.success(), "status failed");
    let stdout = String::from_utf8_lossy(&status_out.stdout);
    assert!(
        stdout.contains("subagent:test:probe") && stdout.to_lowercase().contains("running"),
        "status must name the spawn and its running state; got:\n{stdout}"
    );
    assert!(
        stdout.contains("subagent:test:probe.log"),
        "status must name the log path; got:\n{stdout}"
    );

    // Exited: a second spawn whose stub exits immediately, then wait for the
    // pid to die before probing.
    let out = spawn_cmd(tmp.path(), &args_log)
        .args(["pipeline", "spawn", "subagent:test:done", "done prompt"])
        .env("PI_STUB_SLEEP", "0")
        .output()
        .expect("spawn should run");
    assert!(
        out.status.success(),
        "spawn failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let done_checkpoint = pipeline_spawn::SpawnCheckpoint::load(
        &tmp.path()
            .join(".wai/pipeline-runs/spawns/subagent:test:done.json"),
    )
    .unwrap();
    assert!(
        wait_until(Duration::from_secs(10), || !pid_alive(done_checkpoint.pid)),
        "done stub should exit (pid {})",
        done_checkpoint.pid
    );

    let status_out = spawn_cmd(tmp.path(), &args_log)
        .args(["pipeline", "spawn-status", "subagent:test:done"])
        .output()
        .expect("status should run");
    let stdout = String::from_utf8_lossy(&status_out.stdout);
    assert!(
        stdout.to_lowercase().contains("exited"),
        "dead pid must classify as exited; got:\n{stdout}"
    );

    // Pure classification: budget-exhausted while pid still alive.
    let past = chrono::Utc::now() - chrono::Duration::minutes(40);
    let exhausted = pipeline_spawn::classify(
        true,
        30,
        &past.to_rfc3339(),
        chrono::Utc::now(),
        "working...",
    );
    assert!(
        matches!(
            exhausted,
            pipeline_spawn::SpawnStatus::BudgetExhausted { .. }
        ),
        "alive pid past deadline must classify budget-exhausted, got {exhausted:?}"
    );

    // Pure classification: exited (dead pid beats a stale deadline).
    let exited = pipeline_spawn::classify(
        false,
        30,
        &past.to_rfc3339(),
        chrono::Utc::now(),
        "stub pi started",
    );
    assert!(
        matches!(exited, pipeline_spawn::SpawnStatus::Exited { .. }),
        "dead pid must classify exited, got {exited:?}"
    );
    match &exited {
        pipeline_spawn::SpawnStatus::Exited { last_line } => {
            assert_eq!(
                last_line, "stub pi started",
                "exited must probe the log tail"
            );
        }
        _ => unreachable!(),
    }
}

// ---------------------------------------------------------------------------
// 1.4 Resume: killed run resumes from the named session (canon Invariant 6
// exception) with a fresh checkpoint, never re-prompting the full brief.
// ---------------------------------------------------------------------------

#[test]
fn spawn_resume_continues_named_session() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    let args_log = tmp.path().join("pi-args.log");

    // Spawn a session whose stub exits quickly (simulates interrupted run:
    // checkpoint remains, pid dead).
    let out = spawn_cmd(tmp.path(), &args_log)
        .args([
            "pipeline",
            "spawn",
            "subagent:test:kill",
            "full brief prompt",
        ])
        .env("PI_STUB_SLEEP", "0")
        .output()
        .expect("spawn should run");
    assert!(
        out.status.success(),
        "spawn failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let checkpoint_path = tmp
        .path()
        .join(".wai/pipeline-runs/spawns/subagent:test:kill.json");
    let checkpoint = pipeline_spawn::SpawnCheckpoint::load(&checkpoint_path).unwrap();
    assert!(
        wait_until(Duration::from_secs(10), || !pid_alive(checkpoint.pid)),
        "stub should exit (pid {} must die)",
        checkpoint.pid
    );

    // Resume: canon shape `pi -c -p -n "<name>-resume"` against the named
    // session — the full brief is NOT re-prompted.
    let out = spawn_cmd(tmp.path(), &args_log)
        .args(["pipeline", "spawn-resume", "subagent:test:kill"])
        .env("PI_STUB_SLEEP", "0")
        .output()
        .expect("resume should run");
    assert!(
        out.status.success(),
        "resume failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(
        wait_until(Duration::from_secs(10), || {
            let argv = read_args_log(&args_log);
            argv.lines().nth(1).is_some()
        }),
        "stub must be invoked a second time; argv log:\n{}",
        read_args_log(&args_log)
    );
    let argv = read_args_log(&args_log);
    let resume_line = argv.lines().nth(1).expect("second invocation recorded");
    assert!(
        resume_line.contains("-c") && resume_line.contains("-n subagent:test:kill-resume"),
        "resume must launch `pi -c -p -n <name>-resume`; got: {resume_line}"
    );
    assert!(
        !resume_line.contains("full brief prompt"),
        "resume must NEVER re-prompt the full brief; got: {resume_line}"
    );

    // Fresh checkpoint for the resumed session.
    let resume_checkpoint_path = tmp
        .path()
        .join(".wai/pipeline-runs/spawns/subagent:test:kill-resume.json");
    assert!(
        resume_checkpoint_path.exists(),
        "resume must write a fresh checkpoint at {resume_checkpoint_path:?}"
    );

    // Resume without a checkpoint is refused.
    let out = spawn_cmd(tmp.path(), &args_log)
        .args(["pipeline", "spawn-resume", "subagent:never:spawned"])
        .output()
        .expect("resume of unknown spawn should run");
    assert!(
        !out.status.success(),
        "resume without a checkpoint must be refused"
    );
}

// ---------------------------------------------------------------------------
// ro5u fixes: budget-0 rejection (M1) and stderr streamed to the log file
// (C1 — a piped-but-never-drained stderr would deadlock the spawn at ~64KB).
// ---------------------------------------------------------------------------

#[test]
fn spawn_rejects_zero_budget() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    let args_log = tmp.path().join("pi-args.log");

    let out = spawn_cmd(tmp.path(), &args_log)
        .args([
            "pipeline",
            "spawn",
            "subagent:test:zero",
            "prompt",
            "--budget",
            "0",
        ])
        .output()
        .expect("spawn should run");
    assert!(
        !out.status.success(),
        "zero budget must be refused (would kill the spawn instantly)"
    );
    assert!(
        !tmp.path()
            .join(".wai/pipeline-runs/spawns/subagent:test:zero.json")
            .exists(),
        "refused spawn must not write a checkpoint"
    );
}

#[test]
fn spawn_streams_stderr_to_log() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());
    let bin_dir = tmp.path().join("fake-bin");
    fs::create_dir_all(&bin_dir).unwrap();
    // A stub that writes 128KB to stderr (well past the 64KB pipe buffer that
    // would deadlock a piped stderr) then exits 0.
    let script_path = bin_dir.join("pi");
    fs::write(
        &script_path,
        "#!/bin/sh\nhead -c 131072 /dev/zero | tr '\\0' 'x' >&2\necho 'stub pi stderr done'\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        let _ = fs::set_permissions(&script_path, perms);
    }

    let mut cmd = hermetic_wai_cmd(tmp.path());
    let path = std::env::var("PATH").unwrap_or_default();
    cmd.env("PATH", format!("{}:{path}", bin_dir.display()));
    let out = cmd
        .args(["pipeline", "spawn", "subagent:test:stderr", "prompt"])
        .output()
        .expect("spawn should run");
    assert!(
        out.status.success(),
        "spawn failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let checkpoint = pipeline_spawn::SpawnCheckpoint::load(
        &tmp.path()
            .join(".wai/pipeline-runs/spawns/subagent:test:stderr.json"),
    )
    .unwrap();
    // Wait for the stub to finish, then the log must contain the full stderr
    // payload plus the stdout line — stderr reached the file, no pipe involved.
    assert!(
        wait_until(Duration::from_secs(10), || {
            fs::read_to_string(&checkpoint.log_path)
                .map(|l| l.contains("stub pi stderr done"))
                .unwrap_or(false)
        }),
        "log must contain stdout after 128KB of stderr (stderr must stream to \
         the log, not a never-drained pipe)"
    );
}
