pub mod provider;
pub mod worker;

use crate::provider::{AiRequest, ProfileAiConfig, ReviewEngine};
use crate::worker::registry::WorkerRegistry;
use anyhow::{anyhow, Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub use dn_ipc::{WorkerRequest, WorkerResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOptions {
    pub profile_name: String,
    #[serde(default)]
    pub include_hidden: bool,
    #[serde(default)]
    pub include_content: bool,
    #[serde(default)]
    pub python_worker: bool,
    #[serde(default)]
    pub max_files: usize,
    #[serde(default)]
    pub format: OutputFormat,
    #[serde(default)]
    pub fail_on_severity: Option<String>,
    #[serde(default)]
    pub summary_only: bool,
    #[serde(default)]
    pub strict_integrations: bool,
    #[serde(default = "default_command_name")]
    pub command_name: String,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            profile_name: "quick".to_string(),
            include_hidden: false,
            include_content: false,
            python_worker: false,
            max_files: 10_000,
            format: OutputFormat::Text,
            fail_on_severity: None,
            summary_only: false,
            strict_integrations: false,
            command_name: default_command_name(),
        }
    }
}

fn default_command_name() -> String {
    "scan".to_string()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Markdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScanLimits {
    #[serde(default = "default_max_file_size")]
    pub max_file_size_bytes: u64,
    #[serde(default = "default_read_size")]
    pub max_file_read_bytes: usize,
    #[serde(default = "default_max_total_bytes")]
    pub max_total_bytes: u64,
    #[serde(default = "default_max_files")]
    pub max_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileSelectionConfig {
    #[serde(default)]
    pub include_hidden: bool,
    #[serde(default)]
    pub include_globs: Vec<String>,
    #[serde(default)]
    pub exclude_globs: Vec<String>,
    #[serde(default)]
    pub include_binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyzerConfig {
    #[serde(default = "default_detectors")]
    pub deterministic_rules: Vec<String>,
    #[serde(default)]
    pub suspicious_patterns: Vec<String>,
    #[serde(default)]
    pub prioritize: Vec<String>,
    #[serde(default)]
    pub min_severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerProfileConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_worker_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputConfig {
    #[serde(default)]
    pub include_content_preview: bool,
    #[serde(default)]
    pub severity_threshold: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeProfile {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub rules: AnalyzerConfig,
    #[serde(default)]
    pub file_selection: FileSelectionConfig,
    #[serde(default)]
    pub limits: ScanLimits,
    #[serde(default)]
    pub worker: WorkerProfileConfig,
    #[serde(default)]
    pub ai: ProfileAiConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub inherits: Option<String>,
    #[serde(default)]
    pub include_hidden: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveProfile {
    pub name: String,
    pub description: String,
    pub enabled_rules: Vec<String>,
    pub suspicious_patterns: Vec<String>,
    pub prioritize_rules: Vec<String>,
    pub min_severity: String,
    pub include_hidden: bool,
    pub include_globs: Vec<String>,
    pub exclude_globs: Vec<String>,
    pub include_binary: bool,
    pub limits: ScanLimits,
    pub worker_enabled: bool,
    pub worker_timeout_ms: u64,
    pub worker_retries: u32,
    pub ai: ProfileAiConfig,
    pub include_content_preview: bool,
    pub severity_threshold: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProfileSelection {
    pub root: String,
    pub chosen_profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub rule: String,
    pub severity: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integration_notes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityStats {
    pub info: usize,
    pub low: usize,
    pub medium: usize,
    pub high: usize,
    pub critical: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub root: String,
    pub profile: String,
    pub profile_source: String,
    pub command: String,
    pub output_format: OutputFormat,
    pub summary_only: bool,
    pub duration_ms: u128,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportStats {
    pub files_discovered: usize,
    pub files_scanned: usize,
    pub files_selected: usize,
    pub files_skipped: usize,
    pub total_files: usize,
    pub total_bytes: u64,
    pub skipped_large_files: usize,
    pub findings_total: usize,
    pub severity_breakdown: SeverityStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationRuntimeStatus {
    pub enabled: bool,
    pub mode: String,
    pub strict: bool,
    pub used: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_languages: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ai_files: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_sent: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationSummary {
    pub worker: IntegrationRuntimeStatus,
    pub provider: IntegrationRuntimeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: String,
    pub source: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub schema_version: String,
    pub metadata: ReportMetadata,
    pub stats: ReportStats,
    pub integrations: IntegrationSummary,
    pub diagnostics: Vec<Diagnostic>,
    pub files: Vec<FileEntry>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitEvaluation {
    pub threshold: Option<String>,
    pub threshold_triggered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOutcome {
    pub root: String,
    pub profile: String,
    pub provider: String,
    pub worker: String,
    pub profile_source: String,
    pub files_discovered: usize,
    pub files_scanned: usize,
    pub files_selected: usize,
    pub files_skipped: usize,
    pub total_files: usize,
    pub total_bytes: u64,
    pub skipped_large_files: usize,
    pub truncated: bool,
    pub errors: Vec<String>,
    pub files: Vec<FileEntry>,
    pub severity_breakdown: SeverityStats,
    pub duration_ms: u128,
    pub summary: String,
    pub report: ScanReport,
    pub exit_evaluation: ExitEvaluation,
}

#[derive(Debug)]
pub enum ProfileSource {
    Builtin,
    Loaded(PathBuf),
}

fn default_max_file_size() -> u64 {
    1024 * 1024
}

fn default_read_size() -> usize {
    32 * 1024
}

fn default_max_total_bytes() -> u64 {
    50 * 1024 * 1024
}

fn default_max_files() -> usize {
    20_000
}

fn default_worker_timeout_ms() -> u64 {
    10_000
}

fn default_detectors() -> Vec<String> {
    vec![
        "todo-comment".to_string(),
        "unsafe-usage".to_string(),
        "possible-secret".to_string(),
    ]
}

impl Default for ScanLimits {
    fn default() -> Self {
        Self {
            max_file_size_bytes: default_max_file_size(),
            max_file_read_bytes: default_read_size(),
            max_total_bytes: default_max_total_bytes(),
            max_files: default_max_files(),
        }
    }
}

impl Default for FileSelectionConfig {
    fn default() -> Self {
        Self {
            include_hidden: false,
            include_globs: Vec::new(),
            exclude_globs: vec![
                ".git/**".to_string(),
                "target/**".to_string(),
                "node_modules/**".to_string(),
            ],
            include_binary: false,
        }
    }
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            deterministic_rules: default_detectors(),
            suspicious_patterns: vec![
                "password".to_string(),
                "secret".to_string(),
                "api_key".to_string(),
                "token=".to_string(),
            ],
            prioritize: vec!["possible-secret".to_string(), "unsafe-usage".to_string()],
            min_severity: "info".to_string(),
        }
    }
}

impl Default for WorkerProfileConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            timeout_ms: default_worker_timeout_ms(),
            retries: 1,
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            include_content_preview: false,
            severity_threshold: "info".to_string(),
        }
    }
}

impl Default for RuntimeProfile {
    fn default() -> Self {
        Self {
            name: "quick".to_string(),
            description: "Quick local-only baseline scan".to_string(),
            rules: AnalyzerConfig::default(),
            file_selection: FileSelectionConfig::default(),
            limits: ScanLimits::default(),
            worker: WorkerProfileConfig::default(),
            ai: ProfileAiConfig::default(),
            output: OutputConfig::default(),
            inherits: None,
            include_hidden: None,
        }
    }
}

impl RuntimeProfile {
    fn merge(self, parent: &RuntimeProfile) -> RuntimeProfile {
        RuntimeProfile {
            name: self.name,
            include_hidden: if self.include_hidden.is_some() {
                self.include_hidden
            } else {
                parent.include_hidden
            },
            description: if self.description.is_empty() {
                parent.description.clone()
            } else {
                self.description
            },
            rules: if self.rules.deterministic_rules.is_empty()
                && self.rules.suspicious_patterns.is_empty()
            {
                parent.rules.clone()
            } else {
                AnalyzerConfig {
                    deterministic_rules: if self.rules.deterministic_rules.is_empty() {
                        parent.rules.deterministic_rules.clone()
                    } else {
                        self.rules.deterministic_rules.clone()
                    },
                    suspicious_patterns: if self.rules.suspicious_patterns.is_empty() {
                        parent.rules.suspicious_patterns.clone()
                    } else {
                        self.rules.suspicious_patterns.clone()
                    },
                    prioritize: if self.rules.prioritize.is_empty() {
                        parent.rules.prioritize.clone()
                    } else {
                        self.rules.prioritize.clone()
                    },
                    min_severity: if self.rules.min_severity.is_empty() {
                        parent.rules.min_severity.clone()
                    } else {
                        self.rules.min_severity.clone()
                    },
                }
            },
            file_selection: FileSelectionConfig {
                include_hidden: self.file_selection.include_hidden
                    || self.include_hidden.unwrap_or(false)
                    || parent.file_selection.include_hidden,
                include_globs: if self.file_selection.include_globs.is_empty() {
                    parent.file_selection.include_globs.clone()
                } else {
                    self.file_selection.include_globs.clone()
                },
                exclude_globs: if self.file_selection.exclude_globs.is_empty() {
                    parent.file_selection.exclude_globs.clone()
                } else {
                    self.file_selection.exclude_globs.clone()
                },
                include_binary: self.file_selection.include_binary
                    || parent.file_selection.include_binary,
            },
            limits: ScanLimits {
                max_file_size_bytes: if self.limits.max_file_size_bytes == 0 {
                    parent.limits.max_file_size_bytes
                } else {
                    self.limits.max_file_size_bytes
                },
                max_file_read_bytes: if self.limits.max_file_read_bytes == 0 {
                    parent.limits.max_file_read_bytes
                } else {
                    self.limits.max_file_read_bytes
                },
                max_total_bytes: if self.limits.max_total_bytes == 0 {
                    parent.limits.max_total_bytes
                } else {
                    self.limits.max_total_bytes
                },
                max_files: if self.limits.max_files == 0 {
                    parent.limits.max_files
                } else {
                    self.limits.max_files
                },
            },
            worker: WorkerProfileConfig {
                enabled: self.worker.enabled || parent.worker.enabled,
                timeout_ms: if self.worker.timeout_ms == 0 {
                    parent.worker.timeout_ms
                } else {
                    self.worker.timeout_ms
                },
                retries: if self.worker.retries == 0 {
                    parent.worker.retries
                } else {
                    self.worker.retries
                },
            },
            ai: if self.ai.enabled || !self.ai.prompt.is_empty() {
                if self.ai.enabled {
                    self.ai
                } else {
                    parent.ai.clone()
                }
            } else {
                parent.ai.clone()
            },
            output: OutputConfig {
                include_content_preview: if self.output.include_content_preview {
                    true
                } else {
                    parent.output.include_content_preview
                },
                severity_threshold: if self.output.severity_threshold.is_empty() {
                    parent.output.severity_threshold.clone()
                } else {
                    self.output.severity_threshold.clone()
                },
            },
            inherits: None,
        }
    }

    fn into_effective(self) -> EffectiveProfile {
        EffectiveProfile {
            name: self.name,
            description: self.description,
            enabled_rules: self.rules.deterministic_rules,
            suspicious_patterns: self.rules.suspicious_patterns,
            prioritize_rules: self.rules.prioritize,
            min_severity: self.rules.min_severity,
            include_hidden: self
                .include_hidden
                .unwrap_or(self.file_selection.include_hidden),
            include_globs: self.file_selection.include_globs,
            exclude_globs: self.file_selection.exclude_globs,
            include_binary: self.file_selection.include_binary,
            limits: self.limits,
            worker_enabled: self.worker.enabled,
            worker_timeout_ms: self.worker.timeout_ms,
            worker_retries: self.worker.retries,
            ai: self.ai,
            include_content_preview: self.output.include_content_preview,
            severity_threshold: self.output.severity_threshold,
        }
    }
}

fn is_path_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

fn builtin_profile(name: &str) -> Option<RuntimeProfile> {
    let base = RuntimeProfile {
        name: name.to_string(),
        description: "Local-first profile".to_string(),
        ..RuntimeProfile::default()
    };

    match name {
        "quick" => Some(RuntimeProfile {
            name: name.to_string(),
            limits: ScanLimits {
                max_file_size_bytes: 512 * 1024,
                max_file_read_bytes: 8 * 1024,
                max_total_bytes: 10 * 1024 * 1024,
                max_files: 500,
            },
            worker: WorkerProfileConfig {
                enabled: false,
                timeout_ms: 5_000,
                retries: 0,
            },
            ..base
        }),
        "security" => {
            let mut profile = base;
            profile.name = "security".to_string();
            profile.description = "Security-sensitive review profile".to_string();
            profile.rules.deterministic_rules = vec![
                "todo-comment".to_string(),
                "unsafe-usage".to_string(),
                "possible-secret".to_string(),
                "hardcoded-value".to_string(),
            ];
            profile.rules.suspicious_patterns = vec![
                "password".to_string(),
                "secret".to_string(),
                "token".to_string(),
                "api_key".to_string(),
                "client_secret".to_string(),
            ];
            profile.rules.prioritize = vec![
                "possible-secret".to_string(),
                "hardcoded-value".to_string(),
                "unsafe-usage".to_string(),
            ];
            profile.ai.enabled = true;
            profile.ai.provider = provider::ProviderConfig::Mock {
                message: "No security finding by AI in this file snippet".to_string(),
            };
            profile.ai.suspicious_patterns = vec![
                "password".to_string(),
                "secret".to_string(),
                "token".to_string(),
                "api_key".to_string(),
                "unsafe".to_string(),
            ];
            profile.worker.enabled = true;
            Some(profile)
        }
        "architecture" => Some(RuntimeProfile {
            description: "Architecture review profile".to_string(),
            rules: AnalyzerConfig {
                deterministic_rules: vec![
                    "todo-comment".to_string(),
                    "large-file".to_string(),
                    "hard-to-read-function".to_string(),
                ],
                suspicious_patterns: vec![
                    "TODO".to_string(),
                    "FIXME".to_string(),
                    "XXX".to_string(),
                ],
                prioritize: vec!["large-file".to_string(), "todo-comment".to_string()],
                min_severity: "info".to_string(),
            },
            ..base
        }),
        "deep" => Some(RuntimeProfile {
            name: name.to_string(),
            description: "Deep local analysis profile".to_string(),
            limits: ScanLimits {
                max_file_size_bytes: 2 * 1024 * 1024,
                max_file_read_bytes: 64 * 1024,
                max_total_bytes: 120 * 1024 * 1024,
                max_files: 20000,
            },
            ai: ProfileAiConfig {
                enabled: true,
                max_ai_files: 80,
                ..ProfileAiConfig::default()
            },
            ..base
        }),
        "performance" => Some(RuntimeProfile {
            description: "Performance-focused review profile".to_string(),
            limits: ScanLimits {
                max_file_size_bytes: 2 * 1024 * 1024,
                max_file_read_bytes: 32 * 1024,
                max_total_bytes: 80 * 1024 * 1024,
                max_files: 15_000,
            },
            ..base
        }),
        "maintainability" => Some(RuntimeProfile {
            description: "Maintainability-focused review profile".to_string(),
            worker: WorkerProfileConfig {
                enabled: true,
                timeout_ms: 8_000,
                retries: 1,
            },
            rules: AnalyzerConfig {
                deterministic_rules: vec![
                    "todo-comment".to_string(),
                    "possible-secret".to_string(),
                    "hardcoded-value".to_string(),
                ],
                suspicious_patterns: vec![
                    "TODO".to_string(),
                    "FIXME".to_string(),
                    "deprecated".to_string(),
                    "unsafe".to_string(),
                ],
                prioritize: vec!["todo-comment".to_string()],
                min_severity: "info".to_string(),
            },
            ..base
        }),
        "ai-generated-code-review" => {
            let mut p = base;
            p.name = name.to_string();
            p.description = "AI-generated code review profile".to_string();
            p.ai.enabled = true;
            p.ai.suspicious_patterns = vec![
                "todo".to_string(),
                "fixme".to_string(),
                "unsafe".to_string(),
                "eval(".to_string(),
            ];
            p.ai.provider = provider::ProviderConfig::Mock {
                message:
                    "AI-generated code smell: check for repetitive patterns and naming quality."
                        .to_string(),
            };
            p.worker.enabled = true;
            Some(p)
        }
        "legacy-modernization" => Some(RuntimeProfile {
            name: name.to_string(),
            description: "Legacy modernization profile".to_string(),
            rules: AnalyzerConfig {
                deterministic_rules: vec![
                    "todo-comment".to_string(),
                    "deprecated-api".to_string(),
                    "possible-secret".to_string(),
                ],
                suspicious_patterns: vec![
                    "TODO".to_string(),
                    "FIXME".to_string(),
                    "XXX".to_string(),
                    "deprecated".to_string(),
                    "legacy".to_string(),
                    "unsafe".to_string(),
                ],
                prioritize: vec!["deprecated-api".to_string(), "possible-secret".to_string()],
                min_severity: "low".to_string(),
            },
            ..base
        }),
        "pre-merge" => Some(RuntimeProfile {
            name: name.to_string(),
            description: "Pre-merge quality gate profile".to_string(),
            limits: ScanLimits {
                max_file_size_bytes: 512 * 1024,
                max_file_read_bytes: 16 * 1024,
                max_total_bytes: 40 * 1024 * 1024,
                max_files: 5000,
            },
            output: OutputConfig {
                include_content_preview: false,
                severity_threshold: "medium".to_string(),
            },
            ..base
        }),
        "strict" => Some(RuntimeProfile {
            name: name.to_string(),
            description: "Strict policy profile".to_string(),
            output: OutputConfig {
                include_content_preview: true,
                severity_threshold: "low".to_string(),
            },
            ai: ProfileAiConfig {
                enabled: true,
                max_ai_files: 120,
                max_content_chars: 8 * 1024,
                suspicious_patterns: vec![
                    "password".to_string(),
                    "secret".to_string(),
                    "token".to_string(),
                    "unsafe".to_string(),
                    "eval(".to_string(),
                ],
                ..ProfileAiConfig::default()
            },
            ..base
        }),
        "educational" => Some(RuntimeProfile {
            name: name.to_string(),
            description: "Educational profile with guidance focus".to_string(),
            ai: ProfileAiConfig {
                enabled: true,
                provider: provider::ProviderConfig::Mock {
                    message: "Explainability guidance requested by educational profile".to_string(),
                },
                ..ProfileAiConfig::default()
            },
            ..base
        }),
        "production-readiness" => Some(RuntimeProfile {
            name: name.to_string(),
            description: "Production readiness checks".to_string(),
            limits: ScanLimits {
                max_file_size_bytes: 2 * 1024 * 1024,
                max_file_read_bytes: 16 * 1024,
                max_total_bytes: 80 * 1024 * 1024,
                max_files: 15000,
            },
            rules: AnalyzerConfig {
                deterministic_rules: vec![
                    "todo-comment".to_string(),
                    "possible-secret".to_string(),
                    "hardcoded-value".to_string(),
                    "unsafe-usage".to_string(),
                ],
                suspicious_patterns: vec![
                    "TODO".to_string(),
                    "FIXME".to_string(),
                    "api_key".to_string(),
                    "password".to_string(),
                    "secret".to_string(),
                    "unsafe".to_string(),
                ],
                prioritize: vec![
                    "possible-secret".to_string(),
                    "hardcoded-value".to_string(),
                    "unsafe-usage".to_string(),
                ],
                min_severity: "medium".to_string(),
            },
            ..base
        }),
        _ => None,
    }
}

fn is_text_by_name(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => matches!(
            ext,
            "rs" | "toml"
                | "md"
                | "txt"
                | "json"
                | "yaml"
                | "yml"
                | "py"
                | "js"
                | "ts"
                | "tsx"
                | "jsx"
                | "java"
                | "c"
                | "h"
                | "cpp"
                | "hpp"
                | "go"
                | "rb"
                | "php"
                | "sh"
                | "html"
                | "css"
                | "sql"
                | "Dockerfile"
        ),
        None => false,
    }
}

fn can_read_text_preview(path: &Path, max_bytes: usize) -> bool {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return false,
    };

    let mut buffer = vec![0u8; max_bytes.max(1)];
    let read = file.read(&mut buffer).unwrap_or(0);
    if read == 0 {
        return true;
    }
    buffer.truncate(read);
    std::str::from_utf8(&buffer).is_ok()
}

fn detect_language(path: &Path) -> Option<String> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("py") => Some("python".to_string()),
        Some("rs") => Some("rust".to_string()),
        Some("js") => Some("javascript".to_string()),
        Some("ts") => Some("typescript".to_string()),
        Some("tsx") => Some("typescript".to_string()),
        Some("jsx") => Some("javascript".to_string()),
        Some("json") => Some("json".to_string()),
        Some("toml") => Some("toml".to_string()),
        Some("md") => Some("markdown".to_string()),
        Some("mdx") => Some("markdown".to_string()),
        _ => None,
    }
}

fn read_text_preview(path: &Path, max_bytes: usize) -> Result<String> {
    let mut file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut buffer = vec![0u8; max_bytes];
    let bytes_read = file
        .read(&mut buffer)
        .with_context(|| format!("read {}", path.display()))?;
    buffer.truncate(bytes_read);
    String::from_utf8(buffer)
        .map_err(|err| anyhow!("read text preview {}: {}", path.display(), err))
}

fn is_text_file(
    path: &Path,
    metadata: &std::fs::Metadata,
    include_binary: bool,
    max_read_bytes: usize,
) -> bool {
    if include_binary {
        return true;
    }

    if is_text_by_name(path) {
        return true;
    }

    if is_path_hidden(path)
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| !name.trim_start().is_empty())
    {
        return can_read_text_preview(path, max_read_bytes);
    }

    if metadata.len() == 0 {
        return true;
    }

    can_read_text_preview(path, max_read_bytes)
}

/// Check if content contains a word with boundary matching (not substring)
fn contains_word(content: &str, word: &str) -> bool {
    content
        .split_whitespace()
        .any(|w| w.trim_matches(|c: char| !c.is_alphanumeric()) == word)
}

fn normalized_code_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with("//")
        || trimmed.starts_with('#')
        || trimmed.starts_with('*')
        || trimmed.starts_with("/*")
        || trimmed.starts_with("--")
    {
        return None;
    }

    let before_inline_comment = trimmed
        .split_once("//")
        .map(|(head, _)| head)
        .unwrap_or(trimmed)
        .split_once(" #")
        .map(|(head, _)| head)
        .unwrap_or(trimmed)
        .trim();

    if before_inline_comment.is_empty() {
        return None;
    }

    Some(before_inline_comment.to_lowercase())
}

fn line_number_from_index(index: usize) -> u32 {
    (index + 1).try_into().unwrap_or(u32::MAX)
}

fn likely_doc_or_demo_line(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("example")
        || lower.contains("sample")
        || lower.contains("for example")
        || lower.contains("demo")
}

fn looks_like_placeholder_value(value: &str) -> bool {
    let normalized = value.trim_matches(|c: char| matches!(c, '"' | '\'' | '`' | ' '));
    if normalized.is_empty() {
        return true;
    }

    let lower = normalized.to_lowercase();
    let placeholders = [
        "example",
        "sample",
        "dummy",
        "fake",
        "mock",
        "test",
        "changeme",
        "change-me",
        "replace-me",
        "replace_this",
        "placeholder",
        "your_",
        "your-",
        "xxxx",
        "abc123",
        "foobar",
        "<secret>",
        "<token>",
        "<password>",
        "${",
        "process.env",
        "env(",
        "os.getenv",
    ];

    placeholders.iter().any(|token| lower.contains(token))
}

fn parse_assignment(line: &str) -> Option<(&str, &str)> {
    for separator in ["=>", "=", ":"] {
        if let Some((left, right)) = line.split_once(separator) {
            let key = left
                .trim()
                .trim_matches(|c: char| matches!(c, '"' | '\'' | '`'));
            let value = right
                .trim()
                .trim_end_matches(',')
                .trim_end_matches(';')
                .trim();
            if !key.is_empty() && !value.is_empty() {
                return Some((key, value));
            }
        }
    }
    None
}

fn is_secret_like_key(key: &str) -> bool {
    let normalized = key.to_lowercase();
    let indicators = [
        "password",
        "passwd",
        "pwd",
        "secret",
        "token",
        "api_key",
        "apikey",
        "access_key",
        "secret_key",
        "private_key",
        "client_secret",
        "auth_token",
        "bearer",
        "github_token",
    ];

    indicators.iter().any(|needle| normalized.contains(needle))
}

fn is_quoted_or_literal_secret_value(value: &str) -> bool {
    let trimmed = value.trim();
    let quoted = (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        || (trimmed.starts_with('`') && trimmed.ends_with('`'));
    let bare_literal = !trimmed.contains(' ') && !trimmed.contains('(') && !trimmed.contains('{');

    (quoted || bare_literal) && trimmed.len() >= 4
}

fn find_todo_line(content: &str) -> Option<u32> {
    content.lines().enumerate().find_map(|(index, line)| {
        let trimmed = line.trim();
        if (trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with('*')
            || trimmed.starts_with("/*"))
            && contains_word(trimmed, "TODO")
        {
            return Some(line_number_from_index(index));
        }
        None
    })
}

fn find_code_keyword_line(content: &str, keyword: &str) -> Option<u32> {
    let kw = keyword.to_lowercase();
    for (index, line) in content.lines().enumerate() {
        let lower = line.to_lowercase();
        let trimmed = lower.trim();
        if trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with('*')
            || trimmed.starts_with("/*")
        {
            continue;
        }
        if lower
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .any(|token| token == kw)
        {
            return Some(line_number_from_index(index));
        }
    }
    None
}

fn find_secret_assignment_line(content: &str) -> Option<u32> {
    content.lines().enumerate().find_map(|(index, line)| {
        if likely_doc_or_demo_line(line) {
            return None;
        }
        let normalized = normalized_code_line(line)?;
        let (key, value) = parse_assignment(&normalized)?;
        if is_secret_like_key(key)
            && is_quoted_or_literal_secret_value(value)
            && !looks_like_placeholder_value(value)
        {
            Some(line_number_from_index(index))
        } else {
            None
        }
    })
}

fn find_hardcoded_credential_line(content: &str) -> Option<u32> {
    content.lines().enumerate().find_map(|(index, line)| {
        if likely_doc_or_demo_line(line) {
            return None;
        }
        let normalized = normalized_code_line(line)?;
        let (key, value) = parse_assignment(&normalized)?;
        if !is_secret_like_key(key) || !is_quoted_or_literal_secret_value(value) {
            return None;
        }

        let normalized_value = value.trim_matches(|c: char| matches!(c, '"' | '\'' | '`' | ' '));
        if normalized_value.len() >= 8
            && !looks_like_placeholder_value(value)
            && normalized_value.chars().any(|c| c.is_ascii_digit())
        {
            Some(line_number_from_index(index))
        } else {
            None
        }
    })
}

fn content_matches_suspicious_patterns(content: &str, patterns: &[String]) -> bool {
    let lower = content.to_lowercase();
    patterns
        .iter()
        .any(|pat| !pat.trim().is_empty() && lower.contains(&pat.to_lowercase()))
}

fn normalize_severity(severity: &str) -> &str {
    match severity {
        "critical" | "high" | "medium" | "low" | "info" => severity,
        "warn" | "warning" => "medium",
        _ => "info",
    }
}

fn diagnostic(level: &str, source: &str, code: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        level: level.to_string(),
        source: source.to_string(),
        code: code.to_string(),
        message: message.into(),
        path: None,
    }
}

fn diagnostic_with_path(
    level: &str,
    source: &str,
    code: &str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> Diagnostic {
    Diagnostic {
        level: level.to_string(),
        source: source.to_string(),
        code: code.to_string(),
        message: message.into(),
        path: Some(path.into()),
    }
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "critical" => 5,
        "high" => 4,
        "medium" => 3,
        "low" => 2,
        _ => 1,
    }
}

fn run_local_rules(content: &str, profile: &EffectiveProfile) -> Vec<Finding> {
    let mut findings = Vec::new();
    if profile.rules_enabled("todo-comment") {
        if let Some(line) = find_todo_line(content) {
            findings.push(Finding {
                rule: "todo-comment".to_string(),
                severity: "info".to_string(),
                message: "TODO marker found in comment".to_string(),
                category: Some("maintainability".to_string()),
                line: Some(line),
                source: Some("local-rules".to_string()),
                origin: "deterministic".to_string(),
            });
        }
    }

    // SECURITY: Use word-boundary matching to reduce false positives on "unsafe" in prose
    if profile.rules_enabled("unsafe-usage") {
        if let Some(line) = find_code_keyword_line(content, "unsafe") {
            findings.push(Finding {
                rule: "unsafe-usage".to_string(),
                severity: "high".to_string(),
                message: "Possible unsafe keyword usage detected (code context)".to_string(),
                category: Some("safety".to_string()),
                line: Some(line),
                source: Some("local-rules".to_string()),
                origin: "deterministic".to_string(),
            });
        }
    }

    if profile.rules_enabled("possible-secret") {
        if let Some(line) = find_secret_assignment_line(content) {
            findings.push(Finding {
                rule: "possible-secret".to_string(),
                severity: "high".to_string(),
                message: "Possible secret or credential assignment detected".to_string(),
                category: Some("security".to_string()),
                line: Some(line),
                source: Some("local-rules".to_string()),
                origin: "deterministic".to_string(),
            });
        }
    }

    if profile.rules_enabled("large-file") && content.len() > 20 * 1024 {
        findings.push(Finding {
            rule: "large-file".to_string(),
            severity: "low".to_string(),
            message: "Large file snippet may hide complexity/quality issues".to_string(),
            category: Some("maintainability".to_string()),
            line: None,
            source: Some("local-rules".to_string()),
            origin: "deterministic".to_string(),
        });
    }

    if profile.rules_enabled("hardcoded-value") {
        if let Some(line) = find_hardcoded_credential_line(content) {
            findings.push(Finding {
                rule: "hardcoded-value".to_string(),
                severity: "high".to_string(),
                message: "Possible hardcoded credential or key assignment detected".to_string(),
                category: Some("security".to_string()),
                line: Some(line),
                source: Some("local-rules".to_string()),
                origin: "deterministic".to_string(),
            });
        }
    }

    findings
}

fn severity_threshold_met(finding: &Finding, threshold: &str) -> bool {
    let t = severity_rank(normalize_severity(threshold));
    let s = severity_rank(normalize_severity(&finding.severity));
    s >= t
}

impl EffectiveProfile {
    fn rules_enabled(&self, rule: &str) -> bool {
        self.enabled_rules.iter().any(|r| r == rule)
    }
}

fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = Glob::new(pattern).context("invalid glob pattern")?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|err| anyhow!("build glob set: {err}"))
}

