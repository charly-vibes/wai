use std::path::Path;

#[derive(Debug, Clone)]
pub struct TaskSection {
    pub name: String,
    pub done: usize,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct ChangeStatus {
    pub name: String,
    pub sections: Vec<TaskSection>,
    pub done: usize,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct OpenSpecStatus {
    pub specs: Vec<String>,
    pub changes: Vec<ChangeStatus>,
}

pub fn read_status(project_root: &Path) -> Option<OpenSpecStatus> {
    let openspec_dir = project_root.join("openspec");
    if !openspec_dir.is_dir() {
        return None;
    }

    let specs = scan_specs(&openspec_dir.join("specs"));
    let changes = scan_changes(&openspec_dir.join("changes"));

    Some(OpenSpecStatus { specs, changes })
}

fn scan_specs(specs_dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let entries = match std::fs::read_dir(specs_dir) {
        Ok(entries) => entries,
        Err(_) => return names,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    names
}

fn scan_changes(changes_dir: &Path) -> Vec<ChangeStatus> {
    let mut results = Vec::new();
    let entries = match std::fs::read_dir(changes_dir) {
        Ok(entries) => entries,
        Err(_) => return results,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };
        if name == "archive" {
            continue;
        }
        let tasks_path = entry.path().join("tasks.md");
        let sections = parse_tasks(&tasks_path);
        let done: usize = sections.iter().map(|s| s.done).sum();
        let total: usize = sections.iter().map(|s| s.total).sum();
        results.push(ChangeStatus {
            name,
            sections,
            done,
            total,
        });
    }
    results.sort_by(|a, b| a.name.cmp(&b.name));
    results
}

fn parse_tasks(tasks_path: &Path) -> Vec<TaskSection> {
    let content = match std::fs::read_to_string(tasks_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut sections = Vec::new();
    let mut current_name: Option<String> = None;
    let mut done: usize = 0;
    let mut total: usize = 0;

    for line in content.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            if let Some(name) = current_name.take() {
                sections.push(TaskSection { name, done, total });
            }
            let heading = heading.trim();
            let section_name = heading
                .find(". ")
                .map(|i| heading[i + 2..].to_string())
                .unwrap_or_else(|| heading.to_string());
            current_name = Some(section_name);
            done = 0;
            total = 0;
        } else {
            let trimmed = line.trim_start();
            if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
                total += 1;
                done += 1;
            } else if trimmed.starts_with("- [ ] ") {
                total += 1;
            }
        }
    }

    if let Some(name) = current_name {
        sections.push(TaskSection { name, done, total });
    }

    sections
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn parse_tasks_empty_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("tasks.md");
        fs::write(&path, "").unwrap();
        let sections = parse_tasks(&path);
        assert!(sections.is_empty());
    }

    #[test]
    fn parse_tasks_missing_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("tasks.md");
        let sections = parse_tasks(&path);
        assert!(sections.is_empty());
    }

    #[test]
    fn parse_tasks_all_checked() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("tasks.md");
        fs::write(
            &path,
            "## 1. Setup\n\n- [x] 1.1 Do thing\n- [x] 1.2 Do other\n",
        )
        .unwrap();
        let sections = parse_tasks(&path);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].name, "Setup");
        assert_eq!(sections[0].done, 2);
        assert_eq!(sections[0].total, 2);
    }

    #[test]
    fn parse_tasks_mixed() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("tasks.md");
        fs::write(
            &path,
            "## 1. Alpha\n\n- [x] done\n- [ ] todo\n\n## 2. Beta\n\n- [ ] a\n- [ ] b\n- [x] c\n",
        )
        .unwrap();
        let sections = parse_tasks(&path);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].name, "Alpha");
        assert_eq!(sections[0].done, 1);
        assert_eq!(sections[0].total, 2);
        assert_eq!(sections[1].name, "Beta");
        assert_eq!(sections[1].done, 1);
        assert_eq!(sections[1].total, 3);
    }

    #[test]
    fn parse_tasks_multi_section() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("tasks.md");
        fs::write(
            &path,
            "## 1. First\n\n- [ ] a\n\n## 2. Second\n\n- [x] b\n\n## 3. Third\n\n- [ ] c\n- [ ] d\n- [x] e\n",
        )
        .unwrap();
        let sections = parse_tasks(&path);
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[2].name, "Third");
        assert_eq!(sections[2].done, 1);
        assert_eq!(sections[2].total, 3);
    }

    #[test]
    fn read_status_returns_none_without_openspec() {
        let tmp = TempDir::new().unwrap();
        assert!(read_status(tmp.path()).is_none());
    }

    #[test]
    fn read_status_scans_specs_and_changes() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        fs::create_dir_all(root.join("openspec/specs/auth")).unwrap();
        fs::create_dir_all(root.join("openspec/specs/billing")).unwrap();
        fs::create_dir_all(root.join("openspec/changes/add-feature")).unwrap();
        fs::create_dir_all(root.join("openspec/changes/archive/old")).unwrap();
        fs::write(
            root.join("openspec/changes/add-feature/tasks.md"),
            "## 1. Work\n\n- [x] done\n- [ ] todo\n",
        )
        .unwrap();

        let status = read_status(root).unwrap();
        assert_eq!(status.specs, vec!["auth", "billing"]);
        assert_eq!(status.changes.len(), 1);
        assert_eq!(status.changes[0].name, "add-feature");
        assert_eq!(status.changes[0].done, 1);
        assert_eq!(status.changes[0].total, 2);
    }
}
