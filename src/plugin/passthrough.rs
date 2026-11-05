//! Passthrough execution of plugin commands.

use super::*;

/// Execute a pass-through command.
pub fn execute_passthrough(
    project_root: &Path,
    passthrough: &str,
    extra_args: &[String],
) -> std::io::Result<std::process::ExitStatus> {
    check_passthrough_allowed(passthrough)?;

    let child = spawn_passthrough(project_root, passthrough, extra_args)?;
    let child = std::sync::Arc::new(std::sync::Mutex::new(child));
    let rx = wait_with_poller(&child);

    match rx.recv_timeout(PLUGIN_TIMEOUT) {
        Ok(result) => result,
        Err(_) => kill_on_timeout(&child, &rx),
    }
}

/// Validate context + parse the passthrough command line.
fn check_passthrough_allowed(passthrough: &str) -> std::io::Result<Vec<String>> {
    let context = current_context();
    let parts = shell_words::split(passthrough)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
    if parts.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Empty passthrough command",
        ));
    }
    if context.no_input {
        return Err(std::io::Error::other(WaiError::NonInteractive {
            message: "Plugin passthrough requires interactive input".to_string(),
        }));
    }
    if context.safe {
        return Err(std::io::Error::other(WaiError::SafeModeViolation {
            action: "plugin passthrough".to_string(),
        }));
    }
    Ok(parts)
}

fn spawn_passthrough(
    project_root: &Path,
    passthrough: &str,
    extra_args: &[String],
) -> std::io::Result<std::process::Child> {
    let parts = check_passthrough_allowed(passthrough)?;
    let mut cmd = Command::new(&parts[0]);
    cmd.args(&parts[1..]);
    cmd.args(extra_args);
    cmd.current_dir(project_root);
    cmd.spawn()
}

/// Spawn a polling thread that reports exit status over the returned receiver.
type ExitReceiver = mpsc::Receiver<std::io::Result<std::process::ExitStatus>>;

pub(super) fn wait_with_poller(
    child: &std::sync::Arc<std::sync::Mutex<std::process::Child>>,
) -> ExitReceiver {
    use std::sync::Arc;
    let child_thread = Arc::clone(child);
    let (tx, rx): (mpsc::Sender<std::io::Result<std::process::ExitStatus>>, _) = mpsc::channel();
    std::thread::spawn(move || {
        // Poll try_wait() in a loop so the mutex is released between iterations,
        // allowing the timeout path to acquire it for kill().
        // IMPORTANT: extract the result before the match so the MutexGuard
        // is dropped before the sleep (Rust edition 2024 scoping rules).
        loop {
            let exited = child_thread.lock().unwrap().try_wait().ok().flatten();
            match exited {
                Some(status) => {
                    let _ = tx.send(Ok(status));
                    break;
                }
                None => {
                    std::thread::sleep(Duration::from_millis(50));
                    continue;
                }
            }
        }
    });
    rx
}

/// Timed out — kill the child and report an error.
fn kill_on_timeout(
    child: &std::sync::Arc<std::sync::Mutex<std::process::Child>>,
    rx: &ExitReceiver,
) -> std::io::Result<std::process::ExitStatus> {
    // The waiter thread releases the mutex between try_wait() calls,
    // so we can acquire it here (unlike the old wait() which held it).
    let _ = child.lock().unwrap().kill();
    // Wait briefly for the waiter thread to reap the child.
    let _ = rx.recv_timeout(Duration::from_secs(2));
    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "plugin command exceeded 30-second timeout",
    ))
}
