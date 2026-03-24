use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;

use crate::commands::resource::parse_skill_frontmatter;
use crate::config::{SKILLS_DIR, agent_config_dir};
use crate::context::current_context;
use crate::output::print_json;

const SKILL_RULE_OF_5: (&str, &str) = (
    "rule-of-5-universal",
    include_str!("../../.wai/resources/agent-config/skills/rule-of-5-universal/SKILL.md"),
);

const SKILL_COMMIT: (&str, &str) = (
    "commit",
    include_str!("../../.wai/resources/agent-config/skills/commit/SKILL.md"),
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Pass,
    Info,
}

#[derive(Serialize)]
struct CheckResult {
    name: String,
    status: Status,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    success_criteria: Option<String>,
    suggestion: Option<String>,
}

#[derive(Serialize)]
struct WayPayload {
    checks: Vec<CheckResult>,
    summary: Summary,
}

#[derive(Debug, Clone, Serialize)]
struct Summary {
    pass: usize,
    recommendations: usize,
}

pub fn run(topic: Option<String>, fix: Option<String>) -> Result<()> {
    // way works in any directory - doesn't require .wai/ initialization
    let repo_root = std::env::current_dir()
        .map_err(|e| miette::miette!("Cannot determine current directory: {}", e))?;

    if let Some(ref topic) = topic {
        return print_topic_guide(topic, &repo_root);
    }

    if let Some(target) = fix {
        return match target.as_str() {
            "skills" => fix_skills(&repo_root),
            other => miette::bail!("Unknown fix target '{}'. Available: skills", other),
        };
    }

    let context = current_context();

    let checks = vec![
        check_task_runner(&repo_root),
        check_git_hooks(&repo_root),
        check_editorconfig(&repo_root),
        check_typos(&repo_root),
        check_vale(&repo_root),
        check_shell_linting(&repo_root),
        check_documentation(&repo_root),
        check_ai_instructions(&repo_root),
        check_llm_txt(&repo_root),
        check_agent_skills(&repo_root),
        check_gh_cli(),
        check_ci_cd(&repo_root),
        check_devcontainer(&repo_root),
        check_release_pipeline(&repo_root),
        check_test_coverage(&repo_root),
        check_beads(&repo_root),
        check_openspec(&repo_root),
    ];

    let summary = Summary {
        pass: checks.iter().filter(|c| c.status == Status::Pass).count(),
        recommendations: checks.iter().filter(|c| c.status == Status::Info).count(),
    };

    if context.json {
        let payload = WayPayload { checks, summary };
        print_json(&payload)?;
    } else {
        render_human(&checks, &summary, context.verbose)?;
    }

    // Always exit 0 - these are recommendations, not requirements
    Ok(())
}

const AVAILABLE_TOPICS: &[&str] = &[
    "ai", "ci", "coverage", "devxp", "docs", "gh", "hooks", "issues", "specs",
];

fn print_topic_guide(topic: &str, repo_root: &Path) -> Result<()> {
    let guide = match topic {
        "ai" => guide_ai(repo_root),
        "ci" => guide_ci(repo_root),
        "coverage" => guide_coverage(repo_root),
        "devxp" => guide_devxp(repo_root),
        "docs" => guide_docs(repo_root),
        "gh" => guide_gh(),
        "hooks" => guide_hooks(repo_root),
        "issues" => guide_issues(repo_root),
        "specs" => guide_specs(repo_root),
        other => {
            miette::bail!(
                "Unknown topic '{}'. Available: {}",
                other,
                AVAILABLE_TOPICS.join(", ")
            );
        }
    };

    println!("{}", guide);
    Ok(())
}

fn guide_ci(repo_root: &Path) -> String {
    // Detect current state to give the LLM context
    let has_github_actions = repo_root.join(".github/workflows").is_dir();
    let has_gitlab_ci = repo_root.join(".gitlab-ci.yml").exists();
    let has_circleci = repo_root.join(".circleci/config.yml").exists();

    let current_state = if has_github_actions {
        let count = std::fs::read_dir(repo_root.join(".github/workflows"))
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .is_some_and(|ext| ext == "yml" || ext == "yaml")
                    })
                    .count()
            })
            .unwrap_or(0);
        format!(
            "GitHub Actions detected ({} workflow(s) in .github/workflows/)",
            count
        )
    } else if has_gitlab_ci {
        "GitLab CI detected (.gitlab-ci.yml)".to_string()
    } else if has_circleci {
        "CircleCI detected (.circleci/config.yml)".to_string()
    } else {
        "No CI configuration detected".to_string()
    };

    // Detect language/ecosystem for tailored advice
    let has_cargo = repo_root.join("Cargo.toml").exists();
    let has_package_json = repo_root.join("package.json").exists();
    let has_go_mod = repo_root.join("go.mod").exists();
    let has_pyproject = repo_root.join("pyproject.toml").exists();

    let mut ecosystems = Vec::new();
    if has_cargo {
        ecosystems.push("Rust");
    }
    if has_package_json {
        ecosystems.push("Node.js");
    }
    if has_go_mod {
        ecosystems.push("Go");
    }
    if has_pyproject {
        ecosystems.push("Python");
    }

    let ecosystem_line = if ecosystems.is_empty() {
        "No recognized language ecosystem detected.".to_string()
    } else {
        format!("Detected: {}", ecosystems.join(", "))
    };

    format!(
        r#"> *Facilitation guide generated by `wai way ci` — instructs an LLM to lead an interactive discussion with the user.*

# CI Setup Guide

**TL;DR**: Walk through 10 topics to build a CI pipeline. For each: explain
trade-offs, ask the user's preference, generate config. By the end, the user
will have a working CI configuration tailored to their repository.
Current state: {current_state}. Ecosystems: {ecosystem_line}.

You are helping the user set up continuous integration for this repository.
Walk through the topics below **one at a time**, asking the user about their
preferences and constraints before generating any configuration. Do not dump
an entire CI config upfront — build it up topic by topic.

## Current State

- **CI status**: {current_state}
- **Ecosystems**: {ecosystem_line}

## Discussion Topics

Work through these in order. For each one, explain the trade-offs briefly,
ask what the user wants, then move on.

### 1. CI Platform
If no CI is configured yet, ask which platform they want (GitHub Actions,
GitLab CI, CircleCI, etc.). If one is already detected, confirm they want
to keep it or migrate.

### 2. Workflow Separation
Should checks be split into separate workflows/jobs or combined?
- **Separate workflows**: lint, test, build each in their own file — easier
  to re-run individually, clearer failure signals, can have different triggers.
- **Single workflow, multiple jobs**: one file, jobs run in parallel — simpler
  to manage, shared trigger config.
- **→ Ask:** Do they want lint/format checks separate from tests? Should builds
  (release artifacts) be a separate workflow?

### 3. Triggers
Which events should start CI?
- Pull requests (which branches?)
- Pushes to main/develop
- Tags (for releases)
- Manual dispatch
- Scheduled (nightly builds, dependency checks)
- Path filters — skip CI when only docs change?

### 4. Caching
Caching build dependencies dramatically speeds up CI. Discuss:
- **What to cache**: dependency downloads, build artifacts, tool binaries
- **Cache keys**: use lockfile hashes (Cargo.lock, package-lock.json, go.sum)
- **Cache scope**: per-branch with main fallback
- Platform-specific: `actions/cache`, `sccache` for Rust, `node_modules`
  caching for JS, Go module cache

### 5. Matrix Builds
Should CI test across multiple configurations?
- OS matrix (ubuntu, macos, windows)
- Language version matrix (rustc stable/nightly, node 18/20/22, python 3.11/3.12)
- Feature flags or build variants
- **→ Ask:** What's the minimum they need vs what's nice to have?
  More matrix entries = slower + more expensive.

### 6. Linting & Formatting
If the user mentions they already configured hooks or linting locally,
skip the overlap and focus on what CI adds (enforcing checks that
developers could skip locally).
- Code formatting (rustfmt, prettier, black, gofmt)
- Linters (clippy, eslint, ruff, golangci-lint)
- Type checking (tsc, mypy, pyright)
- Spell checking (typos)
- Prose linting (vale)
- Should these block merge or just warn?

### 7. Testing
If the user mentions they already configured coverage, skip the overlap
and focus on CI-specific concerns like parallelism, services, and merge blocking.
- Unit tests vs integration tests (separate jobs?)
- Test coverage — should CI enforce a threshold?
- Flaky test handling — retries?
- Test parallelism
- Do any tests need services (databases, Redis, etc.)?

### 8. Security & Secrets
- Dependency auditing (cargo audit, npm audit, pip-audit)
- Secret scanning
- SAST / code scanning (CodeQL, semgrep)
- How are secrets managed? (GitHub secrets, Vault, etc.)
- Should PRs from forks have restricted access?

### 9. Artifacts & Deployment
- Should CI produce build artifacts?
- Release automation (on tag push?)
- Container image builds
- Deploy previews for PRs
- Where do releases go? (GitHub Releases, npm, crates.io, PyPI, Docker Hub)

### 10. Performance & Cost
- Timeouts — what's a reasonable max for the whole pipeline?
- `fail-fast` — cancel other matrix jobs on first failure?
- Concurrency — cancel in-progress runs when a new push arrives?
- Self-hosted runners vs hosted — any cost constraints?

## Guidelines

- If the user says they already have something or want to skip a topic, move on.
- If the user is unsure or says "just pick for me", suggest sensible defaults
  based on the detected ecosystem and explain why.
- Generate config incrementally as decisions are made.
- Use the detected ecosystem to suggest sensible defaults.
- Prefer simple, maintainable configs over clever ones.
- Pin action versions to SHA for security (e.g. `actions/checkout@<sha>`)
  — `@v4` is common but vulnerable to tag mutation.
- Add comments in generated YAML explaining non-obvious choices.
"#
    )
}