fn build_selectors(profile: &EffectiveProfile) -> Result<(GlobSet, GlobSet)> {
    let include = if profile.include_globs.is_empty() {
        vec!["**/*".to_string()]
    } else {
        profile.include_globs.clone()
    };
    let include_set = build_globset(&include)?;
    let exclude_set = build_globset(&profile.exclude_globs)?;
    Ok((include_set, exclude_set))
}

fn is_included(path: &Path, include: &GlobSet, exclude: &GlobSet) -> bool {
    let rel = path.to_string_lossy();
    include.is_match(rel.as_ref()) && !exclude.is_match(rel.as_ref())
}

fn profile_path_candidates(root: &Path, name: &str) -> Vec<PathBuf> {
    // SECURITY: Reject profile names containing path separators or traversal sequences
    // to prevent escaping the profile directory
    if name.contains('/') || name.contains('\\') || name.contains("..") || name.is_empty() {
        return vec![];
    }
    [("toml"), ("yaml"), ("yml")]
        .into_iter()
        .map(|ext| root.join(format!("{name}.{ext}")))
        .collect()
}

// SECURITY: Maximum profile file size to prevent resource exhaustion from malicious repos
const MAX_PROFILE_FILE_BYTES: u64 = 256 * 1024;
// SECURITY: Maximum inheritance depth to prevent stack exhaustion
const MAX_INHERITANCE_DEPTH: usize = 8;

