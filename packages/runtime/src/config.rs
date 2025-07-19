//! Configuration module for runtime

pub use crate::runtime::{
    AiSdkConfig, CacheConfig, ConsolidationConfig, DecisionConfig, DecisionStrategy,
    ErrorHandlingConfig, ExecutionLimits, KnowledgeGraphConfig, McpAuth, McpConfig,
    McpServerConfig, MemoryConfig, MonitoringConfig, RelationshipConfig, RelationshipDecayConfig,
    RuntimeConfig, RuntimeSettings, StandingMethod, StorageConfig, ToolDiscoveryConfig,
    TrustConfig,
};

// serde imports used by derive macros
use std::path::Path;

impl RuntimeConfig {
    /// Load configuration from file
    pub async fn from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let _content = tokio::fs::read_to_string(path).await?;
        // TODO: Add toml parsing when toml crate is added to dependencies
        let config = Self::default();
        Ok(config)
    }

    /// Save configuration to file
    pub async fn to_file(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Add toml serialization when toml crate is added to dependencies
        let content = format!("{:#?}", self);
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate AI SDK config
        if self.ai_sdk.model.is_empty() {
            return Err("AI SDK model cannot be empty".into());
        }

        if self.ai_sdk.temperature < 0.0 || self.ai_sdk.temperature > 2.0 {
            return Err("Temperature must be between 0.0 and 2.0".into());
        }

        // Validate MCP config
        if self.mcp.execution_limits.timeout_ms == 0 {
            return Err("Execution timeout must be greater than 0".into());
        }

        // Validate memory config
        if let Some(max_nodes) = Some(self.memory.knowledge_graph.max_nodes) {
            if max_nodes == 0 {
                return Err("Max nodes must be greater than 0".into());
            }
        }

        // Validate relationship config
        if self.relationships.trust.initial_trust < 0.0
            || self.relationships.trust.initial_trust > 1.0
        {
            return Err("Initial trust must be between 0.0 and 1.0".into());
        }

        // Validate decision config
        let total_weight = self.decision.emotion_weight
            + self.decision.logic_weight
            + self.decision.memory_weight
            + self.decision.relationship_weight;

        if (total_weight - 1.0).abs() > 0.01 {
            return Err("Decision weights must sum to 1.0".into());
        }

        Ok(())
    }

    /// Create a development configuration
    pub fn development() -> Self {
        let mut config = Self::default();

        config.runtime.debug = true;
        config.ai_sdk.temperature = 0.9;
        config.memory.consolidation.enabled = false;
        config.relationships.decay.enabled = false;

        config
    }

    /// Create a production configuration
    pub fn production() -> Self {
        let mut config = Self::default();

        config.runtime.debug = false;
        config.ai_sdk.temperature = 0.7;
        config.runtime.monitoring.enabled = true;
        config.runtime.error_handling.retry_enabled = true;

        config
    }
}