fn guide_hooks(repo_root: &Path) -> String {
    let prek = repo_root.join("prek.toml").exists();
    let lefthook =
        repo_root.join("lefthook.yml").exists() || repo_root.join("lefthook.yaml").exists();
    let husky = repo_root.join(".husky").is_dir();
    let precommit = repo_root.join(".pre-commit-config.yaml").exists();

    let current_state = if prek {
        "prek detected (prek.toml)"
    } else if lefthook {
        "lefthook detected (lefthook.yml)"
    } else if husky {
        "husky detected (.husky/)"
    } else if precommit {
        "pre-commit framework detected (.pre-commit-config.yaml)"
    } else {
        "No git hook manager detected"
    };

    let has_typos_config = repo_root.join("typos.toml").exists()
        || repo_root.join("_typos.toml").exists()
        || repo_root.join(".typos.toml").exists();
    let has_vale_config =
        repo_root.join(".vale.ini").exists() || repo_root.join("vale.ini").exists();

    let has_cargo = repo_root.join("Cargo.toml").exists();
    let has_package_json = repo_root.join("package.json").exists();
    let has_go_mod = repo_root.join("go.mod").exists();
    let has_pyproject = repo_root.join("pyproject.toml").exists();

    let mut ecosystems = Vec::new();
    if has_cargo {
        ecosystems.push("Rust");
    }
    if has_package_json {
        ecosystems.push("Node.js");
    }
    if has_go_mod {
        ecosystems.push("Go");
    }
    if has_pyproject {
        ecosystems.push("Python");
    }

    let ecosystem_line = if ecosystems.is_empty() {
        "No recognized language ecosystem detected.".to_string()
    } else {
        ecosystems.join(", ")
    };

    format!(
        r#"> *Facilitation guide generated by `wai way hooks` — instructs an LLM to lead an interactive discussion with the user.*

# Pre-commit Hooks Guide

**TL;DR**: Walk through 8 topics to set up pre-commit quality gates. For each:
explain trade-offs, ask the user's preference, generate config. By the end,
the user will have a working hook configuration with formatting, linting,
and quality checks. Current state: {current_state}.

You are helping the user set up pre-commit quality gates for this repository.
Walk through the topics below **one at a time**, asking the user about their
preferences before generating configuration.

## Current State

- **Hook manager**: {current_state}
- **Ecosystems**: {ecosystem_line}
- **Typos config**: {typos}
- **Vale config**: {vale}

## Discussion Topics

### 1. Hook Manager
Which tool should manage git hooks?
- **prek** — simple TOML config, runs commands directly, no ecosystem lock-in.
  Good for polyglot repos. https://github.com/chshersh/prek
- **lefthook** — YAML config, parallel execution, skip conditions, mature.
  https://github.com/evilmartians/lefthook
- **husky** — JS ecosystem standard, npm-based. Best if repo is Node-only.
- **pre-commit (framework)** — Python-based, huge hook library, language-agnostic.
  Heavier setup but great plugin ecosystem.
- **→ Ask:** Do they have a preference? Any constraints (e.g. can't install Python)?

### 2. Hook Stages
Which git hooks should run checks?
- **pre-commit** — most common, runs before each commit. Fast checks only.
- **pre-push** — runs before push. Good for slower checks (full test suite).
- **commit-msg** — validate commit message format (conventional commits?).
- **→ Ask:** Do they want fast feedback on commit, thorough checks on push, or both?

### 3. Code Formatting
Formatting should be the first check — it's fast and unambiguous.
- Rust: `cargo fmt --check`
- JS/TS: `prettier --check .` or `biome check`
- Python: `ruff format --check` or `black --check`
- Go: `gofmt -l .` or `goimports`
- **→ Ask:** Which formatter do they use? Should the hook auto-fix or just check?

### 4. Linting
Catch bugs and enforce code style beyond formatting.
- Rust: `cargo clippy -- -D warnings`
- JS/TS: `eslint` or `biome lint`
- Python: `ruff check` or `flake8`
- Go: `golangci-lint run`
- **→ Ask:** Which linters? How strict — warnings or errors? Should linting
  block the commit or just warn?

### 5. Spell Checking (typos)
Catch typos in source code, comments, docs, and filenames.
- Tool: `typos` — fast, low false-positive rate, works on any language.
  https://github.com/crate-ci/typos
- Config file: `_typos.toml` (underscore prefix = local overrides)
- **→ Ask:** Do they want spell checking? Any domain words to add to the
  allow list? Should it cover all files or specific extensions?

### 6. Prose Linting (vale)
Enforce writing style in documentation and markdown.
- Tool: `vale` — configurable, supports style guides (Microsoft, Google,
  write-good). https://vale.sh
- **→ Ask:** Do they have a writing style guide? Which file types should
  it cover (*.md, *.txt, *.rst)? How strict — suggestion vs error?

### 7. Other Checks
Depending on the project, consider:
- **Security**: `cargo audit`, `npm audit`, `pip-audit` — check for
  known vulnerabilities in dependencies.
- **Secrets**: `gitleaks` or `detect-secrets` — prevent committing
  API keys, tokens, passwords.
- **File hygiene**: trailing whitespace, EOF newline, large file detection.
- **Commit message**: conventional commits format? (`commitlint`)
- **→ Ask:** Any of these relevant? Which ones matter most?

### 8. Performance
Pre-commit hooks must be fast or developers will skip them.
- Run checks only on staged files when possible (not the whole repo).
- Parallel execution — does the hook manager support it?
- Move slow checks (full test suite, coverage) to pre-push or CI only.
- **→ Ask:** What's an acceptable wait time? (target: under 5-10 seconds
  with warm caches)

## Guidelines

- If the user says they already have something or want to skip a topic, move on.
- If the user is unsure or says "just pick for me", suggest sensible defaults
  based on the detected ecosystem and explain why.
- Generate config for the chosen hook manager incrementally.
- Suggest the lightest effective checks — don't overload pre-commit.
- If a formatter can auto-fix, suggest running it in fix mode and
  re-staging, rather than just failing.
- Include install instructions (e.g. `prek install`, `lefthook install`).
"#,
        typos = if has_typos_config {
            "detected"
        } else {
            "not configured"
        },
        vale = if has_vale_config {
            "detected"
        } else {
            "not configured"
        },
    )
}

fn guide_coverage(repo_root: &Path) -> String {
    let has_cargo = repo_root.join("Cargo.toml").exists();
    let has_package_json = repo_root.join("package.json").exists();
    let has_pyproject = repo_root.join("pyproject.toml").exists();
    let has_go_mod = repo_root.join("go.mod").exists();

    let has_tarpaulin = repo_root.join("tarpaulin.toml").exists()
        || std::fs::read_to_string(repo_root.join("Cargo.toml"))
            .ok()
            .is_some_and(|c| c.contains("[package.metadata.tarpaulin]"));
    let has_codecov =
        repo_root.join(".codecov.yml").exists() || repo_root.join("codecov.yml").exists();
    let has_coveralls = repo_root.join(".coveralls.yml").exists();

    let mut detected = Vec::new();
    if has_tarpaulin {
        detected.push("tarpaulin");
    }
    if has_codecov {
        detected.push("codecov");
    }
    if has_coveralls {
        detected.push("coveralls");
    }

    let current_state = if detected.is_empty() {
        "No coverage tools detected".to_string()
    } else {
        format!("Detected: {}", detected.join(", "))
    };

    let mut ecosystems = Vec::new();
    if has_cargo {
        ecosystems.push("Rust");
    }
    if has_package_json {
        ecosystems.push("Node.js");
    }
    if has_pyproject {
        ecosystems.push("Python");
    }
    if has_go_mod {
        ecosystems.push("Go");
    }

    let ecosystem_line = if ecosystems.is_empty() {
        "No recognized language ecosystem detected.".to_string()
    } else {
        ecosystems.join(", ")
    };

    format!(
        r#"> *Facilitation guide generated by `wai way coverage` — instructs an LLM to lead an interactive discussion with the user.*

# Test Coverage Guide

**TL;DR**: Walk through 6 topics to set up test coverage. For each: explain
trade-offs, ask the user's preference, generate config. By the end, the user
will have coverage tooling with thresholds and reporting configured.
Current state: {current_state}. Ecosystems: {ecosystem_line}.

You are helping the user set up test coverage for this repository.
Walk through the topics below **one at a time**, asking about their
goals and constraints before generating configuration.

## Current State

- **Coverage tools**: {current_state}
- **Ecosystems**: {ecosystem_line}

## Discussion Topics

### 1. Coverage Tool
Which tool should measure coverage?
- **Rust**: `cargo-tarpaulin` (easy setup, good defaults) or `cargo-llvm-cov`
  (more accurate, uses LLVM instrumentation). tarpaulin is simpler;
  llvm-cov is more precise, requires `llvm-tools-preview` component.
- **Python**: `coverage.py` / `pytest-cov` — the standard. Works with
  pytest out of the box.
- **JS/TS**: `vitest` has built-in coverage, or `c8`/`nyc` (istanbul).
- **Go**: `go test -cover` built-in. Use `go tool cover` for reports.
- **→ Ask:** Do they already have a preference? Any constraints?

### 2. Coverage Threshold
Should the project enforce a minimum coverage percentage?
- **Why**: prevents regressions — new code without tests gets caught.
- **Starting point**: don't aim for 100%. Pick a realistic baseline
  (measure current coverage first) and ratchet up over time.
- **How to enforce**: `fail-under` (tarpaulin/coverage.py), `thresholds`
  (vitest), CI check.
- **→ Ask:** Do they want a hard threshold? What feels realistic — 60%? 80%?
  Should it block merges or just report?

### 3. What to Measure
Not all code is equally worth covering.
- **Include**: business logic, data transformations, error handling paths.
- **Exclude**: generated code, test utilities, CLI scaffolding, migration
  files, vendored dependencies.
- **→ Ask:** Are there directories or patterns that should be excluded from
  coverage? (e.g. `tests/`, `benches/`, `migrations/`, `vendor/`)

### 4. Coverage Reporting
Where should coverage reports go?
- **Local**: HTML reports for developer inspection (`tarpaulin --out Html`,
  `coverage html`, `vitest --coverage`).
- **CI**: upload to a service for PR-level feedback.
  - **Codecov** — free for open source, PR comments with diff coverage.
  - **Coveralls** — similar, good GitHub integration.
  - **CI-only** — just print the number, fail if below threshold.
- **→ Ask:** Do they want a reporting service, or is a CI threshold enough?

### 5. Diff Coverage vs Total Coverage
- **Total coverage**: overall project percentage. Can be gamed by deleting
  untested code. Useful as a baseline.
- **Diff coverage**: coverage of *changed lines only*. Ensures new code
  is tested without penalizing legacy gaps.
- Services like Codecov support both.
- **→ Ask:** Do they care more about overall project coverage or ensuring
  new PRs are well-tested?

### 6. Integration with CI
- Should coverage run on every PR, or only on pushes to main?
- How long does the test suite take? Coverage adds overhead (10-30%).
- Should coverage failure block merge?
- **→ Ask:** Where in the pipeline should this run?

## Guidelines

- If the user says they already have something or want to skip a topic, move on.
- If the user is unsure or says "just pick for me", suggest sensible defaults
  based on the detected ecosystem and explain why.
- Start by measuring current coverage before setting a threshold.
- Suggest `fail-under` / `thresholds` in config, not just CI scripts,
  so it works locally too.
- Recommend excluding test files and generated code from coverage.
- If they have no tests yet, suggest starting with coverage tooling
  anyway — it creates the habit.
"#
    )
}

