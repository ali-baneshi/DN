use std::collections::HashMap;

use anyhow::Result;

use crate::worker::session::WorkerSession;
use crate::Finding;

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub timeout_ms: u64,
    pub retries: u32,
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

        let script_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("workers")
            .join("python")
            .join("dn_worker.py");

        registry.register(
            "python",
            WorkerConfig {
                command: "python".to_string(),
                args: vec![script_path.to_string_lossy().to_string()],
                timeout_ms: worker_timeout_ms,
                retries: worker_retries,
            },
        );

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

                        let _ = retries;
                    }
                }
            }
        }
    }
}
