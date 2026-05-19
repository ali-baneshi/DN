use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::Result;

use crate::worker::session::WorkerSession;
use crate::Finding;

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub timeout_ms: u64,
    pub retries: u32,
    pub preflight: Vec<String>,
}

pub struct WorkerRegistry {
    workers: HashMap<String, WorkerConfig>,
    sessions: HashMap<String, WorkerSession>,
}

impl WorkerRegistry {
    pub fn new(worker_timeout_ms: u64, worker_retries: u32) -> Self {
        let mut registry = Self {
            workers: HashMap::new(),
            sessions: HashMap::new(),
        };

        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("workers");

        let python_script = root.join("python").join("dn_worker.py");
        if let Some(command) = resolve_runtime_command(&python_script, &["python3", "python"]) {
            registry.register(
                "python",
                WorkerConfig {
                    command,
                    args: vec![python_script.to_string_lossy().to_string()],
                    timeout_ms: worker_timeout_ms,
                    retries: worker_retries,
                    preflight: vec!["python-worker".to_string()],
                },
            );
        }

        let java_script = root.join("java").join("dn_worker.py");
        if let Some(command) = resolve_runtime_command(&java_script, &["python3", "python"]) {
            registry.register(
                "java",
                WorkerConfig {
                    command,
                    args: vec![java_script.to_string_lossy().to_string()],
                    timeout_ms: worker_timeout_ms,
                    retries: worker_retries,
                    preflight: vec!["java-worker".to_string()],
                },
            );
        }

        let ts_script = root.join("typescript").join("dn_worker.js");
        if let Some(command) = resolve_runtime_command(&ts_script, &["node"]) {
            for language in ["typescript", "javascript"] {
                registry.register(
                    language,
                    WorkerConfig {
                        command: command.clone(),
                        args: vec![ts_script.to_string_lossy().to_string()],
                        timeout_ms: worker_timeout_ms,
                        retries: worker_retries,
                        preflight: vec!["typescript-worker".to_string()],
                    },
                );
            }
        }

        registry
    }

    pub fn register(&mut self, language: &str, config: WorkerConfig) {
        self.workers.insert(language.to_string(), config);
    }

    pub fn get(&self, language: &str) -> Option<&WorkerConfig> {
        self.workers.get(language)
    }

    pub fn supports(&self, language: &str) -> bool {
        self.workers.contains_key(language)
    }

    pub fn analyze(&mut self, language: &str, path: &str, content: &str) -> Result<Vec<Finding>> {
        let mut attempts = 0;
        loop {
            attempts += 1;

            if !self.sessions.contains_key(language) {
                let config = self.workers.get(language).ok_or_else(|| {
                    anyhow::anyhow!("no worker registered for language: {language}")
                })?;

                if !worker_script_is_safe(&config.args) {
                    return Err(anyhow::anyhow!(
                        "worker script path is missing or unsafe for language: {language}"
                    ));
                }
                let session = WorkerSession::new(&config.command, &config.args)?;
                self.sessions.insert(language.to_string(), session);
            }

            let maybe_session = self.sessions.get_mut(language);
            if let Some(session) = maybe_session {
                match session.analyze(path, Some(language), content) {
                    Ok(findings) => return Ok(findings),
                    Err(err) => {
                        self.sessions.remove(language);
                        let retries = self
                            .workers
                            .get(language)
                            .map(|cfg| cfg.retries)
                            .unwrap_or(0);
                        if attempts > retries + 1 {
                            return Err(err);
                        }
                    }
                }
            }
        }
    }
}

fn resolve_runtime_command(script_path: &Path, candidates: &[&str]) -> Option<String> {
    if !script_path.is_file() {
        return None;
    }

    for candidate in candidates {
        let available = Command::new(candidate)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        if available {
            return Some((*candidate).to_string());
        }
    }

    None
}

fn worker_script_is_safe(args: &[String]) -> bool {
    if args.is_empty() {
        return false;
    }
    let script = std::path::Path::new(&args[0]);
    if !script.is_file() {
        return false;
    }
    fs::canonicalize(script).is_ok()
}
