use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::Finding;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum ProviderConfig {
    #[default]
    #[serde(rename = "disabled")]
    Disabled,
    Mock {
        #[serde(default = "mock_message_default")]
        message: String,
    },
    Ollama {
        base_url: String,
        model: String,
        #[serde(default)]
        api_key: String,
        #[serde(default = "default_timeout_secs")]
        timeout_secs: u64,
        #[serde(default = "default_temperature")]
        temperature: f32,
        #[serde(default)]
        extra_system_prompt: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileAiConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub max_ai_files: usize,
    #[serde(default = "default_ai_content_chars")]
    pub max_content_chars: usize,
    #[serde(default)]
    pub suspicious_patterns: Vec<String>,
    #[serde(default)]
    pub provider: ProviderConfig,
    #[serde(default)]
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    pub path: String,
    pub language: Option<String>,
    pub content: String,
    pub summary_profile: String,
}

#[derive(Debug, Clone)]
enum Provider {
    Disabled,
    Mock { message: String },
    Ollama(OllamaProvider),
}

#[derive(Debug, Clone)]
struct OllamaProvider {
    base_url: String,
    model: String,
    api_key: String,
    timeout_secs: u64,
    temperature: f32,
    extra_system_prompt: String,
}

impl Provider {
    fn from_config(config: ProviderConfig) -> Self {
        match config {
            ProviderConfig::Disabled => Self::Disabled,
            ProviderConfig::Mock { message } => Self::Mock { message },
            ProviderConfig::Ollama {
                base_url,
                model,
                api_key,
                timeout_secs,
                temperature,
                extra_system_prompt,
            } => Self::Ollama(OllamaProvider {
                base_url,
                model,
                api_key,
                timeout_secs,
                temperature,
                extra_system_prompt,
            }),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Provider::Disabled => "disabled",
            Provider::Mock { .. } => "mock",
            Provider::Ollama(_) => "ollama",
        }
    }

    fn analyze(&self, request: &AiRequest) -> Result<Vec<Finding>> {
        match self {
            Provider::Disabled => Ok(vec![]),
            Provider::Mock { message } => Ok(vec![Finding {
                severity: "info".to_string(),
                rule: "mock-ai-review".to_string(),
                message: message.clone(),
                category: Some("review".to_string()),
                line: None,
                source: Some("mock-provider".to_string()),
            }]),
            Provider::Ollama(provider) => analyze_with_ollama(provider, request),
        }
    }
}

fn analyze_with_ollama(provider: &OllamaProvider, request: &AiRequest) -> Result<Vec<Finding>> {
    let url = format!(
        "{}/api/chat/completions",
        provider.base_url.trim_end_matches('/')
    );
    let system_prompt = format!(
        "You are a strict code reviewer for local repository review. Return JSON only.\n\nProfile: {}\n{}",
        request.summary_profile, provider.extra_system_prompt
    );
    let user_prompt = format!(
        "Analyze this file and return JSON object with `findings` array.\nEach item: rule, severity, message, category (optional), line (optional).\nFile: {}\nLanguage: {}\n\n{}",
        request.path,
        request.language.clone().unwrap_or_else(|| "unknown".to_string()),
        request.content
    );

    let payload = serde_json::json!({
        "model": provider.model,
        "stream": false,
        "temperature": provider.temperature,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt},
        ]
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(provider.timeout_secs))
        .build()
        .context("build HTTP client")?;

    let mut builder = client.post(&url).json(&payload);
    if !provider.api_key.trim().is_empty() {
        builder = builder.bearer_auth(&provider.api_key);
    }

    let response = builder.send().context("send request to local provider")?;
    if !response.status().is_success() {
        return Err(anyhow!(
            "provider returned HTTP status {}",
            response.status()
        ));
    }

    let body: serde_json::Value = response.json().context("parse provider response")?;
    let mut content = body
        .get("choices")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("message"))
        .and_then(|v| v.get("content"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    if content.is_empty() {
        return Ok(vec![]);
    }

    if let Some(start) = content.find('{') {
        content = content[start..].to_string();
    }

    let parsed = serde_json::from_str::<serde_json::Value>(&content)
        .context("decode provider findings payload")?;
    let findings_json = parsed
        .get("findings")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut findings = Vec::new();
    for item in findings_json {
        findings.push(Finding {
            severity: item
                .get("severity")
                .and_then(|v| v.as_str())
                .unwrap_or("info")
                .to_string(),
            rule: item
                .get("rule")
                .and_then(|v| v.as_str())
                .unwrap_or("ai-review")
                .to_string(),
            message: item
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("No review message")
                .to_string(),
            category: item
                .get("category")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string()),
            line: item
                .get("line")
                .and_then(|v| v.as_u64())
                .and_then(|line| u32::try_from(line).ok()),
            source: Some(format!("{}:{}", "ollama", request.path)),
        });
    }

    Ok(findings)
}

#[derive(Debug, Clone)]
pub struct ReviewEngine {
    provider: Provider,
}

impl Default for ReviewEngine {
    fn default() -> Self {
        Self {
            provider: Provider::from_config(ProviderConfig::Disabled),
        }
    }
}

impl ReviewEngine {
    pub fn from_config(config: &ProfileAiConfig) -> Self {
        Self {
            provider: Provider::from_config(config.provider.clone()),
        }
    }

    pub fn analyze_file_if_enabled(
        &self,
        config: &ProfileAiConfig,
        mut request: AiRequest,
    ) -> Result<Vec<Finding>> {
        if !config.enabled {
            return Ok(vec![]);
        }

        if request.content.len() > config.max_content_chars {
            request.content = request
                .content
                .chars()
                .take(config.max_content_chars)
                .collect::<String>();
        }

        self.provider.analyze(&request)
    }

    pub fn provider_name(&self) -> &'static str {
        self.provider.name()
    }
}

fn mock_message_default() -> String {
    "AI review unavailable; manual follow-up is required".to_string()
}

fn default_timeout_secs() -> u64 {
    30
}

fn default_temperature() -> f32 {
    0.2
}

fn default_ai_content_chars() -> usize {
    12_000
}

impl Default for ProfileAiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_ai_files: 0,
            max_content_chars: default_ai_content_chars(),
            suspicious_patterns: vec!["TODO".to_string(), "unsafe".to_string()],
            provider: ProviderConfig::Disabled,
            prompt: "Look for risky patterns and code smells. Keep findings actionable."
                .to_string(),
        }
    }
}
