use genesis::envelope::{Envelope, EnvelopeKind, HintEntry, Warning};
use miette::{IntoDiagnostic, Result};
use serde::Serialize;

/// Print a payload wrapped in the genesis shared envelope.
pub fn print_envelope<T: Serialize>(
    kind: EnvelopeKind,
    data: T,
    warnings: Vec<Warning>,
    hints: Vec<HintEntry>,
) -> Result<()> {
    let envelope = Envelope::success(kind, data, warnings, hints);
    print_json(&envelope)
}

/// Convenience: print a success response with no warnings or hints.
pub fn print_envelope_ok<T: Serialize>(data: T) -> Result<()> {
    print_envelope(EnvelopeKind::Ok, data, vec![], vec![])
}

/// Convenience: print a list response with no warnings or hints.
pub fn print_envelope_list<T: Serialize>(data: T) -> Result<()> {
    print_envelope(EnvelopeKind::List, data, vec![], vec![])
}

/// Convenience: print a doctor diagnostic response.
pub fn print_envelope_doctor<T: Serialize>(data: T) -> Result<()> {
    print_envelope(EnvelopeKind::Doctor, data, vec![], vec![])
}

/// Convenience: print a check result response.
pub fn print_envelope_check<T: Serialize>(data: T) -> Result<()> {
    print_envelope(EnvelopeKind::Check, data, vec![], vec![])
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
