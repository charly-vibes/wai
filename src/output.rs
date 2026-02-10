use miette::{IntoDiagnostic, Result};
use serde::Serialize;

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
