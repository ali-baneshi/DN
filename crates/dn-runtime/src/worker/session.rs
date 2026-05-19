use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use anyhow::{anyhow, Result};
use dn_ipc::{WorkerFinding, WorkerRequest, PROTOCOL_VERSION};

use crate::Finding;

// SECURITY: Maximum size of a single worker response line to prevent memory exhaustion
const MAX_WORKER_RESPONSE_BYTES: usize = 4 * 1024 * 1024;
// SECURITY: Maximum findings from a single worker response
const MAX_WORKER_FINDINGS: usize = 500;
// SECURITY: Maximum length for individual worker finding fields
const MAX_WORKER_FIELD_LEN: usize = 4096;

/// Allowed severity values from worker output
const WORKER_ALLOWED_SEVERITIES: &[&str] = &["info", "low", "medium", "high", "critical"];

/// Read a line from the worker, enforcing a maximum size limit
fn read_bounded_line(reader: &mut BufReader<ChildStdout>, limit: usize) -> Result<String> {
    let mut line = String::new();
    let mut total = 0usize;
    loop {
        let len = {
            let available = reader
                .fill_buf()
                .map_err(|err| anyhow!("failed reading worker output: {err}"))?;
            if available.is_empty() {
                break;
            }
            if let Some(newline_pos) = available.iter().position(|&b| b == b'\n') {
                let chunk = &available[..=newline_pos];
                total += chunk.len();
                if total > limit {
                    return Err(anyhow!(
                        "worker response exceeds size limit ({} bytes)",
                        limit
                    ));
                }
                let text = std::str::from_utf8(chunk)
                    .map_err(|_| anyhow!("worker response is not valid UTF-8"))?;
                line.push_str(text);
                chunk.len()
            } else {
                let len = available.len();
                total += len;
                if total > limit {
                    return Err(anyhow!(
                        "worker response exceeds size limit ({} bytes)",
                        limit
                    ));
                }
                let text = std::str::from_utf8(available)
                    .map_err(|_| anyhow!("worker response is not valid UTF-8"))?;
                line.push_str(text);
                len
            }
        };
        reader.consume(len);
        if line.ends_with('\n') {
            break;
        }
    }
    Ok(line)
}

/// Validate and sanitize a severity value from worker output
fn validate_worker_severity(raw: &str) -> String {
    let lower = raw.trim().to_lowercase();
    if WORKER_ALLOWED_SEVERITIES.contains(&lower.as_str()) {
        lower
    } else {
        "info".to_string()
    }
}

/// Truncate a field value to max length with valid UTF-8 boundary
fn truncate_field(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut end = max_len;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        s[..end].to_string()
    }
}

pub struct WorkerSession {
    _child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    handshake_done: bool,
    request_seq: u64,
}

impl WorkerSession {
    pub fn new(command: &str, args: &[String]) -> Result<Self> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // SECURITY: Prevent the worker from inheriting unnecessary environment variables
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .env("HOME", std::env::var("HOME").unwrap_or_default())
            .env("LANG", "C.UTF-8")
            .spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("worker stdin unavailable"))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("worker stdout unavailable"))?;

        Ok(Self {
            _child: child,
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
            handshake_done: false,
            request_seq: 0,
        })
    }

    pub fn analyze(
        &mut self,
        path: &str,
        language: Option<&str>,
        content: &str,
    ) -> Result<Vec<Finding>> {
        if !self.handshake_done {
            self.handshake()?;
            self.handshake_done = true;
        }

        self.request_seq = self.request_seq.saturating_add(1);
        let request_id = format!("scan-request-{}", self.request_seq);
        let request = WorkerRequest {
            protocol_version: PROTOCOL_VERSION.to_string(),
            request_id: request_id.clone(),
            method: "analyze_file".to_string(),
            params: dn_ipc::WorkerAnalyzeFileParams {
                path: path.to_string(),
                language: language.map(str::to_string),
                content: content.to_string(),
            },
        };

        let payload = serde_json::to_string(&request)?;
        self.stdin
            .write_all(format!("{}\n", payload).as_bytes())
            .map_err(|err| anyhow!("failed to write request: {err}"))?;
        self.stdin
            .flush()
            .map_err(|err| anyhow!("failed to flush request: {err}"))?;

        // SECURITY: Use bounded read to prevent memory exhaustion from malicious worker
        let line = read_bounded_line(&mut self.stdout, MAX_WORKER_RESPONSE_BYTES)?;

        if line.trim().is_empty() {
            return Err(anyhow!("empty worker response"));
        }

        let response: dn_ipc::WorkerResponse = serde_json::from_str(&line)
            .map_err(|err| anyhow!("failed parsing worker response: {err}"))?;
        if response.request_id != request_id {
            return Err(anyhow!(
                "unexpected worker response id: expected {}, got {}",
                request_id,
                response.request_id
            ));
        }

        if response.status != "ok" {
            return Err(anyhow!(response
                .error
                .unwrap_or_else(|| "worker returned non-ok status".to_string())));
        }

        Ok(response
            .findings
            .into_iter()
            .take(MAX_WORKER_FINDINGS)
            .map(|finding: WorkerFinding| Finding {
                rule: truncate_field(&finding.rule, 256),
                severity: validate_worker_severity(&finding.severity),
                message: truncate_field(&finding.message, MAX_WORKER_FIELD_LEN),
                category: finding.category.map(|c| truncate_field(&c, 256)),
                line: finding.line,
                source: Some("worker:python".to_string()),
                origin: "worker".to_string(),
            })
            .collect())
    }

    fn handshake(&mut self) -> Result<()> {
        let request = serde_json::json!({
            "protocol_version": PROTOCOL_VERSION,
            "request_id": "hello-request",
            "method": "hello",
            "params": {}
        });

        self.stdin
            .write_all(format!("{}\n", request).as_bytes())
            .map_err(|err| anyhow!("failed to send worker hello: {err}"))?;
        self.stdin
            .flush()
            .map_err(|err| anyhow!("failed to flush worker hello: {err}"))?;

        // SECURITY: Use bounded read for handshake response too
        let line = read_bounded_line(&mut self.stdout, MAX_WORKER_RESPONSE_BYTES)?;

        let response: dn_ipc::WorkerResponse = serde_json::from_str(&line)
            .map_err(|err| anyhow!("failed parsing worker hello response: {err}"))?;

        if response.status != "ok" {
            return Err(anyhow!(
                "{}",
                response
                    .error
                    .unwrap_or_else(|| "worker hello failed".to_string())
            ));
        }

        Ok(())
    }
}