fn load_profile_file(path: &Path) -> Result<RuntimeProfile> {
    // SECURITY: Check file size before reading to prevent memory abuse
    let metadata =
        fs::metadata(path).with_context(|| format!("stat profile {}", path.display()))?;
    if metadata.len() > MAX_PROFILE_FILE_BYTES {
        return Err(anyhow!(
            "profile file too large ({} bytes, limit {}): {}",
            metadata.len(),
            MAX_PROFILE_FILE_BYTES,
            path.display()
        ));
    }
    // SECURITY: Reject symlinks in profile files to prevent following links outside repo
    if metadata.file_type().is_symlink()
        || fs::symlink_metadata(path)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    {
        return Err(anyhow!(
            "profile file is a symlink (rejected for safety): {}",
            path.display()
        ));
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext == "toml")
    {
        let profile = toml::from_str::<RuntimeProfile>(&raw)
            .with_context(|| format!("parse toml profile {}", path.display()))?;
        return Ok(profile);
    }

    let profile = serde_yaml::from_str::<RuntimeProfile>(&raw)
        .with_context(|| format!("parse yaml profile {}", path.display()))?;
    Ok(profile)
}

pub fn validate_profile_config(
    profile: &RuntimeProfile,
    source_label: &str,
) -> Result<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();

    if profile.name.trim().is_empty() {
        return Err(anyhow!(
            "profile {source_label} is missing required 'name' field"
        ));
    }
    if profile.limits.max_files == 0 {
        return Err(anyhow!(
            "profile {source_label} has invalid limits.max_files = 0"
        ));
    }
    if profile.limits.max_total_bytes == 0 {
        return Err(anyhow!(
            "profile {source_label} has invalid limits.max_total_bytes = 0"
        ));
    }
    if profile.limits.max_file_read_bytes == 0 {
        return Err(anyhow!(
            "profile {source_label} has invalid limits.max_file_read_bytes = 0"
        ));
    }

    let normalized_min = normalize_severity(&profile.rules.min_severity).to_string();
    if normalized_min != profile.rules.min_severity && !profile.rules.min_severity.is_empty() {
        diagnostics.push(diagnostic(
            "warning",
            "profile",
            "normalized-min-severity",
            format!(
                "profile {} uses non-standard min_severity '{}'; normalized to '{}'",
                profile.name, profile.rules.min_severity, normalized_min
            ),
        ));
    }

    if profile.ai.enabled && matches!(profile.ai.provider, provider::ProviderConfig::Disabled) {
        diagnostics.push(diagnostic(
            "warning",
            "provider",
            "ai-enabled-provider-disabled",
            format!(
                "profile {} enables AI but provider is disabled; provider analysis will be skipped",
                profile.name
            ),
        ));
    }

    if profile.worker.enabled && profile.rules.suspicious_patterns.is_empty() {
        diagnostics.push(diagnostic(
            "warning",
            "worker",
            "worker-without-patterns",
            format!(
                "profile {} enables worker analysis without suspicious patterns; worker may never run",
                profile.name
            ),
        ));
    }

    if profile.ai.enabled
        && profile.rules.suspicious_patterns.is_empty()
        && profile.ai.suspicious_patterns.is_empty()
    {
        diagnostics.push(diagnostic(
            "warning",
            "provider",
            "provider-without-patterns",
            format!(
                "profile {} enables provider analysis without suspicious patterns; provider may never run",
                profile.name
            ),
        ));
    }

    if profile.ai.enabled
        && profile.rules.suspicious_patterns.len() < 2
        && profile.ai.suspicious_patterns.is_empty()
    {
        diagnostics.push(diagnostic(
            "info",
            "provider",
            "narrow-provider-patterns",
            format!(
                "profile {} uses a very narrow suspicious pattern set; provider coverage may miss relevant files",
                profile.name
            ),
        ));
    }

    Ok(diagnostics)
}

