use miette::Result;
use owo_colors::OwoColorize;

use crate::cli::PluginCommands;
use crate::context::current_context;
use crate::json::{PluginCommandInfo, PluginDetector, PluginListItem};
use crate::output::print_envelope_list;
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
                return print_envelope_list(plugins);
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
            let plugins = plugin::detect_plugins(&project_root);
            if !plugins.iter().any(|p| p.def.name == name) {
                return Err(crate::error::WaiError::PluginNotFound { name }.into());
            }

            let mut config = crate::config::ProjectConfig::load(&project_root)?;
            if let Some(entry) = config.plugins.iter_mut().find(|p| p.name == name) {
                entry.enabled = true;
            } else {
                config.plugins.push(crate::config::PluginConfig {
                    name: name.clone(),
                    enabled: true,
                    settings: toml::Table::new(),
                });
            }
            config.save(&project_root)?;

            if context.json {
                return print_envelope_list(serde_json::json!({
                    "plugin": name,
                    "enabled": true,
                }));
            }
            println!("  {} Plugin '{}' enabled", "✓".green(), name);
            Ok(())
        }
        PluginCommands::Disable { name } => {
            let plugins = plugin::detect_plugins(&project_root);
            if !plugins.iter().any(|p| p.def.name == name) {
                return Err(crate::error::WaiError::PluginNotFound { name }.into());
            }

            let mut config = crate::config::ProjectConfig::load(&project_root)?;
            if let Some(entry) = config.plugins.iter_mut().find(|p| p.name == name) {
                entry.enabled = false;
            } else {
                config.plugins.push(crate::config::PluginConfig {
                    name: name.clone(),
                    enabled: false,
                    settings: toml::Table::new(),
                });
            }
            config.save(&project_root)?;

            if context.json {
                return print_envelope_list(serde_json::json!({
                    "plugin": name,
                    "enabled": false,
                }));
            }
            println!("  {} Plugin '{}' disabled", "○".dimmed(), name);
            Ok(())
        }
        PluginCommands::Trust {
            name,
            list,
            revoke,
            hook,
        } => {
            use crate::plugin::{TrustEntry, TrustStore};

            if list {
                let store = TrustStore::load();
                let entries = store.list();
                if context.json {
                    return print_envelope_list(entries);
                }
                println!();
                println!("  {} Approved plugin hooks", "◆".cyan());
                println!();
                if entries.is_empty() {
                    println!("    No approved hooks yet.");
                    println!("    Approve a plugin with: `wai plugin trust <name>`");
                } else {
                    for e in entries {
                        println!("    {} {}", "•".dimmed(), e.digest.dimmed());
                        println!(
                            "        {} plugin: {}  hook: {}  command: {}",
                            "↳".dimmed(),
                            e.plugin_name,
                            e.hook_name,
                            e.command.dimmed()
                        );
                    }
                }
                println!();
                return Ok(());
            }

            if let Some(digest) = revoke {
                let mut store = TrustStore::load();
                let removed = store.revoke(&digest);
                store
                    .save()
                    .map_err(|e| crate::error::WaiError::PluginTrustError { message: e })?;
                if context.json {
                    return print_envelope_list(serde_json::json!({
                        "revoked": removed,
                        "digest": digest,
                    }));
                }
                if removed {
                    println!(
                        "  {} Revoked approval for digest {}",
                        "✓".green(),
                        digest.dimmed()
                    );
                } else {
                    println!(
                        "  {} No approval found for digest {}",
                        "○".dimmed(),
                        digest.dimmed()
                    );
                }
                return Ok(());
            }

            // Approve path: require a plugin name
            let Some(name) = name else {
                return Err(crate::error::WaiError::PluginTrustError {
                    message: "provide a plugin name to approve, or use --list/--revoke".to_string(),
                }
                .into());
            };

            let plugins = plugin::detect_plugins(&project_root);
            let target = plugins.iter().find(|p| p.def.name == name);
            let Some(target) = target else {
                return Err(crate::error::WaiError::PluginNotFound { name }.into());
            };
            if target.source != plugin::PluginSource::Custom {
                return Err(crate::error::WaiError::PluginTrustError {
                    message: format!("'{name}' is a built-in plugin and is always trusted"),
                }
                .into());
            }

            let mut store = TrustStore::load();
            let hook_names: Vec<String> = target.def.hooks.keys().cloned().collect();
            if hook_names.is_empty() {
                return Err(crate::error::WaiError::PluginTrustError {
                    message: format!("plugin '{name}' defines no hooks"),
                }
                .into());
            }

            let mut approved = Vec::new();
            for hook_name in &hook_names {
                if let Some(h) = hook.as_ref()
                    && h != hook_name
                {
                    continue;
                }
                if let Some(hook_def) = target.def.hooks.get(hook_name) {
                    let digest = plugin::compute_hook_digest(&target.def, hook_name, hook_def);
                    store.approve(TrustEntry {
                        plugin_name: name.clone(),
                        hook_name: hook_name.clone(),
                        digest: digest.clone(),
                        command: hook_def.command.clone(),
                        approved_at: chrono::Utc::now().to_rfc3339(),
                    });
                    approved.push((hook_name.clone(), digest));
                }
            }
            store
                .save()
                .map_err(|e| crate::error::WaiError::PluginTrustError { message: e })?;

            if context.json {
                return print_envelope_list(serde_json::json!({
                    "plugin": name,
                    "approved": approved
                        .iter()
                        .map(|(h, d)| serde_json::json!({"hook": h, "digest": d}))
                        .collect::<Vec<_>>(),
                }));
            }
            for (hook_name, digest) in &approved {
                println!(
                    "  {} Trusted hook '{}' of plugin '{}' (digest {})",
                    "✓".green(),
                    hook_name,
                    name,
                    &digest[..digest.len().min(16)]
                );
            }
            Ok(())
        }
    }
}
