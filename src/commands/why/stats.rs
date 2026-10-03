//! Verbose run statistics rendering for `wai why`.

use owo_colors::OwoColorize;

/// Build the verbose diagnostic lines shown after an LLM call.
pub fn verbose_stats_lines(
    verbose: u8,
    elapsed_ms: u128,
    prompt: &str,
    response: &str,
    model_id: &str,
) -> Vec<String> {
    if verbose == 0 {
        return vec![];
    }

    let elapsed_s = elapsed_ms as f64 / 1000.0;
    let mut lines = vec![format!("  {} {:.2}s", "○".dimmed(), elapsed_s)];

    if verbose >= 2 {
        let input_chars = prompt.len();
        let output_chars = response.len();
        let input_tokens = input_chars / 4;
        let output_tokens = output_chars / 4;
        lines.push(format!(
            "  {} prompt {} chars (~{} tokens), response {} chars (~{} tokens)",
            "◇".dimmed(),
            input_chars,
            input_tokens,
            output_chars,
            output_tokens,
        ));
        if let Some(cost) = crate::llm::estimate_cost(model_id, input_chars, output_chars) {
            lines.push(format!("  {} ~${:.4} estimated", "◇".dimmed(), cost));
        }
    }

    if verbose >= 3 {
        lines.push(String::new());
        lines.push(format!("  {} Full prompt:", "◇".dimmed()));
        lines.push("  ─────────────────────────────────────────".to_string());
        for line in prompt.lines() {
            lines.push(format!("  {}", line));
        }
        lines.push("  ─────────────────────────────────────────".to_string());
    }

    lines
}

pub(crate) fn print_verbose_stats(
    verbose: u8,
    elapsed_ms: u128,
    prompt: &str,
    response: &str,
    model_id: &str,
) {
    let lines = verbose_stats_lines(verbose, elapsed_ms, prompt, response, model_id);
    if !lines.is_empty() {
        println!();
        for line in lines {
            println!("{}", line);
        }
    }
}

// ── Command entry point ───────────────────────────────────────────────────────