pub fn effective_profile(
    name_or_path: &str,
    root: &Path,
) -> Result<(EffectiveProfile, String, Vec<Diagnostic>)> {
    let (profile, source) = load_profile(name_or_path, root)?;
    let source_desc = match &source {
        ProfileSource::Builtin => "builtin profile".to_string(),
        ProfileSource::Loaded(path) => path.to_string_lossy().to_string(),
    };
    let diagnostics = validate_profile_config(&profile, &source_desc)?;
    let source_label = match source {
        ProfileSource::Builtin => "builtin".to_string(),
        ProfileSource::Loaded(path) => format!("file:{}", path.display()),
    };
    Ok((profile.into_effective(), source_label, diagnostics))
}

fn resolve_profile_from_file(
    path: &Path,
    root: &Path,
    seen: &mut HashSet<PathBuf>,
    seen_names: &mut HashSet<String>,
) -> Result<(RuntimeProfile, ProfileSource)> {
    // SECURITY: Enforce maximum inheritance depth to prevent stack exhaustion
    if seen.len() >= MAX_INHERITANCE_DEPTH {
        return Err(anyhow!(
            "profile inheritance too deep (limit {})",
            MAX_INHERITANCE_DEPTH
        ));
    }

    let canonical = path
        .canonicalize()
        .with_context(|| format!("canonicalize profile {}", path.display()))?;

    // SECURITY: Validate that the canonicalized profile path stays within the scan root
    let root_canonical = root
        .canonicalize()
        .with_context(|| format!("canonicalize root {}", root.display()))?;
    if !canonical.starts_with(&root_canonical) {
        return Err(anyhow!(
            "profile path escapes scan root boundary: {}",
            canonical.display()
        ));
    }

    if seen.contains(&canonical) || seen_names.contains(&canonical.to_string_lossy().to_string()) {
        return Err(anyhow!(
            "circular profile inheritance involving '{}'",
            path.display()
        ));
    }

    let manifest = load_profile_file(&canonical)?;
    let mut profile = manifest;

    if profile.name.trim().is_empty() {
        return Err(anyhow!(
            "profile {} is missing required 'name' field",
            canonical.display()
        ));
    }
    // SECURITY: Validate profile name constraints
    let name_trimmed = profile.name.trim();
    if name_trimmed.len() > 128 {
        return Err(anyhow!(
            "profile name too long (limit 128 chars): {}",
            canonical.display()
        ));
    }
    if name_trimmed.contains('/') || name_trimmed.contains('\\') || name_trimmed.contains("..") {
        return Err(anyhow!(
            "profile name contains invalid path characters: '{}'",
            name_trimmed
        ));
    }

    seen.insert(canonical.clone());
    seen_names.insert(profile.name.clone());

    if let Some(inherits) = profile.inherits.clone() {
        // SECURITY: Validate inherited profile name to prevent path traversal
        let inherits_trimmed = inherits.trim();
        if inherits_trimmed.contains('/')
            || inherits_trimmed.contains('\\')
            || inherits_trimmed.contains("..")
        {
            return Err(anyhow!(
                "inherited profile name contains invalid characters: '{}'",
                inherits_trimmed
            ));
        }

        let base_profile = if let Some(base_path) =
            profile_path_candidates(&root.join(".dn/profiles"), inherits_trimmed)
                .into_iter()
                .find(|candidate| candidate.exists())
        {
            let (base, _) = resolve_profile_from_file(&base_path, root, seen, seen_names)?;
            base
        } else if let Some(profile) = builtin_profile(inherits_trimmed) {
            profile
        } else {
            return Err(anyhow!("unknown inherited profile '{}'", inherits_trimmed));
        };

        profile = profile.merge(&base_profile);
    }
    Ok((profile, ProfileSource::Loaded(canonical)))
}