fn guide_docs(repo_root: &Path) -> String {
    let has_readme = repo_root.join("README.md").exists();
    let has_license = repo_root.join("LICENSE").exists() || repo_root.join("LICENSE.md").exists();
    let has_contributing = repo_root.join("CONTRIBUTING.md").exists();
    let has_gitignore = repo_root.join(".gitignore").exists();
    let has_docs_dir = repo_root.join("docs").is_dir();
    let doc_tool = detect_doc_tool(repo_root);

    let mut present = Vec::new();
    let mut missing = Vec::new();
    for (name, exists) in [
        ("README.md", has_readme),
        ("LICENSE", has_license),
        ("CONTRIBUTING.md", has_contributing),
        (".gitignore", has_gitignore),
        ("docs/", has_docs_dir),
    ] {
        if exists {
            present.push(name);
        } else {
            missing.push(name);
        }
    }

    let doc_tool_line = doc_tool.as_deref().unwrap_or("none detected");

    format!(
        r#"> *Facilitation guide generated by `wai way docs` — instructs an LLM to lead an interactive discussion with the user.*

# Documentation Guide

**TL;DR**: Walk through 7 topics to set up project documentation. For each:
explain trade-offs, ask the user's preference, generate files. By the end,
the user will have README, LICENSE, and doc tooling configured.
Present: {present}. Missing: {missing}.

You are helping the user set up project documentation for this repository.
Walk through the topics below **one at a time**, asking about their
audience and goals before generating files.

## Current State

- **Present**: {present}
- **Missing**: {missing}
- **Doc tool**: {doc_tool_line}

## Discussion Topics

### 1. README
The front door of the project.
- **Audience**: is this for end-users, developers, or both?
- **Sections**: project name + one-liner, badges, install, quick start,
  usage examples, configuration, contributing link, license.
- **Tone**: formal/corporate or casual/community?
- **→ Ask:** What's the project about in one sentence? Who reads the README?

### 2. License
- **Open source?** If yes, which license? (MIT, Apache 2.0, GPL, etc.)
- **Not open source?** A proprietary notice or "All rights reserved".
- **Dual licensing?** Some projects use MIT + Apache 2.0.
- **→ Ask:** Is this open source? Do they have a preferred license?

### 3. Contributing Guide
Who can contribute, and how?
- Development setup instructions
- Branch naming and PR conventions
- Code style and testing requirements
- Code of conduct reference
- **→ Ask:** Do they accept outside contributions? What should contributors
  know before submitting a PR?

### 4. .gitignore
What should stay out of version control?
- Language-specific build artifacts
- IDE/editor files (.idea/, .vscode/, *.swp)
- OS files (.DS_Store, Thumbs.db)
- Secrets and env files (.env, *.key)
- **→ Ask:** Any custom paths or patterns to ignore? Using a template
  from gitignore.io is a good starting point.

### 5. API / Reference Documentation
Generated documentation from source code.
- **Rust**: `cargo doc` — built-in, doc comments become HTML.
- **Python**: Sphinx, MkDocs + mkdocstrings, pdoc.
- **JS/TS**: TypeDoc, JSDoc.
- **Go**: godoc — built-in.
- **→ Ask:** Do they need generated API docs? Should they be published
  (GitHub Pages, ReadTheDocs)?

### 6. Docs Directory
A `docs/` folder for long-form documentation beyond the README.
- Architecture docs, design decisions, tutorials, guides.
- Should it use a static site generator (MkDocs, Docusaurus, mdBook)?
- **→ Ask:** Is there documentation beyond the README that needs a home?
  Do they want a docs site?

### 7. Changelog
How are changes communicated to users?
- **CHANGELOG.md** — manually curated or auto-generated.
- **Conventional commits** + tools like `git-cliff`, `release-please`,
  `standard-version` to generate changelogs.
- **GitHub Releases** — release notes per tag.
- **→ Ask:** Do they want a changelog? Manual or automated?

## Guidelines

- If the user says they already have something or want to skip a topic, move on.
- If the user is unsure or says "just pick for me", suggest sensible defaults
  based on the detected ecosystem and explain why.
- Generate files one at a time as decisions are made.
- README should be useful on day one — don't over-template it.
- Prefer language-standard doc tools over third-party when available.
- Suggest `just docs` recipe if a justfile exists.
"#,
        present = if present.is_empty() {
            "none".to_string()
        } else {
            present.join(", ")
        },
        missing = if missing.is_empty() {
            "none".to_string()
        } else {
            missing.join(", ")
        },
    )
}

fn guide_devxp(repo_root: &Path) -> String {
    let has_editorconfig = repo_root.join(".editorconfig").exists();
    let has_devcontainer =
        repo_root.join(".devcontainer").is_dir() || repo_root.join(".devcontainer.json").exists();
    let has_justfile = repo_root.join("justfile").exists();
    let has_makefile = repo_root.join("Makefile").exists();

    let task_runner = if has_justfile {
        "justfile detected"
    } else if has_makefile {
        "Makefile detected"
    } else {
        "No task runner detected"
    };

    format!(
        r#"> *Facilitation guide generated by `wai way devxp` — instructs an LLM to lead an interactive discussion with the user.*

# Developer Experience Guide

**TL;DR**: Walk through 6 topics to improve developer experience. For each:
explain trade-offs, ask the user's preference, generate config. By the end,
the user will have editor config, task runner, and environment setup.
EditorConfig: {editorconfig}. Task runner: {task_runner}. Dev container: {devcontainer}.

You are helping the user improve the developer experience in this repository.
Walk through the topics below **one at a time**, asking about their team's
tools and workflows before generating configuration.

## Current State

- **EditorConfig**: {editorconfig}
- **Dev container**: {devcontainer}
- **Task runner**: {task_runner}

## Discussion Topics

### 1. EditorConfig
Consistent formatting across every editor, without extra plugins.
- Defines indent style (tabs vs spaces), indent size, line endings,
  trailing whitespace, final newline.
- Supported by most editors (VS Code via extension, JetBrains built-in, Vim via plugin).
- **→ Ask:** Tabs or spaces? Indent size (2, 4)? Any per-language overrides
  (e.g. 2 spaces for YAML, 4 for Rust)?

### 2. Task Runner
A single entry point for common operations — build, test, lint, format, docs.
- **justfile** — simple, cross-platform, no dependencies beyond `just`.
  Supports arguments, dependencies between recipes, dotenv loading.
  https://just.systems
- **Makefile** — ubiquitous but quirky syntax. Works everywhere.
- **npm scripts** — good if the project is Node.js-only.
- **Taskfile (go-task)** — YAML-based, cross-platform.
- **→ Ask:** Which tool? What are the most common tasks they run?
  Suggested recipes: `build`, `test`, `lint`, `fmt`, `check`, `docs`,
  `dev`/`serve`, `clean`, `install`.

### 3. Dev Containers
Reproducible development environments that "just work".
- **devcontainer.json** — VS Code / GitHub Codespaces / DevPod standard.
- Defines: base image, extensions, port forwarding, post-create commands,
  environment variables, features (e.g. Rust, Node, Docker-in-Docker).
- **→ Ask:** Do team members use different OSes or machine setups? Do they
  use Codespaces or similar? Which tools/runtimes must be in the
  container? Any services needed (database, Redis)?

### 4. Environment Variables
How does the project handle configuration?
- `.env` files (with `.env.example` committed, `.env` gitignored)
- `direnv` with `.envrc` — auto-loads env vars when entering the directory
- **→ Ask:** Do they use env vars for configuration? Should there be a
  template `.env.example`? Do they use `direnv`?

### 5. IDE Configuration
Shared settings that help the whole team.
- `.vscode/settings.json` — format on save, recommended extensions,
  ruler position, file associations.
- `.vscode/extensions.json` — recommended extensions list.
- `.idea/` — JetBrains shared run configs, code style.
- **→ Ask:** Does the team use a common editor? Should shared settings
  be committed or left to individual preference?

### 6. Local Development Server
For projects with a development server or watch mode.
- `just dev` or `just serve` recipe
- Hot reloading / watch mode configuration
- Port conventions
- **→ Ask:** Is there a dev server? What command starts it? Should it
  be standardized in the task runner?

## Guidelines

- If the user says they already have something or want to skip a topic, move on.
- If the user is unsure or says "just pick for me", suggest sensible defaults
  based on the detected ecosystem and explain why.
- Start with editorconfig and task runner — biggest bang for effort.
- Dev containers are high-effort, high-reward — suggest only if there's
  onboarding pain or environment inconsistency.
- Don't over-configure IDE settings — keep them minimal and optional.
- Task runner recipes should mirror CI steps (if `just lint` passes
  locally, CI lint should also pass).
"#,
        editorconfig = if has_editorconfig {
            "detected"
        } else {
            "not configured"
        },
        devcontainer = if has_devcontainer {
            "detected"
        } else {
            "not configured"
        },
    )
}

fn guide_ai(repo_root: &Path) -> String {
    let has_claude_md = repo_root.join("CLAUDE.md").exists();
    let has_agents_md = repo_root.join("AGENTS.md").exists();
    let has_llm_txt = repo_root.join("llm.txt").exists();
    let has_skills = agent_config_dir(repo_root).join(SKILLS_DIR).is_dir();

    format!(
        r#"> *Facilitation guide generated by `wai way ai` — instructs an LLM to lead an interactive discussion with the user.*

# AI-Assisted Development Guide

**TL;DR**: Walk through 5 topics to configure AI-assisted development. For each:
explain trade-offs, ask the user's preference, generate files. By the end,
the user will have CLAUDE.md, llm.txt, and/or agent skills configured.
CLAUDE.md: {claude_md}. llm.txt: {llm_txt}. Skills: {skills}.

You are helping the user configure their repository for effective AI-assisted
development. Walk through the topics below **one at a time**, asking about
their AI workflow before generating files.

## Current State

- **CLAUDE.md**: {claude_md}
- **AGENTS.md**: {agents_md}
- **llm.txt**: {llm_txt}
- **Agent skills**: {skills}

## Discussion Topics

### 1. AI Instruction File
A persistent file that gives AI assistants project context and rules.
- **CLAUDE.md** — read by Claude Code automatically. Project conventions,
  architecture notes, "do this / don't do that" rules.
- **AGENTS.md** — similar, used by some other AI tools.
- **What to include**: coding style, test conventions, architecture overview,
  known gotchas, forbidden patterns, preferred libraries.
- **What NOT to include**: things derivable from code, ephemeral state.
- **→ Ask:** Which AI tools does the team use? What mistakes does the AI
  keep making that instructions could prevent? What context is hardest
  for the AI to figure out on its own?

### 2. LLM Context File (llm.txt)
Machine-readable project summary for LLMs. Lives at `llm.txt` in the root.
- Purpose: gives any LLM a quick orientation — what the project does,
  key files, entry points, architecture.
- Standard: https://llmstxt.org
- Different from CLAUDE.md — llm.txt is a neutral summary, not rules.
- **→ Ask:** Do they want a general-purpose LLM context file? What should
  it cover — public API surface, architecture, or both?

### 3. Agent Skills
Reusable workflow definitions that enhance AI agent behavior.
- Stored in `.wai/resources/agent-config/skills/` (or equivalent).
- Examples: structured commit workflow, iterative review (Rule of 5),
  code review checklist, PR creation workflow.
- Each skill is a markdown file with frontmatter (name, trigger, aliases).
- **→ Ask:** What repetitive workflows could be codified as skills? Do they
  want the recommended defaults (rule-of-5, commit)?
  Run `wai way --fix skills` to scaffold them.

### 4. Subdirectory Instructions
For monorepos or projects with distinct areas, per-directory instruction
files can scope AI behavior.
- `frontend/CLAUDE.md` — React-specific conventions
- `backend/CLAUDE.md` — API-specific patterns
- These supplement the root file, not replace it.
- **→ Ask:** Does the project have distinct areas that need different AI
  guidance? Is it a monorepo?

### 5. Context Management
Help the AI work efficiently with the codebase.
- Keep CLAUDE.md concise — it's loaded into every conversation.
- Move detailed reference material to linked files the AI can read
  on demand (architecture docs, API specs).
- Use `wai reflect` to periodically synthesize project patterns.
- **→ Ask:** Is the CLAUDE.md getting too long? Are there docs the AI
  should reference but doesn't need in every conversation?

## Guidelines

- If the user says they already have something or want to skip a topic, move on.
- If the user is unsure or says "just pick for me", suggest sensible defaults
  and explain why.
- Start with CLAUDE.md — it has the highest impact for Claude Code users.
- Write instructions as rules, not essays. "Use snake_case for file names"
  beats a paragraph explaining why.
- Review and update AI instructions periodically — stale rules are
  worse than no rules.
- Test instructions by starting a fresh conversation and seeing
  if the AI behaves as expected.
"#,
        claude_md = if has_claude_md {
            "detected"
        } else {
            "not present"
        },
        agents_md = if has_agents_md {
            "detected"
        } else {
            "not present"
        },
        llm_txt = if has_llm_txt {
            "detected"
        } else {
            "not present"
        },
        skills = if has_skills {
            "configured"
        } else {
            "not configured"
        },
    )
}

