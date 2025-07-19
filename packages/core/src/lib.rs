//! # Emotional Agents Core
//!
//! Core interfaces and types for the emotional AI agents framework.
//! This crate defines the fundamental traits and types that all plugins
//! and components implement, inspired by ElizaOS's modular architecture.

#![allow(clippy::new_without_default)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::unwrap_or_default)]
#![allow(clippy::clone_on_copy)]

pub mod action;
pub mod agent;
pub mod character;
pub mod context;
pub mod emotion;
pub mod evaluator;
pub mod event;
pub mod memory;
pub mod message;
pub mod plugin;
pub mod provider;

// Re-export core types
pub use action::{Action, ActionRequest, ActionResult};
pub use agent::{Agent, AgentConfig, AgentId};
pub use character::{Character, CharacterSheet, Personality};
pub use context::{Context, MessageContext};
pub use emotion::{Emotion, EmotionalEvent, EmotionalState};
pub use evaluator::{EvaluationResult, Evaluator};
pub use event::{Event, EventHandler};
pub use memory::{Memory, MemoryProvider, MemoryQuery};
pub use message::{Message, MessageContent, MessageRole};
pub use plugin::{Plugin, PluginConfig, PluginManager, PluginRegistry};
pub use provider::{Provider, ProviderResult};

/// Result type used throughout the framework
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// The main emotional agents runtime
pub struct EmotionalAgentsRuntime {
    plugin_manager: PluginManager,
    agents: std::collections::HashMap<AgentId, Box<dyn Agent>>,
}

impl EmotionalAgentsRuntime {
    pub fn new() -> Self {
        Self {
            plugin_manager: PluginManager::new(),
            agents: std::collections::HashMap::new(),
        }
    }

    /// Register a plugin with the runtime
    pub async fn register_plugin(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        self.plugin_manager.register(plugin).await
    }

    /// Create and register a new agent
    pub async fn create_agent(&mut self, config: AgentConfig) -> Result<AgentId> {
        let agent = self.plugin_manager.create_agent(config).await?;
        let id = agent.id();
        self.agents.insert(id.clone(), agent);
        Ok(id)
    }

    /// Process a message for a specific agent
    pub async fn process_message(
        &mut self,
        agent_id: &AgentId,
        message: Message,
    ) -> Result<Vec<Message>> {
        let agent = self.agents.get_mut(agent_id).ok_or("Agent not found")?;
        agent.process_message(message).await
    }
}
