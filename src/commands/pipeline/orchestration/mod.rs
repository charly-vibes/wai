//! Pipeline orchestration: start, advance, status, GC, wait, and spawn.

mod advance;
mod current;
mod gc;
mod spawn;
mod start;
mod wait;

pub(crate) use advance::{cmd_approve, cmd_bypass, cmd_lock, cmd_next, record_handoff_artifact};
pub(crate) use current::{
    clear_complete_pipeline_run, pipeline_current_status, resolve_active_run_id, run_is_incomplete,
};
pub(crate) use gc::cmd_gc;
pub(crate) use gc::{find_stale_runs, stale_threshold_days};
pub(crate) use spawn::{cmd_spawn, cmd_spawn_resume, cmd_spawn_status};
pub(crate) use start::cmd_start;
pub(crate) use wait::{cmd_preflight, cmd_wait};