fn guide_issues(repo_root: &Path) -> String {
    let has_beads = repo_root.join(".beads").is_dir();
    let has_github = repo_root.join(".github").is_dir();

    let current_state = if has_beads {
        "beads detected (.beads/) — local-first issue tracking"
    } else if has_github {
        "GitHub repository detected — GitHub Issues available"
    } else {
        "No issue tracking detected"
    };

    format!(
        r#"> *Facilitation guide generated by `wai way issues` — instructs an LLM to lead an interactive discussion with the user.*

# Issue Tracking Guide

**TL;DR**: Walk through 6 topics to set up issue tracking. For each: explain
trade-offs, ask the user's preference, recommend a setup. By the end, the user
will have a tracking tool chosen and configured with structure, labels, and workflow.
Current state: {current_state}.

You are helping the user set up issue tracking for this repository.
Walk through the topics below **one at a time**, asking about their
team size, workflow, and preferences before recommending a setup.

## Current State

- **Tracking**: {current_state}

## Discussion Topics

### 1. Tracking Tool
Where should issues live?
- **GitHub Issues** — built into GitHub, free, good for open source.
  Labels, milestones, projects. Limited dependency tracking.
- **Linear** — fast, opinionated, great for small teams. Cycles,
  projects, triage workflows. Not free.
- **Jira** — enterprise standard, powerful but complex. Custom workflows,
  deep integrations. Can be overkill for small projects.
- **beads** — local-first, lives in the repo (.beads/). Good for
  solo/AI-assisted work. Dependencies, priorities, no external service.
- **Plain markdown** — `TODO.md` or `docs/tasks.md`. Simple but
  no status tracking or assignment.
- **→ Ask:** Is this solo work, a small team, or a large org? Do they need
  external visibility (stakeholders, open source community)?

### 2. Issue Structure
What goes in an issue?
- **Title** — clear, actionable (verb + noun: "Add caching to API")
- **Description** — why this needs doing, acceptance criteria, context
- **Type** — bug, feature, task, chore, spike/research
- **Priority** — how urgent? (P0-P4, or high/medium/low)
- **Size/effort** — t-shirt sizes, story points, or skip it?
- **→ Ask:** How detailed should issues be? Do they want templates?

### 3. Labels & Categories
How to organize issues beyond type and priority.
- **Area labels**: `frontend`, `backend`, `infra`, `docs`
- **Status labels**: `needs-triage`, `ready`, `blocked`, `wontfix`
- **Effort labels**: `good-first-issue`, `quick-win`, `epic`
- **→ Ask:** What categories make sense for this project? Keep it minimal —
  too many labels means none get used.

### 4. Workflow & States
How does an issue move from creation to completion?
- Basic: `open` → `in_progress` → `closed`
- Kanban: `backlog` → `ready` → `in_progress` → `review` → `done`
- **→ Ask:** How many states do they need? Who moves issues between states?
  Should there be a triage step?

### 5. Dependencies & Blocking
Do issues depend on each other?
- Some tools support explicit dependency links (beads, Linear, Jira).
- GitHub Issues: use "blocked by #123" in description (manual).
- **→ Ask:** Do they need dependency tracking? Or is it simple enough
  that ordering the backlog suffices?

### 6. Templates & Automation
- **Issue templates**: pre-filled forms for bug reports, feature requests.
  (GitHub supports `.github/ISSUE_TEMPLATE/`)
- **Auto-labeling**: label PRs/issues based on file paths changed.
- **Stale bot**: auto-close old issues after inactivity.
- **→ Ask:** Do they want issue templates? Any automation?

## Guidelines

- If the user says they already have something or want to skip a topic, move on.
- If the user is unsure or says "just pick for me", suggest sensible defaults:
  solo/AI → beads, small team → Linear or GitHub Issues, open source →
  GitHub Issues, enterprise → Jira.
- Match tool complexity to team size.
- Start with minimal labels — add more only when you feel the pain.
- Write issues for your future self — include enough context to
  pick them up cold.
- If using beads, remind them to `bd dolt push` to sync.
"#
    )
}

fn guide_specs(repo_root: &Path) -> String {
    let has_openspec = repo_root.join("openspec").is_dir();
    let has_docs_decisions =
        repo_root.join("docs/decisions").is_dir() || repo_root.join("docs/adr").is_dir();
    let has_rfcs = repo_root.join("rfcs").is_dir() || repo_root.join("docs/rfcs").is_dir();

    let current_state = if has_openspec {
        "openspec detected (openspec/)"
    } else if has_rfcs {
        "RFCs directory detected"
    } else if has_docs_decisions {
        "ADR directory detected"
    } else {
        "No change proposal system detected"
    };

    format!(
        r#"> *Facilitation guide generated by `wai way specs` — instructs an LLM to lead an interactive discussion with the user.*

# Change Proposals & Specs Guide

**TL;DR**: Walk through 6 topics to set up a design decision tracking system.
For each: explain trade-offs, ask the user's preference, set up the structure.
By the end, the user will have a proposal format, review process, and lifecycle
configured. Current state: {current_state}.

You are helping the user set up a system for managing design decisions and
change proposals. Walk through the topics below **one at a time**, asking
about their team's decision-making process.

## Current State

- **Proposal system**: {current_state}

## Discussion Topics

### 1. Why Track Decisions
If the user isn't sure they need this, discuss why before choosing a tool:
- **Architecture Drift** — without records, the codebase drifts from
  its intended design. New contributors reinvent or contradict past decisions.
- **Onboarding** — new team members can read *why* things are the way
  they are, not just *what* the code does.
- **Review** — proposals create a review checkpoint before big changes.
- **→ Ask:** What's driving this? Past mistakes, growing team, compliance,
  or just good hygiene?

### 2. Proposal Format
Which format fits their workflow?
- **ADRs (Architecture Decision Records)** — lightweight markdown files.
  Status, Context, Decision, Consequences. One file per decision.
  Standard: https://adr.github.io
- **RFCs** — more structured proposals with problem statement, alternatives
  considered, detailed design. Good for larger teams.
- **openspec** — specification-driven proposals with capabilities, changes,
  tasks, and strict validation. Good for AI-assisted development where
  specs guide implementation. https://github.com/Fission-AI/OpenSpec
- **→ Ask:** How formal do they need? ADRs for lightweight, RFCs for mid-weight,
  openspec for spec-driven development.

### 3. When to Write a Proposal
Not every change needs a proposal. Define the threshold:
- **Always**: breaking changes, new capabilities, architecture shifts,
  security changes, major dependency additions.
- **Never**: bug fixes, small features, refactors within existing patterns.
- **Gray area**: performance work, large migrations, workflow changes.
- **→ Ask:** What kinds of changes should require a proposal? Who decides
  when one is needed?

### 4. Review Process
How are proposals reviewed and approved?
- **PR-based** — proposal is a PR, reviewed like code. Merged = approved.
- **Meeting-based** — discuss in a design review meeting. Document outcome.
- **Lazy consensus** — post it, wait N days, ship if no objections.
- **Owner approval** — designated area owners approve proposals.
- **→ Ask:** How does the team make decisions today? What level of formality?

### 5. Lifecycle & Archival
What happens after a proposal is implemented?
- **Status field**: proposed → approved → implemented → archived/superseded.
- Keep completed proposals as historical record.
- When a decision is reversed, write a new proposal that supersedes
  the old one (don't edit history).
- **→ Ask:** Do they want lifecycle tracking? Should old proposals be
  archived or kept in the main directory?

### 6. Integration with Code
How do proposals connect to implementation?
- Reference proposal ID in commit messages and PRs.
- Link tasks/issues to proposals (e.g. beads issues referencing
  openspec changes).
- Some teams add a "Decision" comment in code pointing to the ADR.
- **→ Ask:** How tightly should proposals link to code? Should there be
  task tracking integration?

## Guidelines

- If the user says they already have something or want to skip a topic, move on.
- If the user is unsure or says "just pick for me", suggest sensible defaults:
  ADRs for most teams, openspec for spec-driven AI development.
- Number proposals sequentially for easy reference.
- Store proposals in the repo (not a wiki) so they're versioned
  alongside the code they describe.
- The first proposal should document the decision to use proposals.
"#
    )
}

fn guide_gh() -> String {
    let gh_installed = std::process::Command::new("gh")
        .arg("--version")
        .output()
        .ok()
        .is_some_and(|o| o.status.success());

    let gh_authed = gh_installed
        && std::process::Command::new("gh")
            .args(["auth", "status"])
            .output()
            .ok()
            .is_some_and(|o| o.status.success());

    let current_state = if gh_authed {
        "gh installed and authenticated"
    } else if gh_installed {
        "gh installed but not authenticated"
    } else {
        "gh not installed"
    };

    format!(
        r#"> *Facilitation guide generated by `wai way gh` — instructs an LLM to lead an interactive discussion with the user.*

# GitHub CLI Integration Guide

**TL;DR**: Walk through 6 topics to set up the GitHub CLI. For each: explain
what it enables, ask if the user wants it, configure it. By the end, the user
will have gh installed, authenticated, and customized for their workflow.
Current state: {current_state}.

You are helping the user set up and configure the GitHub CLI (`gh`) for
this repository. Walk through the topics below **one at a time**.

## Current State

- **gh status**: {current_state}

## Discussion Topics

### 1. Installation & Auth
- **Install**: `brew install gh`, `dnf install gh`, `scoop install gh`,
  or download from https://cli.github.com
- **Auth**: `gh auth login` — choose HTTPS or SSH, browser or token.
- **Multiple accounts**: `gh auth login --hostname` for GitHub Enterprise.
- **→ Ask:** Is gh installed? Do they use github.com or GitHub Enterprise?
  HTTPS or SSH?

### 2. Default Repository Settings
Configure defaults for this repo:
- `gh repo set-default` — set the default remote for `gh` commands.
- **→ Ask:** Is the remote already set up? Do they need to fork a repo
  or work on the origin directly?

### 3. PR Workflow
Do they want to manage pull requests from the CLI?
- `gh pr create` — create PRs from the CLI with title, body, reviewers.
- `gh pr checkout <number>` — check out a PR locally.
- `gh pr merge` — merge with squash, rebase, or merge commit.
- `gh pr view --web` — open PR in browser.
- **→ Ask:** Do they prefer squash, rebase, or merge commits? Default
  reviewers? PR template?

### 4. Issue Management
Do they want to manage issues from the CLI?
- `gh issue create` — create issues from CLI.
- `gh issue list` — list and filter issues.
- `gh issue close` — close from CLI.
- Useful for scripting: `gh issue list --json number,title,labels`
- **→ Ask:** Do they use GitHub Issues? Would CLI issue management help
  their workflow?

### 5. GitHub Actions from CLI
Do they want to interact with CI from the terminal?
- `gh run list` — see recent workflow runs.
- `gh run watch` — live-tail a running workflow.
- `gh run rerun` — retry a failed run.
- `gh workflow run` — trigger a workflow manually.
- **→ Ask:** Do they interact with CI from the terminal? Would watching
  runs or re-triggering failures be useful?

### 6. Extensions & Aliases
- **Aliases**: `gh alias set` — short commands for common operations.
  e.g. `gh alias set prc 'pr create --fill'`
- **Extensions**: community plugins. `gh extension install <repo>`.
  Popular: `gh-dash` (dashboard), `gh-copilot`, `gh-poi` (clean branches).
- **→ Ask:** Any repetitive `gh` commands that could be aliased? Interested
  in any extensions?

## Guidelines

- If the user says they already have something or want to skip a topic, move on.
- If the user is unsure or says "just pick for me", suggest sensible defaults
  and explain why.
- gh is most valuable when the project already uses GitHub for hosting.
- Suggest aliases for the team's most common operations.
- For AI-assisted workflows, `gh pr create` and `gh run watch` are
  the most impactful commands.
- Don't over-configure — gh works well with minimal setup.
"#
    )
}

fn fix_skills(repo_root: &Path) -> Result<()> {
    use cliclack::log;

    let skills_dir = agent_config_dir(repo_root).join(SKILLS_DIR);
    std::fs::create_dir_all(&skills_dir).into_diagnostic()?;

    println!();
    println!(
        "  Scaffolding recommended skills into {}:",
        skills_dir
            .strip_prefix(repo_root)
            .unwrap_or(&skills_dir)
            .display()
    );
    println!("    • rule-of-5-universal — iterative quality review workflow");
    println!("    • commit — structured, deliberate commit workflow");
    println!();

    let mut created = 0usize;

    for (skill_name, content) in [SKILL_RULE_OF_5, SKILL_COMMIT] {
        let skill_dir = skills_dir.join(skill_name);
        let skill_file = skill_dir.join("SKILL.md");
        if skill_file.exists() {
            println!("  {} {} — already present", "○".dimmed(), skill_name);
            continue;
        }
        std::fs::create_dir_all(&skill_dir).into_diagnostic()?;
        std::fs::write(&skill_file, content).into_diagnostic()?;
        log::success(format!("Created skill '{}'", skill_name)).into_diagnostic()?;
        created += 1;
    }

    if created == 0 {
        println!("\n  Recommended skills already present — nothing to do.");
    } else {
        println!(
            "\n  {} skill(s) added to .wai/resources/agent-config/skills/",
            created
        );
    }

    Ok(())
}

fn render_human(checks: &[CheckResult], summary: &Summary, verbose: u8) -> Result<()> {
    use cliclack::outro;
    use miette::IntoDiagnostic;

    println!();
    println!("  {} Repo Hygiene & Agent Workflow Conventions", "◆".cyan());
    println!(
        "  {} For wai workspace health, run 'wai doctor'",
        "·".dimmed()
    );
    println!();

    for check in checks {
        let icon = match check.status {
            Status::Pass => "✓".green().to_string(),
            Status::Info => "ℹ".cyan().to_string(),
        };
        println!("  {} {}: {}", icon, check.name.bold(), check.message);
        if verbose > 0 {
            if let Some(ref intent) = check.intent {
                println!("    {} Intent: {}", "·".dimmed(), intent.dimmed());
            }
            if let Some(ref criteria) = check.success_criteria {
                println!("    {} Success: {}", "·".dimmed(), criteria.dimmed());
            }
        }
        if let Some(ref suggestion) = check.suggestion {
            println!("    {} {}", "→".dimmed(), suggestion.dimmed());
        }
    }

    println!();
    let total_checks = summary.pass + summary.recommendations;
    let summary_line = if summary.recommendations == 0 {
        "excellent! All best practices adopted".to_string()
    } else if summary.pass == 0 {
        format!(
            "{}/{} best practices adopted — quick-start: add README.md, justfile, .gitignore",
            summary.pass, total_checks
        )
    } else {
        format!("{}/{} best practices adopted", summary.pass, total_checks)
    };

    if summary.recommendations > 0 {
        outro(summary_line.cyan().to_string()).into_diagnostic()?;
    } else {
        outro(summary_line.green().to_string()).into_diagnostic()?;
    }

    println!(
        "  {} Deep-dive into any area: {}",
        "·".dimmed(),
        format!("wai way <{}>", AVAILABLE_TOPICS.join("|")).dimmed()
    );
    println!();

    Ok(())
}

fn check_task_runner(repo_root: &Path) -> CheckResult {
    let name = "Command standardization";
    let intent = Some("Provide a single, tool-agnostic entry point for common repository tasks (build, test, deploy).".to_string());
    let success_criteria = Some(
        "A standard interface (justfile, Makefile, npm scripts) exists for common tasks."
            .to_string(),
    );

    let justfile = repo_root.join("justfile");
    let makefile = repo_root.join("Makefile");

    if justfile.exists() {
        let recipes = parse_justfile_recipes(&justfile);
        let message = if recipes.is_empty() {
            "justfile detected".to_string()
        } else {
            format!("justfile detected (recipes: {})", recipes.join(", "))
        };

        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message,
            intent,
            success_criteria,
            suggestion: None,
        }
    } else if makefile.exists() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "Makefile detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Consider migrating to justfile for better ergonomics — https://just.systems"
                    .to_string(),
            ),
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No task runner detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Add a justfile to standardize common tasks — https://just.systems".to_string(),
            ),
        }
    }
}

