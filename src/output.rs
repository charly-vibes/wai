use genesis::envelope::{Envelope, EnvelopeKind, HintEntry, Warning};
use miette::{IntoDiagnostic, Result};
use serde::Serialize;

/// Print a payload wrapped in the genesis shared envelope.
#[allow(dead_code)]
pub fn print_envelope<T: Serialize>(
    kind: EnvelopeKind,
    data: T,
    warnings: Vec<Warning>,
    hints: Vec<HintEntry>,
) -> Result<()> {
    let envelope = Envelope::success(kind, data, warnings, hints);
    print_json(&envelope)
}

pub fn print_json<T: Serialize>(payload: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(payload).into_diagnostic()?;
    println!("{}", json);
    Ok(())
}

pub fn print_json_line<T: Serialize>(payload: &T) -> Result<()> {
    let json = serde_json::to_string(payload).into_diagnostic()?;
    println!("{}", json);
    Ok(())
}
