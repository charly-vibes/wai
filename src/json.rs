use serde::Serialize;

use crate::config::CONFIG_DIR;

#[derive(Debug, Serialize)]
pub struct Suggestion {
    pub label: String,
    pub command: String,
}

#[derive(Debug, Serialize)]
pub struct WelcomePayload {
    pub welcome: String,
    pub project_detected: bool,
    pub suggestions: Vec<Suggestion>,
    pub help_hint: String,
}

#[derive(Debug, Serialize)]
pub struct StatusPlugin {
    pub name: String,
    pub status: String,
    pub detected: bool,
}

#[derive(Debug, Serialize)]
pub struct StatusProject {
    pub name: String,
    pub phase: String,
}

#[derive(Debug, Serialize)]
pub struct StatusPayload {
    pub project_root: String,
    pub projects: Vec<StatusProject>,
    pub plugins: Vec<StatusPlugin>,
    pub hook_outputs: Vec<HookOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openspec: Option<StatusOpenSpec>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Serialize)]
pub struct StatusOpenSpec {
    pub specs: Vec<String>,
    pub changes: Vec<StatusChange>,
}

#[derive(Debug, Serialize)]
pub struct StatusChange {
    pub name: String,
    pub done: usize,
    pub total: usize,
    pub sections: Vec<StatusChangeSection>,
}

#[derive(Debug, Serialize)]
pub struct StatusChangeSection {
    pub name: String,
    pub done: usize,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct HookOutput {
    pub label: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub path: String,
    pub line_number: usize,
    pub line: String,
    pub context: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchPayload {
    pub query: String,
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Serialize)]
pub struct TimelineEntry {
    pub date: String,
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub title: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct TimelinePayload {
    pub project: String,
    pub entries: Vec<TimelineEntry>,
}

#[derive(Debug, Serialize)]
pub struct PluginListItem {
    pub name: String,
    pub description: String,
    pub status: String,
    pub detected: bool,
    pub detector: Option<PluginDetector>,
    pub commands: Vec<PluginCommandInfo>,
    pub hooks: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PluginDetector {
    pub detector_type: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct PluginCommandInfo {
    pub name: String,
    pub description: String,
    pub read_only: bool,
}

#[derive(Debug, Serialize)]
pub struct ShowEntry {
    pub name: String,
    pub phase: Option<String>,
    pub artifact_count: usize,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct ShowPayload {
    pub projects: Vec<ShowEntry>,
    pub areas: Vec<ShowEntry>,
    pub resources: Vec<ShowEntry>,
    pub archives: Vec<ShowEntry>,
}

#[derive(Debug, Serialize)]
pub struct ShowItemPayload {
    pub name: String,
    pub category: String,
    pub phase: Option<String>,
    pub path: String,
    pub contents: Vec<ShowItemEntry>,
}

#[derive(Debug, Serialize)]
pub struct ShowItemEntry {
    pub name: String,
    pub kind: String,
    pub item_count: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct BeadsSummary {
    pub open: u64,
    pub ready: u64,
}

#[derive(Debug, Serialize)]
pub struct OpenspecEntry {
    pub name: String,
    pub done: usize,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct PrimePayload {
    pub project: Option<String>,
    pub phase: Option<String>,
    pub resume: bool,
    pub handoff_summary: Option<String>,
    pub next_steps: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beads: Option<BeadsSummary>,
    pub openspec: Vec<OpenspecEntry>,
}

#[allow(dead_code)]
pub fn sanitize_path(path: &std::path::Path, project_root: &std::path::Path) -> String {
    if let Ok(relative) = path.strip_prefix(project_root) {
        let root = std::path::Path::new(CONFIG_DIR);
        if let Ok(rel_to_wai) = relative.strip_prefix(root) {
            return rel_to_wai.display().to_string();
        }
        return relative.display().to_string();
    }
    path.display().to_string()
}
