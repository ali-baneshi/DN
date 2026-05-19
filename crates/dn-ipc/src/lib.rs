use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerHello {
    pub protocol_version: String,
    pub worker_name: String,
    pub worker_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerAnalyzeFileParams {
    pub path: String,
    pub language: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerRequest {
    pub protocol_version: String,
    pub request_id: String,
    pub method: String,
    pub params: WorkerAnalyzeFileParams,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerResponse {
    pub protocol_version: String,
    pub request_id: String,
    pub status: String,
    #[serde(default)]
    pub findings: Vec<WorkerFinding>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerFinding {
    pub severity: String,
    pub rule: String,
    pub message: String,
    #[serde(default)]
    pub line: Option<u32>,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkerRequestEnvelope {
    Hello,
    AnalyzeFile {
        path: String,
        language: Option<String>,
        content: String,
    },
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkerResponseEnvelope {
    Hello {
        protocol_version: String,
        worker_name: String,
        worker_version: String,
    },
    Ok {
        findings: Vec<WorkerFinding>,
    },
    Error {
        error: WorkerError,
    },
    Bye,
}