fn parse_justfile_recipes(justfile_path: &Path) -> Vec<String> {
    let known_recipes = [
        "install", "serve", "dev", "test", "lint", "fmt", "format", "release", "build", "run",
        "clean", "check", "watch", "docs",
    ];

    let content = match std::fs::read_to_string(justfile_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut found_recipes = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        // Recipe definitions start at the beginning of a line (no leading whitespace)
        // and end with a colon
        if !line.starts_with(' ')
            && !line.starts_with('\t')
            && trimmed.contains(':')
            && let Some(recipe_name) = trimmed.split(':').next()
        {
            let recipe_name = recipe_name.split_whitespace().next().unwrap_or("");
            if known_recipes.contains(&recipe_name)
                && !found_recipes.contains(&recipe_name.to_string())
            {
                found_recipes.push(recipe_name.to_string());
            }
        }
    }

    found_recipes
}

/// Read a named hook file contents, or `None` if absent/unreadable.
fn read_hook(repo_root: &Path, hook_name: &str) -> Option<String> {
    let hook_path = repo_root.join(".git/hooks").join(hook_name);
    if !hook_path.exists() || hook_path.is_dir() {
        return None;
    }
    std::fs::read_to_string(&hook_path).ok()
}

/// Return `true` if pre-commit **or** pre-push hook contains `needle`.
fn hook_contains(repo_root: &Path, needle: &str) -> bool {
    read_hook(repo_root, "pre-commit").is_some_and(|c| c.contains(needle))
        || read_hook(repo_root, "pre-push").is_some_and(|c| c.contains(needle))
}

/// Return `true` if the pre-commit hook file exists and is non-empty.
fn hook_exists_nonempty(repo_root: &Path) -> bool {
    read_hook(repo_root, "pre-commit").is_some_and(|c| !c.trim().is_empty())
}

/// Return the `core.hooksPath` git config value if set, else `None`.
fn git_core_hooks_path(repo_root: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args([
            "-C",
            &repo_root.to_string_lossy(),
            "config",
            "core.hooksPath",
        ])
        .output()
        .ok()?;
    if output.status.success() {
        let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !val.is_empty() { Some(val) } else { None }
    } else {
        None
    }
}

/// Identify which known tool owns the pre-commit hook, or `None`.
///
/// Checks for the same sigils that the rest of `check_git_hooks` uses:
/// `bd`, `lefthook`, `husky`, `pre-commit` (the framework).
fn hook_owner(repo_root: &Path) -> Option<&'static str> {
    let content = read_hook(repo_root, "pre-commit")?;
    // Check more-specific patterns first to avoid false matches.
    for (needle, name) in &[
        ("lefthook", "lefthook"),
        ("husky", "husky"),
        ("bd", "bd"),
        ("pre-commit", "pre-commit"),
    ] {
        if content.contains(needle) {
            return Some(name);
        }
    }
    None
}

fn check_git_hooks(repo_root: &Path) -> CheckResult {
    let name = "Pre-commit quality gates";
    let intent = Some(
        "Prevent low-quality commits by running automated checks before code is saved to history."
            .to_string(),
    );
    let success_criteria = Some(
        "Automated checks (linters, tests) run automatically before code is committed.".to_string(),
    );

    let prek_config = repo_root.join("prek.toml");
    let precommit_config = repo_root.join(".pre-commit-config.yaml");
    let lefthook_config = repo_root.join("lefthook.yml");
    let lefthook_config_yaml = repo_root.join("lefthook.yaml");
    let husky_dir = repo_root.join(".husky");

    if prek_config.exists() {
        // core.hooksPath being set means prek refuses to install — report this first.
        if let Some(hooks_path) = git_core_hooks_path(repo_root) {
            return CheckResult {
                name: name.to_string(),
                status: Status::Info,
                message: format!(
                    "prek.toml found but core.hooksPath is set ('{}') — prek cannot install",
                    hooks_path
                ),
                intent,
                success_criteria,
                suggestion: Some(
                    "Unset it first: git config --unset core.hooksPath && git config --global --unset core.hooksPath".to_string(),
                ),
            };
        }
        // Check both pre-commit and pre-push for prek signature.
        if hook_contains(repo_root, "prek") {
            return CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "prek detected and installed".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            };
        }
        // Another tool owns the hook — name it rather than suggest a re-install.
        if let Some(owner) = hook_owner(repo_root) {
            return CheckResult {
                name: name.to_string(),
                status: Status::Info,
                message: format!(
                    "prek.toml found but hook is owned by {} — chain prek or use {}'s runner",
                    owner, owner
                ),
                intent,
                success_criteria,
                suggestion: None,
            };
        }
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "prek.toml found but hooks not installed — run: prek install".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else if lefthook_config.exists() || lefthook_config_yaml.exists() {
        if hook_contains(repo_root, "lefthook") {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "lefthook detected and installed".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Info,
                message: "lefthook.yml found but hooks not installed — run: lefthook install"
                    .to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        }
    } else if husky_dir.exists() && husky_dir.is_dir() {
        if hook_exists_nonempty(repo_root) {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "husky detected and installed".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Info,
                message: ".husky/ found but hooks not installed — run: npx husky install"
                    .to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        }
    } else if precommit_config.exists() {
        if hook_contains(repo_root, "pre-commit") {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "pre-commit detected and installed".to_string(),
                intent,
                success_criteria,
                suggestion: Some(
                    "Consider prek for simpler hook management — https://github.com/chshersh/prek"
                        .to_string(),
                ),
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Info,
                message: ".pre-commit-config.yaml found but hooks not installed — run: pre-commit install".to_string(),
                intent,
                success_criteria,
                suggestion: Some(
                    "Consider prek for simpler hook management — https://github.com/chshersh/prek"
                        .to_string(),
                ),
            }
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No git hook manager detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Add prek to manage git hooks — https://github.com/chshersh/prek".to_string(),
            ),
        }
    }
}

