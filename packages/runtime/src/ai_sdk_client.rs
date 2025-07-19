//! AI SDK client module - real implementation for various LLM providers

use reqwest::Client;
use serde::{Deserialize, Serialize};

/// AI SDK client for LLM integration
pub struct AiSdkClient {
    client: Client,
    config: Config,
}

/// Configuration for AI SDK client
#[derive(Debug, Clone)]
pub struct Config {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub streaming: bool,
    pub base_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            api_key: String::new(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            streaming: false,
            base_url: None,
        }
    }
}

/// Generate request
#[derive(Debug, Clone, Serialize, Default)]
pub struct GenerateRequest {
    pub messages: Vec<Message>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub tools: Option<Vec<Tool>>,
    pub stream: Option<bool>,
}

/// Message in conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Message content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// Content part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPart {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<ImageUrl>,
}

/// Image URL content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub r#type: String,
    pub function: ToolFunction,
}

/// Tool function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function: FunctionCall,
    pub r#type: String,
}

/// Function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Generate response
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Response choice
#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub index: i32,
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Token usage information
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

/// Anthropic specific request format
#[derive(Debug, Clone, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: i32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

#[derive(Debug, Clone, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic response format
#[derive(Debug, Clone, Deserialize)]
struct AnthropicResponse {
    id: String,
    r#type: String,
    role: String,
    content: Vec<AnthropicContent>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicContent {
    r#type: String,
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicUsage {
    input_tokens: i32,
    output_tokens: i32,
}

impl AiSdkClient {
    /// Create new AI SDK client
    pub async fn new(config: Config) -> crate::Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self { client, config })
    }

    /// Generate completion
    pub async fn generate(&self, request: GenerateRequest) -> crate::Result<GenerateResponse> {
        match self.config.provider.as_str() {
            "openai" => self.generate_openai(request).await,
            "anthropic" => self.generate_anthropic(request).await,
            "azure" => self.generate_azure(request).await,
            provider => Err(format!("Unsupported provider: {}", provider).into()),
        }
    }

    /// Generate completion using OpenAI API
    async fn generate_openai(&self, request: GenerateRequest) -> crate::Result<GenerateResponse> {
        let url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com")
            .to_string()
            + "/v1/chat/completions";

        let mut req_body = serde_json::json!({
            "model": request.model.unwrap_or(self.config.model.clone()),
            "messages": request.messages,
        });

        if let Some(temp) = request.temperature.or(self.config.temperature) {
            req_body["temperature"] = serde_json::json!(temp);
        }

        if let Some(max_tokens) = request
            .max_tokens
            .or(self.config.max_tokens.map(|t| t as i32))
        {
            req_body["max_tokens"] = serde_json::json!(max_tokens);
        }

        if let Some(tools) = request.tools {
            req_body["tools"] = serde_json::json!(tools);
        }

        if let Some(stream) = request.stream {
            req_body["stream"] = serde_json::json!(stream);
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| format!("OpenAI API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("OpenAI API error {}: {}", status, error_text).into());
        }

        let result: GenerateResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

        Ok(result)
    }

    /// Generate completion using Anthropic API
    async fn generate_anthropic(
        &self,
        request: GenerateRequest,
    ) -> crate::Result<GenerateResponse> {
        let url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com")
            .to_string()
            + "/v1/messages";

        // Convert messages to Anthropic format
        let anthropic_messages: Vec<AnthropicMessage> = request
            .messages
            .into_iter()
            .map(|msg| AnthropicMessage {
                role: msg.role,
                content: match msg.content {
                    MessageContent::Text(text) => text,
                    MessageContent::Parts(parts) => parts
                        .into_iter()
                        .filter_map(|part| part.text)
                        .collect::<Vec<_>>()
                        .join(" "),
                },
            })
            .collect();

        let anthropic_request = AnthropicRequest {
            model: request.model.unwrap_or(self.config.model.clone()),
            max_tokens: request
                .max_tokens
                .or(self.config.max_tokens.map(|t| t as i32))
                .unwrap_or(2000),
            messages: anthropic_messages,
            temperature: request.temperature.or(self.config.temperature),
            tools: request.tools,
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| format!("Anthropic API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Anthropic API error {}: {}", status, error_text).into());
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Anthropic response: {}", e))?;

        // Convert to OpenAI-compatible format
        let content_text = anthropic_response
            .content
            .into_iter()
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");

        let result = GenerateResponse {
            id: anthropic_response.id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: anthropic_response.model,
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: anthropic_response.role,
                    content: MessageContent::Text(content_text),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: anthropic_response.stop_reason,
            }],
            usage: Some(Usage {
                prompt_tokens: anthropic_response.usage.input_tokens,
                completion_tokens: anthropic_response.usage.output_tokens,
                total_tokens: anthropic_response.usage.input_tokens
                    + anthropic_response.usage.output_tokens,
            }),
        };

        Ok(result)
    }

    /// Generate completion using Azure OpenAI
    async fn generate_azure(&self, request: GenerateRequest) -> crate::Result<GenerateResponse> {
        let base_url = self
            .config
            .base_url
            .as_deref()
            .ok_or("Azure base URL is required for Azure provider")?;

        // Azure URL format: https://{resource}.openai.azure.com/openai/deployments/{deployment}/chat/completions?api-version=2023-12-01-preview
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version=2023-12-01-preview",
            base_url, self.config.model
        );

        let mut req_body = serde_json::json!({
            "messages": request.messages,
        });

        if let Some(temp) = request.temperature.or(self.config.temperature) {
            req_body["temperature"] = serde_json::json!(temp);
        }

        if let Some(max_tokens) = request
            .max_tokens
            .or(self.config.max_tokens.map(|t| t as i32))
        {
            req_body["max_tokens"] = serde_json::json!(max_tokens);
        }

        if let Some(tools) = request.tools {
            req_body["tools"] = serde_json::json!(tools);
        }

        let response = self
            .client
            .post(&url)
            .header("api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| format!("Azure OpenAI API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Azure OpenAI API error {}: {}", status, error_text).into());
        }

        let result: GenerateResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Azure OpenAI response: {}", e))?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config::default();
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4");
    }

    #[test]
    fn test_request_serialization() {
        let message = Message {
            role: "user".to_string(),
            content: MessageContent::Text("Hello".to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        };

        let request = GenerateRequest {
            messages: vec![message],
            model: Some("gpt-4".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            tools: None,
            stream: None,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("Hello"));
        assert!(serialized.contains("gpt-4"));
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = Config {
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            api_key: "test-key".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2000),
            streaming: false,
            base_url: None,
        };

        let client = AiSdkClient::new(config).await;
        assert!(client.is_ok());
    }
}
