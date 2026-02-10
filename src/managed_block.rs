use std::path::Path;

const WAI_START: &str = "<!-- WAI:START -->";
const WAI_END: &str = "<!-- WAI:END -->";

pub fn wai_block_content() -> String {
    format!(
        r#"{start}
# Wai — Workflow Context

This project uses **wai** for workflow management and decision tracking.
Run `wai status` to see project state, or `wai --help` for all commands.

**Quick reference:**
```bash
wai status                    # Project status and next steps
wai phase show                # Current project phase
wai new project "name"        # Create a new project
wai add research "notes"      # Add research notes
wai add plan "plan"           # Add a plan document
wai add design "design"       # Add a design document
wai search "query"            # Search across all artifacts
wai handoff create <project>  # Generate handoff document
wai sync                      # Sync agent configs
```

The `.wai/` directory contains project artifacts organized by the PARA method
(Projects, Areas, Resources, Archives). Do not edit `.wai/config.toml` directly.

Keep this managed block so `wai init` can refresh the instructions.

{end}"#,
        start = WAI_START,
        end = WAI_END
    )
}

pub fn inject_managed_block(path: &Path) -> Result<InjectResult, std::io::Error> {
    let block = wai_block_content();

    if path.exists() {
        let content = std::fs::read_to_string(path)?;

        if let (Some(start_idx), Some(end_idx)) = (content.find(WAI_START), content.find(WAI_END)) {
            let end_idx = end_idx + WAI_END.len();
            let mut new_content = String::with_capacity(content.len());
            new_content.push_str(&content[..start_idx]);
            new_content.push_str(&block);
            new_content.push_str(&content[end_idx..]);
            std::fs::write(path, new_content)?;
            Ok(InjectResult::Updated)
        } else {
            let mut new_content = block;
            new_content.push_str("\n\n");
            new_content.push_str(&content);
            std::fs::write(path, new_content)?;
            Ok(InjectResult::Prepended)
        }
    } else {
        std::fs::write(path, &block)?;
        Ok(InjectResult::Created)
    }
}

pub fn has_managed_block(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    match std::fs::read_to_string(path) {
        Ok(content) => content.contains(WAI_START) && content.contains(WAI_END),
        Err(_) => false,
    }
}

pub enum InjectResult {
    Created,
    Prepended,
    Updated,
}

impl InjectResult {
    pub fn description(&self, filename: &str) -> String {
        match self {
            InjectResult::Created => format!("Created {} with wai instructions", filename),
            InjectResult::Prepended => {
                format!("Added wai instructions to existing {}", filename)
            }
            InjectResult::Updated => format!("Updated wai instructions in {}", filename),
        }
    }
}