fn check_editorconfig(repo_root: &Path) -> CheckResult {
    let name = "Consistent formatting";
    let intent =
        Some("Ensure consistent code formatting across different editors and IDEs.".to_string());
    let success_criteria =
        Some("Project-wide style rules are enforced by a shared configuration file.".to_string());

    let editorconfig = repo_root.join(".editorconfig");

    if editorconfig.exists() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: ".editorconfig detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No .editorconfig detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Add .editorconfig to standardize formatting — https://editorconfig.org"
                    .to_string(),
            ),
        }
    }
}

fn detect_doc_tool(repo_root: &Path) -> Option<String> {
    if repo_root.join("Cargo.toml").exists() {
        return Some("cargo doc".to_string());
    }
    if repo_root.join("mkdocs.yml").exists() || repo_root.join("mkdocs.yaml").exists() {
        return Some("mkdocs".to_string());
    }
    if repo_root.join("docs").join("conf.py").exists() {
        return Some("sphinx".to_string());
    }
    if repo_root.join("typedoc.json").exists() || repo_root.join(".typedoc.json").exists() {
        return Some("typedoc".to_string());
    }
    if repo_root.join("go.mod").exists() {
        return Some("godoc".to_string());
    }
    None
}

fn check_documentation(repo_root: &Path) -> CheckResult {
    let name = "Project documentation";
    let intent = Some(
        "Provide essential project identity, onboarding, and a discoverable docs/ folder with generated API docs."
            .to_string(),
    );
    let success_criteria = Some(
        "README, .gitignore, a docs/ folder with content, a language doc tool, and a 'just docs' recipe are all present."
            .to_string(),
    );

    let readme = repo_root.join("README.md").exists();
    let license = repo_root.join("LICENSE").exists() || repo_root.join("LICENSE.md").exists();
    let contributing = repo_root.join("CONTRIBUTING.md").exists();
    let gitignore = repo_root.join(".gitignore").exists();

    // docs/ folder with at least one file
    let docs_dir = repo_root.join("docs");
    let has_docs_folder = docs_dir.is_dir()
        && std::fs::read_dir(&docs_dir)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false);

    // Language-specific doc tool
    let doc_tool = detect_doc_tool(repo_root);

    // just docs recipe
    let justfile = repo_root.join("justfile");
    let has_just_docs = if justfile.exists() {
        parse_justfile_recipes(&justfile).contains(&"docs".to_string())
    } else {
        false
    };

    // Collect missing critical files
    let mut missing_critical: Vec<&str> = Vec::new();
    if !readme {
        missing_critical.push("README.md");
    }
    if !gitignore {
        missing_critical.push(".gitignore");
    }

    // Collect improvement suggestions
    let mut suggestions: Vec<String> = Vec::new();
    if !license {
        suggestions.push("LICENSE".to_string());
    }
    if !contributing {
        suggestions.push("CONTRIBUTING.md".to_string());
    }
    if !has_docs_folder {
        suggestions.push("docs/ folder with content".to_string());
    }
    if doc_tool.is_none() {
        suggestions
            .push("language doc tool (e.g. cargo doc, mkdocs, sphinx, typedoc, godoc)".to_string());
    }
    if justfile.exists() && !has_just_docs {
        suggestions.push("just docs recipe".to_string());
    }

    if !missing_critical.is_empty() {
        return CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: format!("Missing critical files: {}", missing_critical.join(", ")),
            intent,
            success_criteria,
            suggestion: Some(format!(
                "Add: {}{}",
                missing_critical.join(", "),
                if suggestions.is_empty() {
                    String::new()
                } else {
                    format!("; also consider: {}", suggestions.join(", "))
                }
            )),
        };
    }

    if suggestions.is_empty() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: format!(
                "Complete (doc tool: {})",
                doc_tool.as_deref().unwrap_or("detected")
            ),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "Essential files present".to_string(),
            intent,
            success_criteria,
            suggestion: Some(format!("Consider adding: {}", suggestions.join(", "))),
        }
    }
}

fn check_ai_instructions(repo_root: &Path) -> CheckResult {
    use crate::config::reflections_dir;

    let name = "AI-agent context";
    let intent = Some(
        "Provide persistent \"rules of the road\" and project context for AI collaborators."
            .to_string(),
    );
    let success_criteria = Some(
        "Persistent instructions define coding standards and context for AI assistants."
            .to_string(),
    );

    let claude_md = repo_root.join("CLAUDE.md");
    let agents_md = repo_root.join("AGENTS.md");

    if claude_md.exists() {
        // Check whether wai reflect has been run: a reflection resource file
        // must exist in .wai/resources/reflections/. The old WAI:REFLECT inline
        // block is no longer written — reflect now writes to a resource file and
        // injects a slim WAI:REFLECT:REF reference block instead.
        let refl_dir = reflections_dir(repo_root);
        let has_reflections = refl_dir
            .exists()
            .then(|| std::fs::read_dir(&refl_dir).ok())
            .flatten()
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false);
        let suggestion = if !has_reflections {
            Some(
                "No reflection resource found — run `wai reflect` to synthesize project-specific AI guidance into .wai/resources/reflections/".to_string(),
            )
        } else {
            None
        };
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "CLAUDE.md detected (recommended for Claude Code)".to_string(),
            intent,
            success_criteria,
            suggestion,
        }
    } else if agents_md.exists() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "AGENTS.md detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some("Consider adding CLAUDE.md for Claude Code compatibility".to_string()),
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No AI instruction files detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some("Create CLAUDE.md to provide context to AI assistants".to_string()),
        }
    }
}

fn check_ci_cd(repo_root: &Path) -> CheckResult {
    let name = "Automated verification";
    let intent = Some(
        "Ensure code quality and correctness through automated builds and tests on every change."
            .to_string(),
    );
    let success_criteria = Some(
        "Every change is automatically validated by a remote build/test pipeline.".to_string(),
    );

    let github_workflows = repo_root.join(".github/workflows");
    let gitlab_ci = repo_root.join(".gitlab-ci.yml");
    let circleci = repo_root.join(".circleci/config.yml");

    if github_workflows.exists() && github_workflows.is_dir() {
        let workflow_count = std::fs::read_dir(&github_workflows)
            .ok()
            .map(|entries| entries.filter_map(|e| e.ok()).count())
            .unwrap_or(0);

        if workflow_count > 0 {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: format!("GitHub Actions configured ({} workflow(s))", workflow_count),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Info,
                message: "GitHub Actions directory present but empty".to_string(),
                intent,
                success_criteria,
                suggestion: Some("Add workflow files to .github/workflows/".to_string()),
            }
        }
    } else if gitlab_ci.exists() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "GitLab CI configured".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else if circleci.exists() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "CircleCI configured".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No CI/CD configuration detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some("Set up continuous integration to automate testing".to_string()),
        }
    }
}

fn check_devcontainer(repo_root: &Path) -> CheckResult {
    let name = "Reproducible environments";
    let intent =
        Some("Provide a standardized, containerized environment for all contributors.".to_string());
    let success_criteria = Some(
        "A configuration exists to spin up a consistent, reproducible dev environment.".to_string(),
    );

    let devcontainer_dir = repo_root.join(".devcontainer");
    let devcontainer_json = repo_root.join(".devcontainer.json");

    if devcontainer_dir.exists() && devcontainer_dir.is_dir() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: ".devcontainer/ directory detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else if devcontainer_json.exists() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: ".devcontainer.json detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No dev container configuration detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Consider adding .devcontainer/ for reproducible development environments"
                    .to_string(),
            ),
        }
    }
}

fn check_llm_txt(repo_root: &Path) -> CheckResult {
    let name = "LLM-friendly context";
    let intent =
        Some("Provide machine-readable project context and navigation for LLMs.".to_string());
    let success_criteria =
        Some("Machine-readable project documentation (llm.txt) exists for AI tools.".to_string());

    let llm_txt = repo_root.join("llm.txt");

    if llm_txt.exists() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "llm.txt detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No llm.txt detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Add llm.txt for AI-friendly project documentation — https://llmstxt.org"
                    .to_string(),
            ),
        }
    }
}

fn check_agent_skills(repo_root: &Path) -> CheckResult {
    let name = "Extended agent capabilities";
    let intent = Some(
        "Enhance agent functionality with specialized iterative review and commit workflows."
            .to_string(),
    );
    let success_criteria =
        Some("Specialized agent workflows (Rule of 5, Deliberate Commits) are active.".to_string());

    let skills_dir = agent_config_dir(repo_root).join(SKILLS_DIR);

    if !skills_dir.exists() {
        return CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No skills configured".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Run 'wai way --fix skills' to scaffold rule-of-5-universal (ro5) and commit"
                    .to_string(),
            ),
        };
    }

    // Collect skill dir names and aliases from frontmatter
    let mut skill_ids: HashSet<String> = HashSet::new();
    let mut skill_count = 0usize;

    if let Ok(entries) = std::fs::read_dir(&skills_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let skill_file = entry.path().join("SKILL.md");
            if skill_file.exists() {
                skill_count += 1;
                if let Some(dir_name) = entry.file_name().to_str() {
                    skill_ids.insert(dir_name.to_string());
                }
                if let Some(meta) = parse_skill_frontmatter(&skill_file) {
                    for alias in meta.aliases {
                        skill_ids.insert(alias);
                    }
                }
            }
        }
    }

    if skill_count == 0 {
        return CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "Skills directory present but empty".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Run 'wai way --fix skills' to scaffold rule-of-5-universal (ro5) and commit"
                    .to_string(),
            ),
        };
    }

    let has_ro5 = skill_ids.contains("rule-of-5-universal") || skill_ids.contains("ro5");
    let has_commit = skill_ids.contains("commit");

    let missing: Vec<&str> = [
        (!has_ro5).then_some("rule-of-5-universal (ro5)"),
        (!has_commit).then_some("commit"),
    ]
    .into_iter()
    .flatten()
    .collect();

    if missing.is_empty() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: format!(
                "{} skill(s) configured — includes rule-of-5-universal (ro5) and commit",
                skill_count
            ),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: format!(
                "{} skill(s) configured — missing recommended: {}",
                skill_count,
                missing.join(", ")
            ),
            intent,
            success_criteria,
            suggestion: Some(format!(
                "Run 'wai way --fix skills' to scaffold missing: {}",
                missing.join(", ")
            )),
        }
    }
}

