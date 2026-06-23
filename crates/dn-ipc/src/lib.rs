use serde::{Deserialize, Serialize};

<<<<<<< HEAD
pub const PROTOCOL_VERSION: &str = "1.0.0";
=======
pub const PROTOCOL_VERSION: &str = "0.1.0";
>>>>>>> feature/persistent-workers

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
<<<<<<< HEAD
pub struct WorkerScanFileParams {
    pub path: String,
    pub language: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum WorkerParams {
    AnalyzeFile(WorkerAnalyzeFileParams),
    ScanFiles { files: Vec<WorkerScanFileParams> },
    Empty(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
=======
>>>>>>> feature/persistent-workers
pub struct WorkerRequest {
    pub protocol_version: String,
    pub request_id: String,
    pub method: String,
<<<<<<< HEAD
    pub params: WorkerParams,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerBatchResult {
    pub path: String,
    #[serde(default)]
    pub findings: Vec<WorkerFinding>,
=======
    pub params: WorkerAnalyzeFileParams,
>>>>>>> feature/persistent-workers
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerResponse {
    pub protocol_version: String,
    pub request_id: String,
    pub status: String,
    #[serde(default)]
    pub findings: Vec<WorkerFinding>,
    #[serde(default)]
<<<<<<< HEAD
    pub results: Vec<WorkerBatchResult>,
    #[serde(default)]
=======
>>>>>>> feature/persistent-workers
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
<<<<<<< HEAD
    pub column: Option<u32>,
    #[serde(default)]
=======
>>>>>>> feature/persistent-workers
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
