use miette::Result;
use owo_colors::OwoColorize;

use crate::cli::PluginCommands;

use super::require_project;

pub fn run(cmd: PluginCommands) -> Result<()> {
    let project_root = require_project()?;

    match cmd {
        PluginCommands::List => {
            println!();
            println!("  {} Plugins", "◆".cyan());
            println!();

            // Built-in plugins with auto-detection
            let plugins = [
                ("beads", ".beads/", "Integration with beads issue tracker"),
                ("git", ".git/", "Git version control integration"),
                (
                    "openspec",
                    "openspec/",
                    "OpenSpec specification management",
                ),
            ];

            for (name, detector, description) in &plugins {
                let detected = project_root.join(detector).exists();
                let status = if detected {
                    "detected".green().to_string()
                } else {
                    "not found".dimmed().to_string()
                };
                println!(
                    "    {} {}  {}  [{}]",
                    "•".dimmed(),
                    name.bold(),
                    description.dimmed(),
                    status
                );
            }

            // Check for custom plugins in .wai/plugins/
            let plugins_dir = crate::config::plugins_dir(&project_root);
            if plugins_dir.exists()
                && let Ok(entries) = std::fs::read_dir(&plugins_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        if entry
                            .path()
                            .extension()
                            .and_then(|e| e.to_str())
                            .map(|e| e == "yml" || e == "yaml")
                            .unwrap_or(false)
                            && let Some(name) = entry.path().file_stem().and_then(|n| n.to_str()) {
                                println!(
                                    "    {} {}  {}",
                                    "•".dimmed(),
                                    name,
                                    "custom".dimmed()
                                );
                            }
                    }
                }

            println!();
            Ok(())
        }
        PluginCommands::Enable { name } => {
            println!("  Enabling plugin '{}'...", name);
            println!(
                "  {} Plugin enable/disable is not yet implemented",
                "○".dimmed()
            );
            Ok(())
        }
        PluginCommands::Disable { name } => {
            println!("  Disabling plugin '{}'...", name);
            println!(
                "  {} Plugin enable/disable is not yet implemented",
                "○".dimmed()
            );
            Ok(())
        }
    }
}
