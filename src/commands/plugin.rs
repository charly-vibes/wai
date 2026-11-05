//! `wai plugin` command: list, enable, disable, and trust plugins.

use miette::Result;
use owo_colors::OwoColorize;
use std::path::Path;

use crate::cli::PluginCommands;
use crate::context::CliContext;
use crate::context::current_context;
use crate::json::{PluginCommandInfo, PluginDetector, PluginListItem};
use crate::output::print_envelope_list;
use crate::plugin::{self, TrustEntry, TrustStore};

use super::require_project;

pub fn run(cmd: PluginCommands) -> Result<()> {
    let project_root = require_project()?;
    let context = current_context();

    match cmd {
        PluginCommands::List => cmd_list(&project_root, &context),
        PluginCommands::Enable { name } => cmd_enable(&project_root, &context, &name, true),
        PluginCommands::Disable { name } => cmd_enable(&project_root, &context, &name, false),
        PluginCommands::Trust {
            name,
            list,
            revoke,
            hook,
        } => cmd_trust(&project_root, &context, name, list, revoke, hook),
    }
}

fn cmd_list(project_root: &Path, context: &CliContext) -> Result<()> {
    if context.json {
        let plugins = plugin::detect_plugins(project_root)
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
    print_plugin_table(project_root);
    Ok(())
}

fn print_plugin_table(project_root: &Path) {
    println!();
    println!("  {} Plugins", "◆".cyan());
    println!();

    let plugins = plugin::detect_plugins(project_root);

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

        print_plugin_commands(&p.def.name, &p.def.commands);
        print_plugin_hooks(&p.def.hooks);
    }

    println!();
}

fn print_plugin_commands(plugin_name: &str, commands: &[plugin::PluginCommand]) {
    if commands.is_empty() {
        return;
    }
    for cmd in commands {
        println!(
            "      {} wai {} {}  — {}",
            "↳".dimmed(),
            plugin_name,
            cmd.name,
            cmd.description.dimmed()
        );
        if cmd.read_only {
            println!("        {} read-only", "·".dimmed());
        }
    }
}

fn print_plugin_hooks(hooks: &std::collections::HashMap<String, plugin::HookDef>) {
    if hooks.is_empty() {
        return;
    }
    let hook_names: Vec<&String> = hooks.keys().collect();
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

fn cmd_enable(project_root: &Path, context: &CliContext, name: &str, enabled: bool) -> Result<()> {
    let plugins = plugin::detect_plugins(project_root);
    if !plugins.iter().any(|p| p.def.name == name) {
        return Err(crate::error::WaiError::PluginNotFound {
            name: name.to_string(),
        }
        .into());
    }

    let mut config = crate::config::ProjectConfig::load(project_root)?;
    if let Some(entry) = config.plugins.iter_mut().find(|p| p.name == name) {
        entry.enabled = enabled;
    } else {
        config.plugins.push(crate::config::PluginConfig {
            name: name.to_string(),
            enabled,
            settings: toml::Table::new(),
        });
    }
    config.save(project_root)?;

    if context.json {
        return print_envelope_list(serde_json::json!({
            "plugin": name,
            "enabled": enabled,
        }));
    }
    if enabled {
        println!("  {} Plugin '{}' enabled", "✓".green(), name);
    } else {
        println!("  {} Plugin '{}' disabled", "○".dimmed(), name);
    }
    Ok(())
}

fn cmd_trust(
    project_root: &Path,
    context: &CliContext,
    name: Option<String>,
    list: bool,
    revoke: Option<String>,
    hook: Option<String>,
) -> Result<()> {
    if list {
        return trust_list(context);
    }
    if let Some(digest) = revoke {
        return trust_revoke(context, &digest);
    }
    trust_approve(project_root, context, name, hook)
}

fn trust_list(context: &CliContext) -> Result<()> {
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
    Ok(())
}

fn trust_revoke(context: &CliContext, digest: &str) -> Result<()> {
    let mut store = TrustStore::load();
    let removed = store.revoke(digest);
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
        println!("  {} No approval found for digest {}", "○".dimmed(), digest);
    }
    Ok(())
}

fn trust_approve(
    project_root: &Path,
    context: &CliContext,
    name: Option<String>,
    hook: Option<String>,
) -> Result<()> {
    // Approve path: require a plugin name
    let Some(name) = name else {
        return Err(crate::error::WaiError::PluginTrustError {
            message: "provide a plugin name to approve, or use --list/--revoke".to_string(),
        }
        .into());
    };

    let plugins = plugin::detect_plugins(project_root);
    let Some(target) = plugins.iter().find(|p| p.def.name == name) else {
        return Err(crate::error::WaiError::PluginNotFound { name }.into());
    };
    if target.source != plugin::PluginSource::Custom {
        return Err(crate::error::WaiError::PluginTrustError {
            message: format!("'{name}' is a built-in plugin and is always trusted"),
        }
        .into());
    }

    let mut store = TrustStore::load();
    let approved = approve_hooks(&mut store, target, &name, hook.as_deref());
    if approved.is_empty() {
        return Err(crate::error::WaiError::PluginTrustError {
            message: format!("plugin '{name}' defines no hooks"),
        }
        .into());
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

fn approve_hooks(
    store: &mut TrustStore,
    target: &plugin::ActivePlugin,
    name: &str,
    hook_filter: Option<&str>,
) -> Vec<(String, String)> {
    let mut approved = Vec::new();
    for hook_name in target.def.hooks.keys() {
        if let Some(h) = hook_filter
            && h != hook_name
        {
            continue;
        }
        if let Some(hook_def) = target.def.hooks.get(hook_name) {
            let digest = plugin::compute_hook_digest(&target.def, hook_name, hook_def);
            store.approve(TrustEntry {
                plugin_name: name.to_string(),
                hook_name: hook_name.clone(),
                digest: digest.clone(),
                command: hook_def.command.clone(),
                approved_at: chrono::Utc::now().to_rfc3339(),
            });
            approved.push((hook_name.clone(), digest));
        }
    }
    approved
}
