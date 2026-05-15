use cliclack::log;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::fs;

use crate::config::pipelines_dir;
use crate::context::require_safe_mode;

use super::definition::validate_pipeline_name;
use crate::commands::require_project;

// ─── init ─────────────────────────────────────────────────────────────────────

pub(super) fn cmd_init(name: &str) -> Result<()> {
    let project_root = require_project()?;
    require_safe_mode("pipeline init")?;

    validate_pipeline_name(name)?;

    let pipelines = pipelines_dir(&project_root);
    fs::create_dir_all(&pipelines).into_diagnostic()?;

    let file_path = pipelines.join(format!("{}.toml", name));
    if file_path.exists() {
        miette::bail!(
            "Pipeline '{}' already exists: {}",
            name,
            file_path.display()
        );
    }

    // Check for built-in template, otherwise use generic scaffold
    let template = if let Some(builtin) = get_builtin_template(name) {
        builtin.to_string()
    } else {
        // The template uses {topic} as the variable substitution placeholder.
        // We build this as a plain string (no format!) to avoid escaping collisions.
        let tmpl = concat!(
            "# Step prompts are navigation hints. Instructions for HOW to do the\n",
            "# work belong in skills.\n",
            "[pipeline]\n",
            "name = \"PIPELINE_NAME\"\n",
            "description = \"Describe what this pipeline does\"\n",
            "\n",
            "[[steps]]\n",
            "id = \"step-one\"\n",
            "prompt = \"\"\"\n",
            "{topic}: TODO describe step one task.\n",
            "Use skill `<skill-name>` if available.\n",
            "Record findings: `wai add research \"...\"`\n",
            "Advance: `wai pipeline next`\n",
            "\"\"\"\n",
            "\n",
            "[[steps]]\n",
            "id = \"step-two\"\n",
            "prompt = \"\"\"\n",
            "{topic}: TODO describe step two task.\n",
            "Use skill `<skill-name>` if available.\n",
            "Record decisions: `wai add design \"...\"`\n",
            "Advance: `wai pipeline next`\n",
            "\"\"\"\n",
        );
        tmpl.replace("PIPELINE_NAME", name)
    };

    fs::write(&file_path, template).into_diagnostic()?;

    // Scaffold oracles directory with README if not present
    let oracles_dir = crate::config::wai_dir(&project_root)
        .join("resources")
        .join("oracles");
    fs::create_dir_all(&oracles_dir).into_diagnostic()?;
    let readme_path = oracles_dir.join("README.md");
    if !readme_path.exists() {
        let readme = "# Oracle Scripts\n\n\
            Oracle scripts are user-defined validators run during pipeline gate checks.\n\n\
            ## Convention\n\n\
            - Place scripts here: `.wai/resources/oracles/<name>[.sh|.py]`\n\
            - Scripts must be executable (`chmod +x`)\n\
            - Exit 0 = pass, non-zero = fail\n\
            - Write failure reasons to stderr\n\
            - Default scope: one invocation per artifact (`<script> <artifact-path>`)\n\
            - Cross-artifact scope: `scope = \"all\"` passes all paths at once\n\n\
            ## Example\n\n\
            ```bash\n\
            #!/usr/bin/env bash\n\
            # example-check.sh — verify artifact contains required sections\n\
            grep -q '## Constraints' \"$1\" || { echo 'Missing ## Constraints section' >&2; exit 1; }\n\
            ```\n\n\
            Configure in your pipeline TOML:\n\
            ```toml\n\
            [[steps.gate.oracles]]\n\
            name = \"example-check\"\n\
            ```\n";
        fs::write(&readme_path, readme).into_diagnostic()?;
    }
    let example_path = oracles_dir.join("example-check.sh");
    if !example_path.exists() {
        let example = "#!/usr/bin/env bash\n\
            # example-check.sh — sample oracle that verifies artifact has content\n\
            # Exit 0 = pass, non-zero = fail. Stderr is shown on failure.\n\
            set -euo pipefail\n\n\
            FILE=\"$1\"\n\
            if [ ! -s \"$FILE\" ]; then\n\
            \x20   echo \"Artifact is empty: $FILE\" >&2\n\
            \x20   exit 1\n\
            fi\n";
        fs::write(&example_path, example).into_diagnostic()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&example_path).into_diagnostic()?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&example_path, perms).into_diagnostic()?;
        }
    }

    log::success(format!("Created pipeline: {}", file_path.display())).into_diagnostic()?;
    println!(
        "  {} Edit the prompts, then start with: wai pipeline start {} --topic=<your-topic>",
        "→".cyan(),
        name
    );
    Ok(())
}

// ─── Built-in templates ───────────────────────────────────────────────────────

/// Returns a built-in template if one exists for the given name.
pub(super) fn get_builtin_template(name: &str) -> Option<&'static str> {
    if !crate::config::BUILTIN_PIPELINE_TEMPLATES.contains(&name) {
        return None;
    }

    match name {
        "scientific-research" => Some(include_str!("../../templates/scientific-research.toml")),
        "tdd-ro5" => Some(include_str!("../../templates/tdd-ro5.toml")),
        _ => None,
    }
}

/// Returns names of all available built-in templates.
#[cfg(test)]
pub(super) fn builtin_template_names() -> &'static [&'static str] {
    crate::config::BUILTIN_PIPELINE_TEMPLATES
}