fn check_release_pipeline(repo_root: &Path) -> CheckResult {
    let name = "Automated delivery";
    let intent = Some(
        "Automate the process of building, packaging, and publishing software releases."
            .to_string(),
    );
    let success_criteria = Some(
        "Software releases and distribution (packages, binaries) are fully automated.".to_string(),
    );

    if !has_binary_target(repo_root) {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "Library project — release pipeline not required".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    if repo_root.join(".goreleaser.yml").exists() || repo_root.join(".goreleaser.yaml").exists() {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "goreleaser detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    if repo_root.join("dist.toml").exists() || has_cargo_dist_in_toml(repo_root) {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "cargo-dist detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    if has_release_workflow(repo_root) {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "GitHub Actions release workflow detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    CheckResult {
        name: name.to_string(),
        status: Status::Info,
        message: "No release pipeline found".to_string(),
        intent,
        success_criteria,
        suggestion: Some(
            "Consider goreleaser (Go/Rust/any) or cargo-dist (Rust) to automate GitHub releases and publish to Homebrew/Scoop".to_string(),
        ),
    }
}

fn has_binary_target(repo_root: &Path) -> bool {
    // Rust: src/main.rs or [[bin]] in Cargo.toml
    if repo_root.join("src/main.rs").exists() {
        return true;
    }
    if let Ok(content) = std::fs::read_to_string(repo_root.join("Cargo.toml"))
        && content.contains("[[bin]]")
    {
        return true;
    }
    // Go: any top-level .go file with "package main"
    if let Ok(entries) = std::fs::read_dir(repo_root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("go")
                && let Ok(content) = std::fs::read_to_string(&path)
                && content.contains("package main")
            {
                return true;
            }
        }
    }
    // Python: [project.scripts] or [tool.poetry.scripts] in pyproject.toml
    if let Ok(content) = std::fs::read_to_string(repo_root.join("pyproject.toml"))
        && (content.contains("[project.scripts]") || content.contains("[tool.poetry.scripts]"))
    {
        return true;
    }
    false
}

fn has_cargo_dist_in_toml(repo_root: &Path) -> bool {
    if let Ok(content) = std::fs::read_to_string(repo_root.join("Cargo.toml")) {
        return content.contains("[workspace.metadata.dist]")
            || content.contains("[package.metadata.dist]");
    }
    false
}

fn has_release_workflow(repo_root: &Path) -> bool {
    let workflows_dir = repo_root.join(".github/workflows");
    if !workflows_dir.exists() {
        return false;
    }
    if let Ok(entries) = std::fs::read_dir(&workflows_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let is_yaml = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == "yml" || e == "yaml");
            if !is_yaml {
                continue;
            }
            // Filename contains "release"
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains("release"))
            {
                return true;
            }
            // Content has tag trigger matching "v*"
            if let Ok(content) = std::fs::read_to_string(&path)
                && content.contains("tags:")
                && content.contains("v*")
            {
                return true;
            }
        }
    }
    false
}

fn check_test_coverage(repo_root: &Path) -> CheckResult {
    let name = "Test coverage";
    let intent = Some(
        "Enforced coverage thresholds catch regressions automatically and keep quality high."
            .to_string(),
    );
    let success_criteria =
        Some("A coverage tool is configured with a minimum threshold.".to_string());

    // Rust — tarpaulin
    let tarpaulin_toml = repo_root.join("tarpaulin.toml");
    if tarpaulin_toml.exists() {
        let has_threshold = std::fs::read_to_string(&tarpaulin_toml)
            .ok()
            .is_some_and(|c| c.contains("fail-under"));
        return if has_threshold {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "tarpaulin configured (threshold enforced)".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "tarpaulin configured".to_string(),
                intent,
                success_criteria,
                suggestion: Some(
                    "Add `fail-under` to tarpaulin config to enforce a minimum — https://github.com/xd009642/tarpaulin".to_string(),
                ),
            }
        };
    }

    // Rust — tarpaulin via Cargo.toml metadata
    if let Ok(cargo_content) = std::fs::read_to_string(repo_root.join("Cargo.toml")) {
        if cargo_content.contains("[package.metadata.tarpaulin]")
            || cargo_content.contains("[workspace.metadata.tarpaulin]")
        {
            let has_threshold = cargo_content.contains("fail-under");
            return if has_threshold {
                CheckResult {
                    name: name.to_string(),
                    status: Status::Pass,
                    message: "tarpaulin configured (threshold enforced)".to_string(),
                    intent,
                    success_criteria,
                    suggestion: None,
                }
            } else {
                CheckResult {
                    name: name.to_string(),
                    status: Status::Pass,
                    message: "tarpaulin configured".to_string(),
                    intent,
                    success_criteria,
                    suggestion: Some(
                        "Add `fail-under` to tarpaulin config to enforce a minimum — https://github.com/xd009642/tarpaulin".to_string(),
                    ),
                }
            };
        }
        // Rust — cargo-llvm-cov via dev-dependencies
        if cargo_content.contains("cargo-llvm-cov") {
            return CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "cargo-llvm-cov detected".to_string(),
                intent,
                success_criteria,
                suggestion: Some(
                    "Configure a threshold via CI flags or llvm-cov config — https://github.com/taiki-e/cargo-llvm-cov".to_string(),
                ),
            };
        }
    }

    // Python — coverage.py / pytest-cov
    let coveragerc = repo_root.join(".coveragerc");
    if coveragerc.exists() {
        let has_threshold = std::fs::read_to_string(&coveragerc)
            .ok()
            .is_some_and(|c| c.contains("fail_under"));
        return if has_threshold {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "coverage.py configured (threshold enforced)".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "coverage.py configured".to_string(),
                intent,
                success_criteria,
                suggestion: Some(
                    "Add `fail_under` to [tool.coverage.report] in pyproject.toml — https://coverage.readthedocs.io".to_string(),
                ),
            }
        };
    }
    if let Ok(pyproject) = std::fs::read_to_string(repo_root.join("pyproject.toml"))
        && pyproject.contains("[tool.coverage.report]")
    {
        let has_threshold = pyproject.contains("fail_under");
        return if has_threshold {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "coverage.py configured (threshold enforced)".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "coverage.py configured".to_string(),
                intent,
                success_criteria,
                suggestion: Some(
                    "Add `fail_under` to [tool.coverage.report] in pyproject.toml — https://coverage.readthedocs.io".to_string(),
                ),
            }
        };
    }

    // JavaScript / TypeScript — vitest
    for config_name in &["vitest.config.ts", "vitest.config.js", "vitest.config.mts"] {
        let config_path = repo_root.join(config_name);
        if let Ok(content) = std::fs::read_to_string(&config_path)
            && content.contains("coverage")
        {
            let has_threshold = content.contains("thresholds");
            return if has_threshold {
                CheckResult {
                    name: name.to_string(),
                    status: Status::Pass,
                    message: "vitest coverage configured (threshold enforced)".to_string(),
                    intent,
                    success_criteria,
                    suggestion: None,
                }
            } else {
                CheckResult {
                    name: name.to_string(),
                    status: Status::Pass,
                    message: "vitest coverage configured".to_string(),
                    intent,
                    success_criteria,
                    suggestion: Some(
                        "Add `thresholds` to the coverage block in vitest.config — https://vitest.dev/config/#coverage".to_string(),
                    ),
                }
            };
        }
    }

    // JavaScript / TypeScript — nyc or c8
    let nycrc = repo_root.join(".nycrc");
    let nycrc_json = repo_root.join(".nycrc.json");
    let c8_config = repo_root.join("c8.config.js");
    if nycrc.exists() || nycrc_json.exists() || c8_config.exists() {
        let content = nycrc
            .exists()
            .then(|| std::fs::read_to_string(&nycrc).ok())
            .flatten()
            .or_else(|| {
                nycrc_json
                    .exists()
                    .then(|| std::fs::read_to_string(&nycrc_json).ok())
                    .flatten()
            })
            .or_else(|| {
                c8_config
                    .exists()
                    .then(|| std::fs::read_to_string(&c8_config).ok())
                    .flatten()
            })
            .unwrap_or_default();
        let has_threshold = content.contains("branches") || content.contains("lines");
        return if has_threshold {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "nyc/c8 configured (threshold enforced)".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "nyc/c8 configured".to_string(),
                intent,
                success_criteria,
                suggestion: Some(
                    "Add branch/line thresholds to nyc or c8 config — https://github.com/istanbuljs/nyc · https://github.com/bcoe/c8".to_string(),
                ),
            }
        };
    }
    if let Ok(pkg) = std::fs::read_to_string(repo_root.join("package.json"))
        && (pkg.contains("\"nyc\"") || pkg.contains("\"c8\""))
    {
        let has_threshold = pkg.contains("branches") || pkg.contains("lines");
        return if has_threshold {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "nyc/c8 configured (threshold enforced)".to_string(),
                intent,
                success_criteria,
                suggestion: None,
            }
        } else {
            CheckResult {
                name: name.to_string(),
                status: Status::Pass,
                message: "nyc/c8 configured".to_string(),
                intent,
                success_criteria,
                suggestion: Some(
                    "Add branch/line thresholds to nyc or c8 config — https://github.com/istanbuljs/nyc · https://github.com/bcoe/c8".to_string(),
                ),
            }
        };
    }

    // Any language — codecov / coveralls
    let codecov_yml = repo_root.join(".codecov.yml");
    let codecov_yaml = repo_root.join("codecov.yml");
    let coveralls_yml = repo_root.join(".coveralls.yml");
    if codecov_yml.exists() || codecov_yaml.exists() || coveralls_yml.exists() {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "Coverage reporting service detected (codecov/coveralls)".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Add a coverage threshold in the config to enforce minimums".to_string(),
            ),
        };
    }

    // Nothing detected
    CheckResult {
        name: name.to_string(),
        status: Status::Info,
        message: "No coverage tool configured".to_string(),
        intent,
        success_criteria,
        suggestion: Some(
            "Configure a coverage tool with an enforced threshold — Rust: https://github.com/xd009642/tarpaulin · Python: https://coverage.readthedocs.io · JS: https://github.com/istanbuljs/nyc".to_string(),
        ),
    }
}

fn check_beads(repo_root: &Path) -> CheckResult {
    let name = "Issue tracking";
    let intent = Some(
        "Structured task tracking keeps work visible and prevents context loss across sessions."
            .to_string(),
    );
    let success_criteria = Some(
        "A beads workspace (.beads/) exists for tracking issues and dependencies.".to_string(),
    );

    let beads_dir = repo_root.join(".beads");
    if beads_dir.exists() && beads_dir.is_dir() {
        let issues_jsonl = beads_dir.join("issues.jsonl");
        let message = if let Ok(content) = std::fs::read_to_string(&issues_jsonl) {
            let count = content.lines().filter(|l| !l.trim().is_empty()).count();
            format!("beads detected ({} issues tracked)", count)
        } else {
            "beads detected".to_string()
        };
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message,
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No issue tracker detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Initialize beads issue tracking — https://github.com/steveyegge/beads".to_string(),
            ),
        }
    }
}

fn check_openspec(repo_root: &Path) -> CheckResult {
    let name = "Change proposals";
    let intent = Some(
        "Formal change proposals prevent architectural drift and create a reviewable design record."
            .to_string(),
    );
    let success_criteria =
        Some("An openspec workspace (openspec/) exists for managing change proposals.".to_string());

    let openspec_dir = repo_root.join("openspec");
    if openspec_dir.exists() && openspec_dir.is_dir() {
        CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "openspec detected".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        }
    } else {
        CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No change proposal system detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Track architectural change proposals — https://github.com/Fission-AI/OpenSpec"
                    .to_string(),
            ),
        }
    }
}

