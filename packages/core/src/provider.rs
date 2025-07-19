use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Context, Message, Result};

/// Provider result containing contextual information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResult {
    /// Provider name
    pub provider: String,

    /// Contextual data provided
    pub data: serde_json::Value,

    /// Relevance score (0.0 to 1.0)
    pub relevance: f32,

    /// Confidence in the data (0.0 to 1.0)
    pub confidence: f32,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Timestamp when data was generated
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider name
    pub name: String,

    /// Provider description
    pub description: String,

    /// Priority level (higher = more important)
    pub priority: u32,

    /// Whether this provider is enabled
    pub enabled: bool,

    /// Provider-specific settings
    pub settings: HashMap<String, serde_json::Value>,
}

/// Core provider trait for supplying contextual information
#[async_trait]
pub trait Provider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Get the provider description
    fn description(&self) -> &str;

    /// Get the provider configuration
    fn config(&self) -> &ProviderConfig;

    /// Provide contextual information for the given context
    async fn provide(&self, context: &Context) -> Result<Vec<ProviderResult>>;

    /// Check if this provider is relevant for the given context
    async fn is_relevant(&self, context: &Context) -> Result<bool>;

    /// Initialize the provider
    async fn initialize(&mut self) -> Result<()>;

    /// Shutdown the provider
    async fn shutdown(&mut self) -> Result<()>;
}

/// Specific provider types
#[allow(async_fn_in_trait)]
pub trait MemoryProvider: Provider {
    /// Retrieve relevant memories
    async fn get_memories(&self, query: &str, limit: usize) -> Result<Vec<ProviderResult>>;

    /// Store a new memory
    async fn store_memory(&self, content: &str, importance: f32, tags: Vec<String>) -> Result<()>;
}

#[allow(async_fn_in_trait)]
pub trait EmotionProvider: Provider {
    /// Get current emotional context
    async fn get_emotional_context(&self, context: &Context) -> Result<ProviderResult>;

    /// Get emotional history
    async fn get_emotional_history(&self, limit: usize) -> Result<Vec<ProviderResult>>;
}

#[allow(async_fn_in_trait)]
pub trait ToolProvider: Provider {
    /// Get available tools for the current context
    async fn get_available_tools(&self, context: &Context) -> Result<Vec<ProviderResult>>;

    /// Get tool usage recommendations
    async fn recommend_tools(&self, context: &Context) -> Result<Vec<ProviderResult>>;
}

#[allow(async_fn_in_trait)]
pub trait KnowledgeProvider: Provider {
    /// Search knowledge base
    async fn search_knowledge(&self, query: &str, limit: usize) -> Result<Vec<ProviderResult>>;

    /// Get related knowledge
    async fn get_related(&self, topic: &str, limit: usize) -> Result<Vec<ProviderResult>>;
}

#[allow(async_fn_in_trait)]
pub trait ContextProvider: Provider {
    /// Get conversation context
    async fn get_conversation_context(&self, context: &Context) -> Result<ProviderResult>;

    /// Get user context
    async fn get_user_context(&self, user_id: &str) -> Result<ProviderResult>;
}

/// Provider execution context
#[derive(Debug, Clone)]
pub struct ProviderContext {
    /// Current message being processed
    pub current_message: Option<Message>,

    /// Previous messages in conversation
    pub conversation_history: Vec<Message>,

    /// User ID if available
    pub user_id: Option<String>,

    /// Session ID
    pub session_id: String,

    /// Additional context data
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Registry for managing providers
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn Provider>>,
    execution_order: Vec<String>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            execution_order: Vec::new(),
        }
    }

    /// Register a new provider
    pub async fn register(&mut self, mut provider: Box<dyn Provider>) -> Result<()> {
        let name = provider.name().to_string();
        provider.initialize().await?;

        // Insert based on priority
        let priority = provider.config().priority;
        let insert_pos = self
            .execution_order
            .iter()
            .position(|p| {
                self.providers
                    .get(p)
                    .map(|prov| prov.config().priority < priority)
                    .unwrap_or(true)
            })
            .unwrap_or(self.execution_order.len());

        self.execution_order.insert(insert_pos, name.clone());
        self.providers.insert(name, provider);

        Ok(())
    }

    /// Get a provider by name
    pub fn get(&self, name: &str) -> Option<&dyn Provider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    /// Execute all relevant providers for the given context
    pub async fn execute_all(&self, context: &Context) -> Result<Vec<ProviderResult>> {
        let mut results = Vec::new();

        for provider_name in &self.execution_order {
            if let Some(provider) = self.providers.get(provider_name) {
                if !provider.config().enabled {
                    continue;
                }

                if provider.is_relevant(context).await? {
                    let mut provider_results = provider.provide(context).await?;
                    results.append(&mut provider_results);
                }
            }
        }

        // Sort by relevance score (highest first)
        results.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }

    /// Execute only enabled providers
    pub async fn execute_enabled(&self, context: &Context) -> Result<Vec<ProviderResult>> {
        let mut results = Vec::new();

        for provider_name in &self.execution_order {
            if let Some(provider) = self.providers.get(provider_name) {
                if provider.config().enabled && provider.is_relevant(context).await? {
                    let mut provider_results = provider.provide(context).await?;
                    results.append(&mut provider_results);
                }
            }
        }

        Ok(results)
    }
}

/// Macro to create simple providers
#[macro_export]
macro_rules! simple_provider {
    ($name:expr, $description:expr, $provide_fn:expr) => {
        struct SimpleProvider {
            config: ProviderConfig,
            provide_fn: fn(&Context) -> Result<Vec<ProviderResult>>,
        }

        #[async_trait]
        impl Provider for SimpleProvider {
            fn name(&self) -> &str {
                &self.config.name
            }

            fn description(&self) -> &str {
                &self.config.description
            }

            fn config(&self) -> &ProviderConfig {
                &self.config
            }

            async fn provide(&self, context: &Context) -> Result<Vec<ProviderResult>> {
                (self.provide_fn)(context)
            }

            async fn is_relevant(&self, _: &Context) -> Result<bool> {
                Ok(true)
            }

            async fn initialize(&mut self) -> Result<()> {
                Ok(())
            }

            async fn shutdown(&mut self) -> Result<()> {
                Ok(())
            }
        }

        SimpleProvider {
            config: ProviderConfig {
                name: $name.to_string(),
                description: $description.to_string(),
                priority: 50,
                enabled: true,
                settings: HashMap::new(),
            },
            provide_fn: $provide_fn,
        }
    };
}