pub fn available_profiles(root: &Path) -> Vec<String> {
    let mut names = vec![
        "quick",
        "security",
        "architecture",
        "deep",
        "performance",
        "maintainability",
        "ai-generated-code-review",
        "legacy-modernization",
        "pre-merge",
        "strict",
        "educational",
        "production-readiness",
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<_>>();

    let profile_dir = root.join(".dn/profiles");
    if let Ok(entries) = fs::read_dir(profile_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if ext == "toml" || ext == "yml" || ext == "yaml" {
                if let Some(name) = path.file_stem().and_then(|name| name.to_str()) {
                    names.push(name.to_string());
                }
            }
        }
    }

    names.sort_unstable();
    names.dedup();
    names
}

pub fn load_profile(name_or_path: &str, root: &Path) -> Result<(RuntimeProfile, ProfileSource)> {
    if Path::new(name_or_path).exists() {
        let mut seen = HashSet::new();
        let mut seen_names = HashSet::new();
        return resolve_profile_from_file(
            Path::new(name_or_path),
            root,
            &mut seen,
            &mut seen_names,
        );
    }

    let profile_dir = root.join(".dn/profiles");
    if let Some(file) = profile_path_candidates(&profile_dir, name_or_path)
        .into_iter()
        .find(|file| file.exists())
    {
        let mut seen = HashSet::new();
        let mut seen_names = HashSet::new();
        return resolve_profile_from_file(&file, root, &mut seen, &mut seen_names);
    }

    if let Some(profile) = builtin_profile(name_or_path) {
        return Ok((profile, ProfileSource::Builtin));
    }

    Err(anyhow!("unknown profile '{}'", name_or_path))
}