fn check_gh_cli() -> CheckResult {
    let name = "Integration & automation";
    let intent = Some(
        "Streamline repository interactions (PRs, issues, releases) from the CLI.".to_string(),
    );
    let success_criteria = Some(
        "CLI tools are configured for seamless integration with the hosting provider.".to_string(),
    );

    let gh_installed = std::process::Command::new("gh")
        .arg("--version")
        .output()
        .is_ok();

    if !gh_installed {
        return CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "gh not installed".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Install gh CLI for better GitHub integration — https://cli.github.com".to_string(),
            ),
        };
    }

    let auth_status = std::process::Command::new("gh")
        .args(["auth", "status"])
        .output();

    match auth_status {
        Ok(output) if output.status.success() => CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "gh installed and authenticated".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        },
        _ => CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "gh installed but not authenticated".to_string(),
            intent,
            success_criteria,
            suggestion: Some("Run 'gh auth login' to authenticate".to_string()),
        },
    }
}

fn check_typos(repo_root: &Path) -> CheckResult {
    let name = "Spell checking";
    let intent = Some(
        "Catch typos in source code, comments, and documentation before they reach history."
            .to_string(),
    );
    let success_criteria =
        Some("A typos configuration exists to enforce spell checking.".to_string());

    let configs = [
        repo_root.join("typos.toml"),
        repo_root.join("_typos.toml"),
        repo_root.join(".typos.toml"),
    ];

    if let Some(found) = configs.iter().find(|p| p.exists()) {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: format!(
                "typos configured ({})",
                found.file_name().unwrap_or_default().to_string_lossy()
            ),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    // Check pyproject.toml for [tool.typos]
    if let Ok(content) = std::fs::read_to_string(repo_root.join("pyproject.toml"))
        && content.contains("[tool.typos]")
    {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "typos configured (pyproject.toml)".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    CheckResult {
        name: name.to_string(),
        status: Status::Info,
        message: "No typos configuration detected".to_string(),
        intent,
        success_criteria,
        suggestion: Some(
            "Add _typos.toml to catch spelling errors in code — https://github.com/crate-ci/typos"
                .to_string(),
        ),
    }
}

fn check_vale(repo_root: &Path) -> CheckResult {
    let name = "Prose linting";
    let intent = Some(
        "Enforce writing style and consistency across documentation and markdown files."
            .to_string(),
    );
    let success_criteria =
        Some("A vale configuration exists to enforce prose style rules.".to_string());

    let configs = [repo_root.join(".vale.ini"), repo_root.join("vale.ini")];

    if let Some(found) = configs.iter().find(|p| p.exists()) {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: format!(
                "vale configured ({})",
                found.file_name().unwrap_or_default().to_string_lossy()
            ),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    CheckResult {
        name: name.to_string(),
        status: Status::Info,
        message: "No vale configuration detected".to_string(),
        intent,
        success_criteria,
        suggestion: Some(
            "Add .vale.ini to lint prose in markdown and docs — https://vale.sh".to_string(),
        ),
    }
}

fn check_shell_linting(repo_root: &Path) -> CheckResult {
    let name = "Shell linting";
    let intent = Some(
        "Catch bugs, portability issues, and bad practices in shell scripts and CI workflow run blocks."
            .to_string(),
    );
    let success_criteria = Some(
        "Shell scripts and CI run blocks are validated by a linter (actionlint or shellcheck)."
            .to_string(),
    );

    // Detect shell-containing files
    let has_workflows = repo_root.join(".github/workflows").is_dir();

    let is_shell_ext = |p: &std::path::Path| {
        p.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "sh" || ext == "bash")
    };
    let dir_has_shell_files = |dir: &std::path::Path| {
        dir.is_dir()
            && std::fs::read_dir(dir)
                .ok()
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .any(|e| is_shell_ext(&e.path()))
                })
                .unwrap_or(false)
    };

    let has_shell_scripts = ["scripts", "bin", "script"]
        .iter()
        .any(|d| dir_has_shell_files(&repo_root.join(d)))
        || std::fs::read_dir(repo_root)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .any(|e| is_shell_ext(&e.path()))
            })
            .unwrap_or(false);

    if !has_workflows && !has_shell_scripts {
        return CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "No shell scripts or workflows to lint".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        };
    }

    // Check for actionlint (covers both workflow YAML and embedded shell via shellcheck)
    let has_actionlint = std::process::Command::new("actionlint")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());

    let has_shellcheck = std::process::Command::new("shellcheck")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());

    match (has_actionlint, has_shellcheck, has_workflows) {
        (true, true, _) => CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "actionlint and shellcheck available".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        },
        (true, false, _) => CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "actionlint available (install shellcheck for deeper run: block analysis)"
                .to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Install shellcheck so actionlint can lint embedded run: blocks — https://www.shellcheck.net"
                    .to_string(),
            ),
        },
        (false, true, false) => CheckResult {
            name: name.to_string(),
            status: Status::Pass,
            message: "shellcheck available".to_string(),
            intent,
            success_criteria,
            suggestion: None,
        },
        (false, true, true) => CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "shellcheck available but actionlint missing for workflow linting".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Install actionlint to lint GitHub Actions workflows (it uses shellcheck for run: blocks) — https://github.com/rhysd/actionlint"
                    .to_string(),
            ),
        },
        (false, false, true) => CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No shell linter detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Install actionlint + shellcheck to lint workflows and shell scripts — https://github.com/rhysd/actionlint"
                    .to_string(),
            ),
        },
        (false, false, false) => CheckResult {
            name: name.to_string(),
            status: Status::Info,
            message: "No shell linter detected".to_string(),
            intent,
            success_criteria,
            suggestion: Some(
                "Install shellcheck to lint shell scripts — https://www.shellcheck.net".to_string(),
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_git_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        dir
    }

    fn write_hook(dir: &tempfile::TempDir, hook_name: &str, content: &str) {
        let hooks_dir = dir.path().join(".git/hooks");
        fs::create_dir_all(&hooks_dir).unwrap();
        fs::write(hooks_dir.join(hook_name), content).unwrap();
    }

    fn write_prek_toml(dir: &tempfile::TempDir) {
        fs::write(
            dir.path().join("prek.toml"),
            "[hooks]\npre-commit = [\"cargo test\"]\n",
        )
        .unwrap();
    }

    // -- core.hooksPath conflict --

    #[test]
    fn prek_toml_with_core_hooks_path_set_reports_conflict() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        std::process::Command::new("git")
            .args(["config", "core.hooksPath", ".git/hooks"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Info);
        assert!(
            result.message.contains("core.hooksPath"),
            "expected core.hooksPath conflict, got: {}",
            result.message
        );
        let suggestion = result.suggestion.unwrap_or_default();
        assert!(
            suggestion.contains("unset"),
            "expected unset hint, got: {}",
            suggestion
        );
    }

    // -- hook owner detection --

    #[test]
    fn prek_toml_with_bd_shim_reports_bd_as_owner() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-commit",
            "#!/usr/bin/env sh\n# bd-shim v2\nexec bd hooks run pre-commit \"$@\"\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Info);
        assert!(
            result.message.contains("bd"),
            "expected bd as owner, got: {}",
            result.message
        );
    }

    #[test]
    fn prek_toml_with_lefthook_shim_reports_lefthook_as_owner() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-commit",
            "#!/usr/bin/env sh\nexec lefthook run pre-commit \"$@\"\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Info);
        assert!(
            result.message.contains("lefthook"),
            "expected lefthook as owner, got: {}",
            result.message
        );
    }

    // -- prek pass cases --

    #[test]
    fn prek_toml_with_prek_in_precommit_passes() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-commit",
            "#!/usr/bin/env sh\n# managed by prek\nexec prek run pre-commit\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Pass);
        assert_eq!(result.message, "prek detected and installed");
    }

    #[test]
    fn prek_toml_with_prek_in_prepush_passes() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-push",
            "#!/usr/bin/env sh\n# managed by prek\nexec prek run pre-push\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Pass);
        assert_eq!(result.message, "prek detected and installed");
    }

    #[test]
    fn bd_hook_that_chains_prek_passes() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);
        write_hook(
            &dir,
            "pre-commit",
            "#!/usr/bin/env sh\n# bd-shim v2\n# chains prek\nexec prek run pre-commit\n",
        );

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Pass);
    }

    // -- no hooks at all --

    #[test]
    fn prek_toml_with_no_hooks_suggests_install() {
        let dir = setup_git_repo();
        write_prek_toml(&dir);

        let result = check_git_hooks(dir.path());
        assert_eq!(result.status, Status::Info);
        assert!(
            result.message.contains("prek install"),
            "expected prek install suggestion, got: {}",
            result.message
        );
    }

    // -- no prek.toml --

    #[test]
    fn no_prek_toml_does_not_fire_prek_branch() {
        let dir = setup_git_repo();
        // No prek.toml written — prek branch must not trigger.

        let result = check_git_hooks(dir.path());
        assert!(
            !result.message.contains("prek.toml found"),
            "prek branch fired without prek.toml: {}",
            result.message
        );
    }

    // -- shell linting --

    #[test]
    fn shell_linting_passes_when_no_shell_content() {
        let dir = setup_git_repo();
        // No workflows, no .sh files → nothing to lint
        let result = check_shell_linting(dir.path());
        assert_eq!(result.status, Status::Pass);
        assert!(
            result.message.contains("No shell scripts or workflows"),
            "unexpected message: {}",
            result.message
        );
    }

    #[test]
    fn shell_linting_detects_workflows_dir() {
        let dir = setup_git_repo();
        let workflows = dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows).unwrap();
        fs::write(workflows.join("ci.yml"), "name: CI\n").unwrap();

        let result = check_shell_linting(dir.path());
        // Should not be "no shell content" — workflows exist
        assert!(
            !result.message.contains("No shell scripts or workflows"),
            "should detect workflows, got: {}",
            result.message
        );
    }

    #[test]
    fn shell_linting_ignores_scripts_dir_without_sh_files() {
        let dir = setup_git_repo();
        let scripts = dir.path().join("scripts");
        fs::create_dir_all(&scripts).unwrap();
        fs::write(scripts.join("deploy.py"), "print('hi')\n").unwrap();

        let result = check_shell_linting(dir.path());
        assert_eq!(result.status, Status::Pass);
        assert!(
            result.message.contains("No shell scripts or workflows"),
            "should ignore scripts dir without .sh files, got: {}",
            result.message
        );
    }

    #[test]
    fn shell_linting_detects_scripts_dir_with_sh_files() {
        let dir = setup_git_repo();
        let scripts = dir.path().join("scripts");
        fs::create_dir_all(&scripts).unwrap();
        fs::write(scripts.join("deploy.sh"), "#!/bin/bash\necho hi\n").unwrap();

        let result = check_shell_linting(dir.path());
        assert!(
            !result.message.contains("No shell scripts or workflows"),
            "should detect scripts dir with .sh files, got: {}",
            result.message
        );
    }

    #[test]
    fn shell_linting_detects_root_sh_files() {
        let dir = setup_git_repo();
        fs::write(dir.path().join("setup.sh"), "#!/bin/bash\necho hi\n").unwrap();

        let result = check_shell_linting(dir.path());
        assert!(
            !result.message.contains("No shell scripts or workflows"),
            "should detect .sh files, got: {}",
            result.message
        );
    }
}
