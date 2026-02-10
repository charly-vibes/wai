use miette::Result;
use owo_colors::OwoColorize;

use crate::cli::PluginCommands;
use crate::context::current_context;
use crate::json::{PluginCommandInfo, PluginDetector, PluginListItem};
use crate::output::print_json;
use crate::plugin;

use super::require_project;

pub fn run(cmd: PluginCommands) -> Result<()> {
    let project_root = require_project()?;
    let context = current_context();

    match cmd {
        PluginCommands::List => {
            if context.json {
                let plugins = plugin::detect_plugins(&project_root)
                    .into_iter()
                    .map(|p| PluginListItem {
                        name: p.def.name,
                        description: p.def.description,
                        status: if p.detected {
                            "detected".to_string()
                        } else {
                            "not found".to_string()
                        },
                        detected: p.detected,
                        detector: p.def.detector.as_ref().map(|detector| PluginDetector {
                            detector_type: detector.detector_type.clone(),
                            path: detector.path.clone(),
                        }),
                        commands: p
                            .def
                            .commands
                            .iter()
                            .map(|cmd| PluginCommandInfo {
                                name: cmd.name.clone(),
                                description: cmd.description.clone(),
                                read_only: cmd.read_only,
                            })
                            .collect(),
                        hooks: p.def.hooks.keys().cloned().collect(),
                    })
                    .collect::<Vec<_>>();
                return print_json(&plugins);
            }
            println!();
            println!("  {} Plugins", "◆".cyan());
            println!();

            let plugins = plugin::detect_plugins(&project_root);

            for p in &plugins {
                let status = if p.detected {
                    "detected".green().to_string()
                } else {
                    "not found".dimmed().to_string()
                };

                let source = if p.def.detector.is_some() {
                    "built-in"
                } else {
                    "custom"
                };

                println!(
                    "    {} {}  {}  [{}] ({})",
                    "•".dimmed(),
                    p.def.name.bold(),
                    p.def.description.dimmed(),
                    status,
                    source.dimmed()
                );

                // Show commands if any
                if !p.def.commands.is_empty() {
                    for cmd in &p.def.commands {
                        println!(
                            "      {} wai {} {}  — {}",
                            "↳".dimmed(),
                            p.def.name,
                            cmd.name,
                            cmd.description.dimmed()
                        );
                        if cmd.read_only {
                            println!("        {} read-only", "·".dimmed());
                        }
                    }
                }

                // Show hooks if any
                if !p.def.hooks.is_empty() {
                    let hook_names: Vec<&String> = p.def.hooks.keys().collect();
                    println!(
                        "      {} hooks: {}",
                        "↳".dimmed(),
                        hook_names
                            .iter()
                            .map(|h| h.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                            .dimmed()
                    );
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
