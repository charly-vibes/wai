use std::cell::RefCell;

use crate::error::WaiError;

#[derive(Debug, Clone, Copy)]
pub struct CliContext {
    pub json: bool,
    pub no_input: bool,
    pub yes: bool,
    pub safe: bool,
    #[allow(dead_code)]
    pub verbose: u8,
}

thread_local! {
    static CLI_CONTEXT: RefCell<Option<CliContext>> = const { RefCell::new(None) };
}

pub fn set_context(context: CliContext) {
    CLI_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(context);
    });
}

pub fn current_context() -> CliContext {
    CLI_CONTEXT.with(|ctx| {
        ctx.borrow().unwrap_or(CliContext {
            json: false,
            no_input: false,
            yes: false,
            safe: false,
            verbose: 0,
        })
    })
}

pub fn require_safe_mode(action: &str) -> Result<(), WaiError> {
    let ctx = current_context();
    if ctx.safe {
        return Err(WaiError::SafeModeViolation {
            action: action.to_string(),
        });
    }
    Ok(())
}
