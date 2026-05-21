pub mod provider;
pub mod rules;
pub mod worker;

use crate::provider::{AiRequest, ProfileAiConfig, ReviewEngine};
use crate::worker::registry::WorkerRegistry;
use anyhow::{anyhow, Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub use dn_ipc::{WorkerRequest, WorkerResponse};

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
pub struct ReportMetadata {
    pub root: String,
    pub profile: String,
    pub profile_source: String,
    pub command: String,
    pub duration_ms: u128,
    pub truncated: bool,
    pub summary_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationStatus {
    pub enabled: bool,
    pub mode: String,
    pub used: bool,
    pub strict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integrations {
    pub worker: IntegrationStatus,
    pub provider: IntegrationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStats {
    pub findings_total: usize,
    pub files_discovered: usize,
    pub files_scanned: usize,
    pub files_selected: usize,
    pub files_skipped: usize,
    pub severity_breakdown: SeverityStats,
}

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
    pub summary_only: bool,
    #[serde(default)]
    pub fast: bool,
    #[serde(default)]
    pub format: OutputFormat,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            profile_name: "quick".to_string(),
            include_hidden: false,
            include_content: false,
            python_worker: false,
            max_files: 10_000,
            summary_only: false,
            fast: false,
            format: OutputFormat::Text,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    size: u64,
    modified: u64,
    content_hash: String,
    findings: Vec<Finding>,
    content_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ScanCache {
    #[serde(default)]
    version: u32,
    files: std::collections::HashMap<String, CacheEntry>,
}

const SCAN_CACHE_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub severity: String,
    pub rule: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_preview: Option<String>,
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
pub struct ScanReport {
    pub schema_version: String,
    pub metadata: ReportMetadata,
    pub integrations: Integrations,
    pub stats: ScanStats,
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
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug)]
pub enum ProfileSource {
    Builtin,
    Loaded(PathBuf),
}

pub fn registered_rule_names() -> Vec<&'static str> {
    rules::registered_rule_names()
}

pub fn rule_specs() -> &'static [rules::RuleSpec] {
    rules::rule_specs()
}

pub fn apply_safe_fixes(content: &str, fixes: &[rules::RuleFix]) -> String {
    rules::apply_safe_fixes(content, fixes)
}

pub fn available_profile_entries(root: &Path) -> Vec<(String, String)> {
    let profile_dir = root.join(".dn/profiles");
    available_profiles(root)
        .into_iter()
        .map(|name| {
            let source = profile_path_candidates(&profile_dir, &name)
                .into_iter()
                .find(|path| path.exists())
                .map(|path| format!("file:{}", path.display()))
                .unwrap_or_else(|| "builtin".to_string());
            (name, source)
        })
        .collect()
}

pub fn effective_profile(
    name_or_path: &str,
    root: &Path,
) -> Result<(EffectiveProfile, String, Vec<Diagnostic>)> {
    let (profile, source) = load_profile(name_or_path, root)?;
    let source_label = match source {
        ProfileSource::Builtin => "builtin".to_string(),
        ProfileSource::Loaded(path) => format!("file:{}", path.display()),
    };
    Ok((profile.to_effective(), source_label, Vec::new()))
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
                include_hidden: if self.file_selection.include_hidden
                    || self.include_hidden.unwrap_or(false)
                {
                    true
                } else {
                    parent.file_selection.include_hidden
                },
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
                    self.ai.clone()
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

    fn to_effective(&self) -> EffectiveProfile {
        EffectiveProfile {
            name: self.name.clone(),
            description: self.description.clone(),
            enabled_rules: self.rules.deterministic_rules.clone(),
            suspicious_patterns: self.rules.suspicious_patterns.clone(),
            prioritize_rules: self.rules.prioritize.clone(),
            min_severity: self.rules.min_severity.clone(),
            include_hidden: self
                .include_hidden
                .unwrap_or(self.file_selection.include_hidden),
            include_globs: self.file_selection.include_globs.clone(),
            exclude_globs: self.file_selection.exclude_globs.clone(),
            include_binary: self.file_selection.include_binary,
            limits: self.limits.clone(),
            worker_enabled: self.worker.enabled,
            worker_timeout_ms: self.worker.timeout_ms,
            worker_retries: self.worker.retries,
            ai: self.ai.clone(),
            include_content_preview: self.output.include_content_preview,
            severity_threshold: self.output.severity_threshold.clone(),
        }
    }
}

fn is_path_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

fn load_ignore_rules(root: &Path) -> Vec<String> {
    let path = root.join(".dn/ignore");
    fs::read_to_string(path)
        .ok()
        .map(|raw| {
            raw.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

fn is_ignored_by_rules(path: &Path, rules: &[String]) -> bool {
    let mut candidates = Vec::new();
    let text = path.to_string_lossy().replace('\\', "/");
    candidates.push(text.clone());
    if !text.starts_with('/') {
        candidates.push(format!("/{text}"));
    }

    rules.iter().any(|rule| {
        let glob = Glob::new(rule).or_else(|_| Glob::new(&format!("**/{rule}")));
        match glob {
            Ok(glob) => {
                let matcher = glob.compile_matcher();
                candidates.iter().any(|c| matcher.is_match(c))
            }
            Err(_) => false,
        }
    })
}

fn cache_path(root: &Path) -> PathBuf {
    root.join(".dn-cache")
}

fn load_cache(root: &Path) -> ScanCache {
    let mut cache: ScanCache = fs::read_to_string(cache_path(root))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    if cache.version != SCAN_CACHE_VERSION {
        cache = ScanCache {
            version: SCAN_CACHE_VERSION,
            ..Default::default()
        };
    }
    cache
}

fn save_cache(root: &Path, cache: &ScanCache) {
    if let Some(parent) = cache_path(root).parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut cache = cache.clone();
    cache.version = SCAN_CACHE_VERSION;
    if let Ok(raw) = serde_json::to_string(&cache) {
        let _ = fs::write(cache_path(root), raw);
    }
}

fn content_cache_hash(content: &str) -> String {
    let bytes = content.as_bytes();
    let slice = if bytes.len() > 4096 {
        &bytes[..4096]
    } else {
        bytes
    };
    let mut hasher = Sha256::new();
    hasher.update(slice);
    format!("{:x}", hasher.finalize())
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
            profile
                .rules
                .deterministic_rules
                .push("possible-secret".to_string());
            profile.rules.suspicious_patterns.push("token".to_string());
            profile.ai.enabled = true;
            profile.ai.provider = provider::ProviderConfig::Mock {
                message: "No security finding by AI in this file snippet".to_string(),
            };
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
                ..Default::default()
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
                suspicious_patterns: vec!["TODO".to_string(), "FIXME".to_string()],
                prioritize: vec!["todo-comment".to_string()],
                min_severity: "info".to_string(),
            },
            ..base
        }),
        "kernel-c" => Some(RuntimeProfile {
            name: name.to_string(),
            description: "Linux kernel C review profile".to_string(),
            limits: ScanLimits {
                max_file_size_bytes: 2 * 1024 * 1024,
                max_file_read_bytes: 64 * 1024,
                max_total_bytes: 250 * 1024 * 1024,
                max_files: 25_000,
            },
            worker: WorkerProfileConfig {
                enabled: true,
                timeout_ms: 15_000,
                retries: 1,
            },
            file_selection: FileSelectionConfig {
                include_hidden: false,
                include_globs: vec![
                    "drivers/**".to_string(),
                    "fs/**".to_string(),
                    "include/**".to_string(),
                    "ipc/**".to_string(),
                    "kernel/**".to_string(),
                    "mm/**".to_string(),
                    "net/**".to_string(),
                ],
                exclude_globs: vec![
                    ".git/**".to_string(),
                    "Documentation/**".to_string(),
                    "samples/**".to_string(),
                    "tools/**".to_string(),
                ],
                include_binary: false,
            },
            rules: AnalyzerConfig {
                deterministic_rules: vec!["unsafe-usage".to_string(), "kernel-style".to_string()],
                suspicious_patterns: vec![
                    "BUG_ON".to_string(),
                    "goto out".to_string(),
                    "printk(".to_string(),
                    "spin_lock(".to_string(),
                    "mutex_lock(".to_string(),
                    "rcu_read_lock(".to_string(),
                    "__init".to_string(),
                    "__exit".to_string(),
                ],
                prioritize: vec![
                    "goto-out-without-cleanup".to_string(),
                    "return-without-unlock".to_string(),
                    "null-deref-before-check".to_string(),
                    "missing-__init-__exit".to_string(),
                    "RCU-missing-annotation".to_string(),
                    "sleeping-in-atomic".to_string(),
                    "BUG_ON-usage".to_string(),
                    "printk-without-level".to_string(),
                ],
                min_severity: "medium".to_string(),
            },
            ..base
        }),
        "ai-generated-code-review" => {
            let mut p = base;
            p.name = name.to_string();
            p.description = "AI-generated code review profile".to_string();
            p.ai.enabled = true;
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
                ..Default::default()
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
                ..Default::default()
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
                    "unsafe-usage".to_string(),
                ],
                suspicious_patterns: vec![
                    "TODO".to_string(),
                    "FIXME".to_string(),
                    "api_key".to_string(),
                ],
                prioritize: vec!["possible-secret".to_string()],
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
        Some("java") => Some("java".to_string()),
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

fn normalize_severity(severity: &str) -> &str {
    match severity {
        "critical" | "high" | "medium" | "low" | "info" => severity,
        "warn" | "warning" => "medium",
        _ => "info",
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
    let lower = content.to_lowercase();
    let is_kernel_profile = profile.name == "kernel-c";

    if profile.rules_enabled("todo-comment") && content.contains("TODO") && !is_kernel_profile {
        findings.push(Finding {
            severity: "info".to_string(),
            rule: "todo-comment".to_string(),
            message: "TODO marker found in file content".to_string(),
            category: Some("maintainability".to_string()),
            line: None,
            source: Some("local-rules".to_string()),
        });
    }

    if profile.rules_enabled("unsafe-usage") && content.contains("unsafe") {
        findings.push(Finding {
            severity: "high".to_string(),
            rule: "unsafe-usage".to_string(),
            message: "Possible unsafe usage detected".to_string(),
            category: Some("safety".to_string()),
            line: None,
            source: Some("local-rules".to_string()),
        });
    }

    if profile.rules_enabled("possible-secret")
        && (lower.contains("password") || lower.contains("secret"))
        && !is_kernel_profile
    {
        findings.push(Finding {
            severity: "high".to_string(),
            rule: "possible-secret".to_string(),
            message: "Possible secret or sensitive value text detected".to_string(),
            category: Some("security".to_string()),
            line: None,
            source: Some("local-rules".to_string()),
        });
    }

    if profile.rules_enabled("large-file") && content.len() > 20 * 1024 {
        findings.push(Finding {
            severity: "low".to_string(),
            rule: "large-file".to_string(),
            message: "Large file snippet may hide complexity/quality issues".to_string(),
            category: Some("maintainability".to_string()),
            line: None,
            source: Some("local-rules".to_string()),
        });
    }

    if profile.rules_enabled("hardcoded-value")
        && (lower.contains("api_key") || lower.contains("token") || lower.contains("password"))
    {
        findings.push(Finding {
            severity: "high".to_string(),
            rule: "hardcoded-value".to_string(),
            message: "Possible hardcoded credential-like token detected".to_string(),
            category: Some("security".to_string()),
            line: None,
            source: Some("local-rules".to_string()),
        });
    }

    findings
}

fn run_local_rules_fast_aware(
    content: &str,
    profile: &EffectiveProfile,
    fast: bool,
) -> Vec<Finding> {
    if !fast {
        return run_local_rules(content, profile);
    }

    let mut findings = Vec::new();

    if profile.rules_enabled("todo-comment")
        && content.contains("TODO")
        && profile.name != "kernel-c"
    {
        findings.push(Finding {
            severity: "info".to_string(),
            rule: "todo-comment".to_string(),
            message: "TODO marker found in file content".to_string(),
            category: Some("maintainability".to_string()),
            line: None,
            source: Some("local-rules".to_string()),
        });
    }

    if profile.rules_enabled("unsafe-usage") && content.contains("unsafe") {
        findings.push(Finding {
            severity: "high".to_string(),
            rule: "unsafe-usage".to_string(),
            message: "Possible unsafe usage detected".to_string(),
            category: Some("safety".to_string()),
            line: None,
            source: Some("local-rules".to_string()),
        });
    }

    findings
}

fn severity_threshold_met(finding: &Finding, threshold: &str) -> bool {
    let t = severity_rank(normalize_severity(threshold));
    let s = severity_rank(normalize_severity(&finding.severity));
    s >= t
}

fn kernel_rule_allowed(finding: &Finding) -> bool {
    matches!(
        finding.rule.as_str(),
        "goto-out-without-cleanup"
            | "return-without-unlock"
            | "null-deref-before-check"
            | "missing-__init-__exit"
            | "RCU-missing-annotation"
            | "sleeping-in-atomic"
            | "BUG_ON-usage"
            | "printk-without-level"
    )
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

fn adapt_kernel_profile_for_subtree(profile: &mut EffectiveProfile, root: &Path) {
    if profile.name != "kernel-c" {
        return;
    }

    let Some(name) = root.file_name().and_then(|name| name.to_str()) else {
        return;
    };

    let subtree_roots = ["drivers", "fs", "include", "ipc", "kernel", "mm", "net"];
    if subtree_roots.contains(&name) {
        profile.include_globs = vec!["**/*.c".to_string(), "**/*.h".to_string()];
        profile.exclude_globs = vec![
            ".git/**".to_string(),
            "Documentation/**".to_string(),
            "samples/**".to_string(),
            "tools/**".to_string(),
        ];
    }
}

fn profile_path_candidates(root: &Path, name: &str) -> Vec<PathBuf> {
    [("toml"), ("yaml"), ("yml")]
        .into_iter()
        .map(|ext| root.join(format!("{name}.{ext}")))
        .collect()
}

fn load_profile_file(path: &Path) -> Result<RuntimeProfile> {
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

fn resolve_profile_from_file(
    path: &Path,
    root: &Path,
    seen: &mut HashSet<PathBuf>,
    seen_names: &mut HashSet<String>,
) -> Result<(RuntimeProfile, ProfileSource)> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("canonicalize profile {}", path.display()))?;
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

    seen.insert(canonical.clone());
    seen_names.insert(profile.name.clone());

    if let Some(inherits) = profile.inherits.clone() {
        let base_profile = if let Some(base_path) =
            profile_path_candidates(&root.join(".dn/profiles"), &inherits)
                .into_iter()
                .find(|candidate| candidate.exists())
        {
            let (base, _) = resolve_profile_from_file(&base_path, root, seen, seen_names)?;
            base
        } else if let Some(profile) = builtin_profile(&inherits) {
            profile
        } else {
            return Err(anyhow!("unknown inherited profile '{inherits}'"));
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

pub fn scan_repository(root: impl AsRef<Path>, options: &ScanOptions) -> Result<ScanReport> {
    let start = Instant::now();
    let root = root.as_ref();
    let root_path = root
        .canonicalize()
        .with_context(|| format!("canonicalize {}", root.display()))?;

    let (base_profile, source) =
        load_profile(&options.profile_name, root).context("load profile")?;
    let mut profile = base_profile.to_effective();
    adapt_kernel_profile_for_subtree(&mut profile, &root_path);
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
    let ignore_rules = load_ignore_rules(&root_path);
    let mut cache = load_cache(&root_path);

    let (include_set, exclude_set) = build_selectors(&profile).context("build selector")?;
    let mut walk = WalkBuilder::new(&root_path);
    walk.hidden(!profile.include_hidden)
        .git_ignore(true)
        .git_global(true)
        .parents(true);

    let mut files = Vec::new();
    let mut total_bytes = 0u64;
    let mut total_files_scanned = 0usize;
    let mut files_discovered = 0usize;
    let mut skipped_large_files = 0usize;
    let mut truncated = false;
    let mut errors = Vec::new();
    let mut worker_mode = if options.fast {
        "disabled (fast)".to_string()
    } else {
        "disabled".to_string()
    };

    let mut worker_registry = if profile.worker_enabled && !options.fast {
        let registry = WorkerRegistry::new(profile.worker_timeout_ms, profile.worker_retries);
        if let Some(cfg) = registry.get("c") {
            if cfg.retries > 0 {
                worker_mode = "c".to_string();
            } else {
                worker_mode = "c (single-shot)".to_string();
            }
        } else if let Some(cfg) = registry.get("python") {
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

    let mut ai_files_used = 0usize;

    let mut pending_batch_c: Vec<(String, String)> = Vec::new();
    let mut pending_batch_indexes: Vec<usize> = Vec::new();

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
                continue;
            }
        };

        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                errors.push(format!("metadata {}: {err}", path.display()));
                continue;
            }
        };

        if metadata.is_dir() {
            continue;
        }

        if !metadata.is_file() {
            continue;
        }

        let path = path.to_path_buf();
        let relative = match path.strip_prefix(&root_path) {
            Ok(rel) => rel,
            Err(_) => path.as_path(),
        };

        if !is_included(relative, &include_set, &exclude_set) {
            continue;
        }
        if is_ignored_by_rules(relative, &ignore_rules) {
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
        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0);

        match read_text_preview(&path, profile.limits.max_file_read_bytes) {
            Ok(content) => {
                let content_hash = content_cache_hash(&content);
                if let Some(entry) = cache.files.get(&rel_path) {
                    if entry.size == size
                        && entry.modified == modified
                        && entry.content_hash == content_hash
                    {
                        files.push(FileEntry {
                            path: rel_path.clone(),
                            size,
                            findings: entry.findings.clone(),
                            content_preview: entry.content_preview.clone(),
                        });
                        continue;
                    }
                }

                let mut findings = run_local_rules_fast_aware(&content, &profile, options.fast);
                let suspicious = !options.fast
                    && profile
                        .suspicious_patterns
                        .iter()
                        .any(|pat| content.to_lowercase().contains(&pat.to_lowercase()));

                let language = detect_language(&path).unwrap_or_else(|| "unknown".to_string());
                let file_index = files.len();
                findings.retain(|finding| {
                    if profile.name == "kernel-c" {
                        return kernel_rule_allowed(finding);
                    }
                    profile.prioritize_rules.contains(&finding.rule)
                        || severity_threshold_met(finding, &profile.severity_threshold)
                });

                let content_preview = if options.include_content || profile.include_content_preview
                {
                    Some(content.chars().take(1024).collect())
                } else {
                    None
                };

                files.push(FileEntry {
                    path: rel_path.clone(),
                    size,
                    findings,
                    content_preview,
                });

                if profile.worker_enabled && suspicious && language == "c" {
                    pending_batch_indexes.push(file_index);
                    pending_batch_c.push((rel_path.clone(), content.clone()));
                } else if profile.worker_enabled && suspicious {
                    if let Some(registry) = worker_registry.as_mut() {
                        if registry.supports(&language) {
                            match registry.analyze(&language, &rel_path, &content) {
                                Ok(worker_findings) => {
                                    files[file_index].findings.extend(worker_findings)
                                }
                                Err(err) => errors.push(format!("worker {}: {err}", rel_path)),
                            }
                        }
                    }
                }

                if profile.ai.enabled
                    && ai_files_used < profile.ai.max_ai_files
                    && suspicious
                    && !options.fast
                {
                    match review_engine.analyze_file_if_enabled(
                        &profile.ai,
                        AiRequest {
                            path: rel_path.clone(),
                            language: detect_language(&path),
                            content,
                            summary_profile: profile.name.clone(),
                        },
                    ) {
                        Ok(ai_findings) => {
                            files[file_index].findings.extend(ai_findings);
                            ai_files_used += 1;
                        }
                        Err(err) => errors.push(format!("ai provider {}: {err}", profile.name)),
                    }
                }

                files[file_index].findings.sort_by(|a, b| {
                    severity_rank(normalize_severity(&b.severity))
                        .cmp(&severity_rank(normalize_severity(&a.severity)))
                });
                cache.files.insert(
                    rel_path,
                    CacheEntry {
                        size,
                        modified,
                        content_hash,
                        findings: files[file_index].findings.clone(),
                        content_preview: files[file_index].content_preview.clone(),
                    },
                );
            }
            Err(err) => errors.push(format!("read {}: {err}", rel_path)),
        }
    }

    if !pending_batch_c.is_empty() && pending_batch_c.len() >= 4 {
        if let Some(registry) = worker_registry.as_mut() {
            match registry.scan_files("c", &pending_batch_c) {
                Ok(results) => {
                    for index in pending_batch_indexes {
                        let path = files[index].path.clone();
                        if let Some(worker_findings) = results.get(&path) {
                            files[index].findings.extend(worker_findings.clone());
                            if profile.name == "kernel-c" {
                                files[index].findings.retain(kernel_rule_allowed);
                            }
                            files[index].findings.sort_by(|a, b| {
                                severity_rank(normalize_severity(&b.severity))
                                    .cmp(&severity_rank(normalize_severity(&a.severity)))
                            });
                            if let Some(cache_entry) = cache.files.get_mut(&path) {
                                cache_entry.findings = files[index].findings.clone();
                            }
                        }
                    }
                    worker_mode = "c-batch".to_string();
                }
                Err(err) => errors.push(format!("worker batch c: {err}")),
            }
        }
    } else if !pending_batch_c.is_empty() {
        if let Some(registry) = worker_registry.as_mut() {
            for (path, content) in pending_batch_c {
                match registry.analyze("c", &path, &content) {
                    Ok(worker_findings) => {
                        if let Some(file) = files.iter_mut().find(|file| file.path == path) {
                            file.findings.extend(worker_findings);
                            if profile.name == "kernel-c" {
                                file.findings.retain(kernel_rule_allowed);
                            }
                            file.findings.sort_by(|a, b| {
                                severity_rank(normalize_severity(&b.severity))
                                    .cmp(&severity_rank(normalize_severity(&a.severity)))
                            });
                            if let Some(cache_entry) = cache.files.get_mut(&path) {
                                cache_entry.findings = file.findings.clone();
                            }
                        }
                    }
                    Err(err) => errors.push(format!("worker {}: {err}", path)),
                }
            }
            worker_mode = "c".to_string();
        }
    }
    save_cache(&root_path, &cache);

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
        worker_mode.clone()
    } else {
        "disabled".to_string()
    };

    let diagnostics: Vec<Diagnostic> = errors
        .iter()
        .map(|message| Diagnostic {
            level: "warning".to_string(),
            source: "scan".to_string(),
            code: "runtime".to_string(),
            message: message.clone(),
            path: None,
        })
        .collect();

    Ok(ScanReport {
        schema_version: "2".to_string(),
        metadata: ReportMetadata {
            root: root_path.to_string_lossy().to_string(),
            profile: profile.name.clone(),
            profile_source: profile_source.clone(),
            command: "scan".to_string(),
            duration_ms: start.elapsed().as_millis(),
            truncated,
            summary_only: options.summary_only,
        },
        integrations: Integrations {
            worker: IntegrationStatus {
                enabled: profile.worker_enabled,
                mode: worker_mode.clone(),
                used: profile.worker_enabled,
                strict: false,
            },
            provider: IntegrationStatus {
                enabled: profile.ai.enabled,
                mode: review_engine.provider_name().to_string(),
                used: profile.ai.enabled,
                strict: false,
            },
        },
        stats: ScanStats {
            findings_total: total_findings,
            files_discovered,
            files_scanned: total_files_scanned,
            files_selected,
            files_skipped,
            severity_breakdown: severity.clone(),
        },
        root: root_path.to_string_lossy().to_string(),
        profile: profile.name.clone(),
        provider: format!("{}@{}", review_engine.provider_name(), profile_source),
        worker: worker_summary,
        profile_source: profile_source.clone(),
        files_discovered,
        files_scanned: total_files_scanned,
        files_selected: if options.summary_only { 0 } else { files_selected },
        files_skipped,
        total_files: files_discovered,
        total_bytes,
        skipped_large_files,
        truncated,
        errors,
        files: if options.summary_only { Vec::new() } else { files },
        severity_breakdown: severity,
        duration_ms: start.elapsed().as_millis(),
        summary: format!(
            "Scanned {files_discovered} files ({total_files_scanned} scanned), {total_findings} findings in {}ms",
            start.elapsed().as_millis()
        ),
        diagnostics,
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
        assert!(
            report.worker == "python"
                || report.worker == "python (single-shot)"
                || report.worker == "c"
                || report.worker == "c-batch"
                || report.worker == "c (single-shot)"
        );
        assert!(
            report.errors.is_empty(),
            "unexpected scan errors: {:?}",
            report.errors
        );
        assert!(!report.files[0].findings.is_empty());
        let _ = fs::remove_dir_all(root);
    }
}