pub fn available_profile_entries(root: &Path) -> Vec<(String, String)> {
    let mut entries = vec![
        "quick",
        "security",
        "architecture",
        "deep",
        "performance",
        "maintainability",
        "ai-generated-code-review",
        "legacy-modernization",
        "pre-merge",
        "strict",
        "educational",
        "production-readiness",
    ]
    .into_iter()
    .map(|name| (name.to_string(), "builtin".to_string()))
    .collect::<Vec<_>>();

    let profile_dir = root.join(".dn/profiles");
    if let Ok(entries_iter) = fs::read_dir(profile_dir) {
        for entry in entries_iter.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
            if ext == "toml" || ext == "yml" || ext == "yaml" {
                if let Some(name) = path.file_stem().and_then(|name| name.to_str()) {
                    entries.push((name.to_string(), format!("file:{}", path.display())));
                }
            }
        }
    }

    entries.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    entries.dedup_by(|a, b| a.0 == b.0);
    entries
}

pub fn scan_repository(root: impl AsRef<Path>, options: &ScanOptions) -> Result<ScanOutcome> {
    let start = Instant::now();
    let root = root.as_ref();
    let root_path = root
        .canonicalize()
        .with_context(|| format!("canonicalize {}", root.display()))?;

    let (base_profile, source) =
        load_profile(&options.profile_name, root).context("load profile")?;
    let source_desc = match &source {
        ProfileSource::Builtin => "builtin profile".to_string(),
        ProfileSource::Loaded(path) => path.to_string_lossy().to_string(),
    };
    let mut diagnostics = validate_profile_config(&base_profile, &source_desc)?;
    let mut profile = base_profile.into_effective();
    if options.include_hidden {
        profile.include_hidden = true;
    }
    if options.python_worker {
        profile.worker_enabled = true;
    }
    if options.max_files != 0 {
        profile.limits.max_files = options.max_files;
    }
    let profile_source = match source {
        ProfileSource::Builtin => "builtin".to_string(),
        ProfileSource::Loaded(path) => format!("file:{}", path.display()),
    };

    let (include_set, exclude_set) = build_selectors(&profile).context("build selector")?;
    let mut walk = WalkBuilder::new(&root_path);
    walk.hidden(!profile.include_hidden)
        .git_ignore(true)
        .git_global(true)
        .parents(true)
        // SECURITY: Do not follow symlinks to prevent escaping the scan root
        .follow_links(false);

    let mut files = Vec::new();
    let mut total_bytes = 0u64;
    let mut total_files_scanned = 0usize;
    let mut files_discovered = 0usize;
    let mut skipped_large_files = 0usize;
    let mut truncated = false;
    let mut errors = Vec::new();
    let mut worker_mode = "disabled".to_string();

    let mut worker_registry = if profile.worker_enabled {
        let registry = WorkerRegistry::new(profile.worker_timeout_ms, profile.worker_retries);
        if let Some(cfg) = registry.get("python") {
            if cfg.retries > 0 {
                worker_mode = "python".to_string();
            } else {
                worker_mode = "python (single-shot)".to_string();
            }
        }
        Some(registry)
    } else {
        None
    };
    let review_engine = ReviewEngine::from_config(&profile.ai);
    let provider_name = review_engine.provider_name().to_string();

    let mut ai_files_used = 0usize;
    let mut worker_used = false;

    for entry in walk.build() {
        if total_files_scanned >= profile.limits.max_files
            || total_bytes >= profile.limits.max_total_bytes
        {
            truncated = true;
            break;
        }

        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                errors.push(format!("walk error: {err}"));
                diagnostics.push(diagnostic(
                    "error",
                    "scanner",
                    "walk-error",
                    format!("{err}"),
                ));
                continue;
            }
        };

        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                errors.push(format!("metadata {}: {err}", path.display()));
                diagnostics.push(diagnostic_with_path(
                    "error",
                    "io",
                    "metadata-error",
                    path.display().to_string(),
                    format!("{err}"),
                ));
                continue;
            }
        };

        if metadata.is_dir() {
            continue;
        }

        // SECURITY: Skip symlinks and non-regular files to prevent escaping scan root
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            continue;
        }

        let path = path.to_path_buf();
        let relative = match path.strip_prefix(&root_path) {
            Ok(rel) => rel,
            Err(_) => {
                // SECURITY: Path outside scan root boundary — skip it
                continue;
            }
        };

        if !is_included(relative, &include_set, &exclude_set) {
            continue;
        }

        files_discovered += 1;

        if !is_text_file(
            &path,
            &metadata,
            profile.include_binary,
            profile.limits.max_file_read_bytes,
        ) {
            continue;
        }

        let size = metadata.len();
        if size > profile.limits.max_file_size_bytes {
            skipped_large_files += 1;
            continue;
        }

        if total_files_scanned >= profile.limits.max_files {
            truncated = true;
            break;
        }

        if total_bytes.saturating_add(size) > profile.limits.max_total_bytes {
            truncated = true;
            break;
        }

        total_files_scanned += 1;
        total_bytes = total_bytes.saturating_add(size);

        let rel_path = relative.to_string_lossy().to_string();
        let mut findings = Vec::new();
        let language = detect_language(&path);
        let mut integration_notes = Vec::new();

        match read_text_preview(&path, profile.limits.max_file_read_bytes) {
            Ok(content) => {
                findings.extend(run_local_rules(&content, &profile));

                let worker_suspicious =
                    content_matches_suspicious_patterns(&content, &profile.suspicious_patterns);
                let provider_patterns = if profile.ai.suspicious_patterns.is_empty() {
                    &profile.suspicious_patterns
                } else {
                    &profile.ai.suspicious_patterns
                };
                let provider_suspicious =
                    content_matches_suspicious_patterns(&content, provider_patterns);

                if profile.worker_enabled && worker_suspicious {
                    let language_name = language.clone().unwrap_or_else(|| "unknown".to_string());
                    if let Some(registry) = worker_registry.as_mut() {
                        if registry.supports(&language_name) {
                            match registry.analyze(&language_name, &rel_path, &content) {
                                Ok(worker_findings) => {
                                    worker_used = true;
                                    integration_notes.push("worker:python".to_string());
                                    findings.extend(worker_findings);
                                }
                                Err(err) => {
                                    errors.push(format!("worker {}: {err}", rel_path));
                                    diagnostics.push(diagnostic_with_path(
                                        if options.strict_integrations || profile.ai.strict {
                                            "error"
                                        } else {
                                            "warning"
                                        },
                                        "worker",
                                        "worker-analysis-failed",
                                        rel_path.clone(),
                                        err.to_string(),
                                    ));
                                    if options.strict_integrations {
                                        return Err(anyhow!(
                                            "worker analysis failed for {}: {err}",
                                            rel_path
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }

                if profile.ai.enabled
                    && ai_files_used < profile.ai.max_ai_files
                    && provider_suspicious
                {
                    match review_engine.analyze_file_if_enabled(
                        &profile.ai,
                        AiRequest {
                            path: rel_path.clone(),
                            language: language.clone(),
                            content,
                            summary_profile: profile.name.clone(),
                        },
                    ) {
                        Ok(ai_findings) => {
                            integration_notes.push(format!("provider:{provider_name}"));
                            findings.extend(ai_findings);
                            ai_files_used += 1;
                        }
                        Err(err) => {
                            errors.push(format!("ai provider {}: {err}", profile.name));
                            diagnostics.push(diagnostic_with_path(
                                if options.strict_integrations || profile.ai.strict {
                                    "error"
                                } else {
                                    "warning"
                                },
                                "provider",
                                "provider-analysis-failed",
                                rel_path.clone(),
                                err.to_string(),
                            ));
                            if options.strict_integrations || profile.ai.strict {
                                return Err(anyhow!(
                                    "provider analysis failed for {}: {err}",
                                    rel_path
                                ));
                            }
                        }
                    }
                }

                let ai_min = normalize_severity(&profile.ai.min_severity).to_string();
                findings.retain(|finding| {
                    let meets_output = profile.prioritize_rules.contains(&finding.rule)
                        || severity_threshold_met(finding, &profile.severity_threshold);
                    let meets_ai_min = finding.origin != "provider"
                        || severity_rank(&finding.severity) >= severity_rank(&ai_min);
                    meets_output && meets_ai_min
                });

                findings.sort_by(|a, b| {
                    severity_rank(normalize_severity(&b.severity))
                        .cmp(&severity_rank(normalize_severity(&a.severity)))
                });
            }
            Err(err) => {
                errors.push(format!("read {}: {err}", rel_path));
                diagnostics.push(diagnostic_with_path(
                    "error",
                    "io",
                    "read-preview-failed",
                    rel_path.clone(),
                    err.to_string(),
                ));
            }
        }

        let content_preview = if options.include_content || profile.include_content_preview {
            read_text_preview(&path, 1024)
                .ok()
                .map(|preview| preview.replace('\0', "\\0"))
        } else {
            None
        };

        files.push(FileEntry {
            path: rel_path,
            size,
            language,
            findings,
            content_preview,
            integration_notes: if integration_notes.is_empty() {
                None
            } else {
                Some(integration_notes)
            },
        });
    }

    let files_selected = files.len();
    let mut severity = SeverityStats {
        info: 0,
        low: 0,
        medium: 0,
        high: 0,
        critical: 0,
    };
    for file in &files {
        for finding in &file.findings {
            match normalize_severity(&finding.severity) {
                "critical" => severity.critical += 1,
                "high" => severity.high += 1,
                "medium" => severity.medium += 1,
                "low" => severity.low += 1,
                _ => severity.info += 1,
            }
        }
    }

    let total_findings: usize =
        severity.info + severity.low + severity.medium + severity.high + severity.critical;
    let files_skipped = files_discovered.saturating_sub(files_selected);
    let worker_summary = if profile.worker_enabled {
        format!("python:{worker_mode}")
    } else {
        "disabled".to_string()
    };

    let duration_ms = start.elapsed().as_millis();
    let threshold = options.fail_on_severity.clone();
    let threshold_triggered = threshold.as_deref().is_some_and(|level| {
        if level == "none" {
            return false;
        }
        let rank = severity_rank(normalize_severity(level));
        severity.critical >= usize::from(rank <= 5)
            && files
                .iter()
                .flat_map(|file| file.findings.iter())
                .any(|finding| severity_rank(&finding.severity) >= rank)
    });

    let public_files = if options.summary_only {
        Vec::new()
    } else {
        files.clone()
    };

    let report = ScanReport {
        schema_version: "2".to_string(),
        metadata: ReportMetadata {
            root: root_path.to_string_lossy().to_string(),
            profile: profile.name.clone(),
            profile_source: profile_source.clone(),
            command: options.command_name.clone(),
            output_format: options.format.clone(),
            summary_only: options.summary_only,
            duration_ms,
            truncated,
        },
        stats: ReportStats {
            files_discovered,
            files_scanned: total_files_scanned,
            files_selected,
            files_skipped,
            total_files: files_discovered,
            total_bytes,
            skipped_large_files,
            findings_total: total_findings,
            severity_breakdown: severity.clone(),
        },
        integrations: IntegrationSummary {
            worker: IntegrationRuntimeStatus {
                enabled: profile.worker_enabled,
                mode: worker_summary.clone(),
                strict: options.strict_integrations,
                used: worker_used,
                supported_languages: if profile.worker_enabled {
                    Some(vec!["python".to_string()])
                } else {
                    None
                },
                name: None,
                max_ai_files: None,
                files_sent: None,
            },
            provider: IntegrationRuntimeStatus {
                enabled: profile.ai.enabled,
                mode: provider_name.clone(),
                strict: options.strict_integrations || profile.ai.strict,
                used: ai_files_used > 0,
                supported_languages: None,
                name: Some(provider_name.clone()),
                max_ai_files: Some(profile.ai.max_ai_files),
                files_sent: Some(ai_files_used),
            },
        },
        diagnostics: diagnostics.clone(),
        files: public_files,
        summary: format!(
            "Scanned {files_discovered} files ({total_files_scanned} scanned), {total_findings} findings in {duration_ms}ms",
        ),
    };

    Ok(ScanOutcome {
        root: report.metadata.root.clone(),
        profile: report.metadata.profile.clone(),
        provider: format!("{}@{}", provider_name, profile_source),
        worker: worker_summary,
        profile_source,
        files_discovered,
        files_scanned: total_files_scanned,
        files_selected,
        files_skipped,
        total_files: files_discovered,
        total_bytes,
        skipped_large_files,
        truncated,
        errors,
        files,
        severity_breakdown: severity,
        duration_ms,
        summary: report.summary.clone(),
        report,
        exit_evaluation: ExitEvaluation {
            threshold,
            threshold_triggered,
        },
    })
}

impl EffectiveProfile {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_tmp_root(prefix: &str) -> PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let root = env::temp_dir().join(format!("dn_runtime_{prefix}_{millis}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".dn/profiles")).unwrap();
        root
    }

    fn write_file(path: &Path, name: &str, body: &str) {
        let file = path.join(name);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = File::create(file).unwrap();
        f.write_all(body.as_bytes()).unwrap();
    }

    #[test]
    fn builtin_profile_exists() {
        let (profile, _) = load_profile("quick", Path::new(".")).unwrap();
        assert_eq!(profile.name, "quick");
        assert!(!profile.worker.enabled || profile.limits.max_files > 0);
    }

    #[test]
    fn can_load_local_profile() {
        let repo = make_tmp_root("local");
        let content = r#"
name = "custom"
inherits = "security"
include_hidden = true
"#;
        fs::write(repo.join(".dn/profiles/custom.toml"), content).unwrap();

        let (profile, source) = load_profile("custom", &repo).unwrap();
        assert_eq!(profile.name, "custom");
        assert!(profile.worker.enabled);
        assert!(profile.include_hidden.unwrap_or(false));
        match source {
            ProfileSource::Loaded(path) => assert!(path.ends_with("custom.toml")),
            _ => panic!("expected file profile"),
        }
        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn invalid_inherited_profile_reports_error() {
        let repo = make_tmp_root("bad-parent");
        fs::write(
            repo.join(".dn/profiles/child.toml"),
            r#"
name = "child"
inherits = "missing_parent"
"#,
        )
        .unwrap();

        let err = load_profile("child", &repo).unwrap_err();
        assert!(err.to_string().contains("unknown inherited profile"));
        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn detects_circular_profile_inheritance() {
        let repo = make_tmp_root("circular");
        fs::write(
            repo.join(".dn/profiles/alpha.toml"),
            r#"
name = "alpha"
inherits = "beta"
"#,
        )
        .unwrap();
        fs::write(
            repo.join(".dn/profiles/beta.toml"),
            r#"
name = "beta"
inherits = "alpha"
"#,
        )
        .unwrap();

        let err = load_profile("alpha", &repo).unwrap_err();
        assert!(err.to_string().contains("circular profile inheritance"));
        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn can_fail_unknown_profile() {
        let repo = make_tmp_root("unknown");
        let err = load_profile("definitely_missing", &repo).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("unknown profile"));
        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn rejects_malformed_local_profile() {
        let repo = make_tmp_root("malformed");
        fs::write(repo.join(".dn/profiles/bad.toml"), "name = bad").unwrap();
        let err = load_profile("bad", &repo).unwrap_err();
        assert!(err.to_string().contains("parse toml profile"));
        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn rejects_unknown_field_in_local_profile() {
        let repo = make_tmp_root("unknown-field");
        fs::write(
            repo.join(".dn/profiles/bad-fields.toml"),
            r#"
name = "bad-fields"
unknown_top_level_field = true
"#,
        )
        .unwrap();

        let err = load_profile("bad-fields", &repo).unwrap_err();
        let message = err.to_string().to_lowercase();
        assert!(!message.is_empty());
        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn hidden_files_follow_hidden_flag() {
        let repo = make_tmp_root("hidden");
        write_file(&repo, "visible.txt", "todo");
        write_file(&repo, ".env", "secret");
        write_file(&repo, ".hiddendir/secret.txt", "password");

        let default = scan_repository(
            &repo,
            &ScanOptions {
                profile_name: "quick".into(),
                ..ScanOptions::default()
            },
        )
        .unwrap();
        assert_eq!(default.files_discovered, 1);
        assert!(!default
            .files
            .iter()
            .any(|entry| entry.path.starts_with(".env")));

        let hidden = scan_repository(
            &repo,
            &ScanOptions {
                profile_name: "quick".into(),
                include_hidden: true,
                ..ScanOptions::default()
            },
        )
        .unwrap();
        assert!(hidden.files_discovered >= 2);
        assert!(hidden.files.iter().any(|entry| entry.path == ".env"));
        assert!(hidden
            .files
            .iter()
            .any(|entry| entry.path == ".hiddendir/secret.txt"));
        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn report_counts_reflect_skips_and_scan_counts() {
        let repo = make_tmp_root("counts");
        write_file(&repo, "good.txt", "todo");
        write_file(&repo, "big.txt", &"a".repeat(2048));
        fs::write(
            repo.join(".dn/profiles/counts.toml"),
            r#"
name = "counts"
inherits = "quick"
[limits]
max_file_size_bytes = 1024
"#,
        )
        .unwrap();

        let report = scan_repository(
            &repo,
            &ScanOptions {
                profile_name: "counts".into(),
                ..ScanOptions::default()
            },
        )
        .unwrap();
        assert_eq!(report.files_discovered, 2);
        assert_eq!(report.files_scanned, 1);
        assert_eq!(report.files_skipped, 1);
        assert_eq!(report.skipped_large_files, 1);
        assert!(report.total_files >= 2);
        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn scan_repository_runs_with_profile() {
        let root = make_tmp_root("scan");
        write_file(&root, "a.py", "print('hello')\\n");
        let options = ScanOptions {
            profile_name: "quick".to_string(),
            include_content: true,
            ..ScanOptions::default()
        };
        let report = scan_repository(&root, &options).unwrap();
        assert_eq!(report.profile, "quick");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn python_worker_profile_flag_enables_worker_analysis() {
        let root = make_tmp_root("worker");
        write_file(&root, "eval.py", "print(eval('2+2'))\n");
        fs::write(
            root.join(".dn/profiles/worker.toml"),
            r#"
name = "worker"
[rules]
deterministic_rules = []
suspicious_patterns = ["eval("]
[worker]
enabled = true
[file_selection]
include_binary = true
"#,
        )
        .unwrap();

        let report = scan_repository(
            &root,
            &ScanOptions {
                profile_name: "worker".into(),
                python_worker: true,
                ..ScanOptions::default()
            },
        )
        .unwrap();

        assert_eq!(report.files.len(), 1);
        assert!(report.worker.starts_with("python:"));
        assert!(
            report.errors.is_empty(),
            "unexpected scan errors: {:?}",
            report.errors
        );
        assert!(!report.files[0].findings.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn summary_only_report_empties_public_files() {
        let root = make_tmp_root("summary-only");
        write_file(&root, "secret.txt", "password = \"hello\"\n");

        let report = scan_repository(
            &root,
            &ScanOptions {
                profile_name: "quick".into(),
                summary_only: true,
                format: OutputFormat::Json,
                ..ScanOptions::default()
            },
        )
        .unwrap();

        assert_eq!(report.report.schema_version, "2");
        assert!(report.report.metadata.summary_only);
        assert_eq!(report.report.files.len(), 0);
        assert!(!report.files.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn invalid_zero_limit_profile_is_rejected() {
        let repo = make_tmp_root("zero-limit");
        fs::write(
            repo.join(".dn/profiles/zero.toml"),
            r#"
name = "zero"
[limits]
max_files = 0
"#,
        )
        .unwrap();

        let err = effective_profile("zero", &repo).unwrap_err();
        assert!(err.to_string().contains("invalid limits.max_files = 0"));
        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn diagnostics_include_ai_disabled_warning() {
        let profile = RuntimeProfile {
            name: "diag".to_string(),
            ai: ProfileAiConfig {
                enabled: true,
                provider: provider::ProviderConfig::Disabled,
                ..ProfileAiConfig::default()
            },
            ..RuntimeProfile::default()
        };

        let diagnostics = validate_profile_config(&profile, "inline").unwrap();
        assert!(diagnostics
            .iter()
            .any(|diag| diag.code == "ai-enabled-provider-disabled"));
    }

    #[test]
    fn rejects_non_local_provider_endpoint() {
        let root = make_tmp_root("remote-provider");
        write_file(&root, "suspicious.py", "eval('2+2')\n");
        fs::write(
            root.join(".dn/profiles/remote-provider.toml"),
            r#"
name = "remote-provider"
[rules]
deterministic_rules = []
suspicious_patterns = ["eval("]
[ai]
enabled = true
max_ai_files = 10
provider = { type = "ollama", base_url = "https://example.com", model = "demo" }
"#,
        )
        .unwrap();

        let report = scan_repository(
            &root,
            &ScanOptions {
                profile_name: "remote-provider".into(),
                ..ScanOptions::default()
            },
        )
        .unwrap();

        assert!(report
            .report
            .diagnostics
            .iter()
            .any(|diag| diag.code == "provider-analysis-failed"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn secret_rule_ignores_comment_examples_and_env_indirection() {
        assert!(find_secret_assignment_line(
            r#"
            // password = "example-secret"
            api_key = "${API_KEY}"
            client_secret: "changeme"
            "#
        )
        .is_none());
    }

    #[test]
    fn secret_rule_detects_json_yaml_and_single_quote_assignments() {
        assert!(find_secret_assignment_line(
            r#"
            const cfg = { "client_secret": "prod-Secret-9" };
            token: 'real-token-42'
            "#
        )
        .is_some());
    }

    #[test]
    fn hardcoded_rule_requires_more_than_placeholder_tokens() {
        assert!(find_hardcoded_credential_line("token = \"test-token\"").is_none());
        assert!(
            find_hardcoded_credential_line("github_token = \"ghp_1234567890abcdef\"").is_some()
        );
    }
}
