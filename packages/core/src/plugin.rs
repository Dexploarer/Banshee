use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Action, Evaluator, Provider, Result};

/// Unique identifier for plugins
pub type PluginId = String;

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Core plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get the plugin configuration
    fn config(&self) -> &PluginConfig;

    /// Initialize the plugin with runtime context
    async fn initialize(&mut self) -> Result<()>;

    /// Shutdown the plugin gracefully
    async fn shutdown(&mut self) -> Result<()>;

    /// Get actions provided by this plugin
    fn actions(&self) -> Vec<Box<dyn Action>>;

    /// Get providers offered by this plugin
    fn providers(&self) -> Vec<Box<dyn Provider>>;

    /// Get evaluators supplied by this plugin
    fn evaluators(&self) -> Vec<Box<dyn Evaluator>>;

    /// Check if plugin is healthy
    async fn health_check(&self) -> Result<bool>;
}

/// Registry for managing all available plugins
pub struct PluginRegistry {
    plugins: HashMap<PluginId, Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a new plugin
    pub async fn register(&mut self, mut plugin: Box<dyn Plugin>) -> Result<()> {
        let id = plugin.config().id.clone();
        plugin.initialize().await?;
        self.plugins.insert(id, plugin);
        Ok(())
    }

    /// Unregister a plugin
    pub async fn unregister(&mut self, plugin_id: &PluginId) -> Result<()> {
        if let Some(mut plugin) = self.plugins.remove(plugin_id) {
            plugin.shutdown().await?;
        }
        Ok(())
    }

    /// Get a plugin by ID
    pub fn get(&self, plugin_id: &PluginId) -> Option<&dyn Plugin> {
        self.plugins.get(plugin_id).map(|p| p.as_ref())
    }

    /// Get all registered plugins
    pub fn all(&self) -> impl Iterator<Item = &dyn Plugin> {
        self.plugins.values().map(|p| p.as_ref())
    }

    /// Get all actions from all plugins
    pub fn all_actions(&self) -> Vec<Box<dyn Action>> {
        self.plugins
            .values()
            .flat_map(|plugin| plugin.actions())
            .collect()
    }

    /// Get all providers from all plugins
    pub fn all_providers(&self) -> Vec<Box<dyn Provider>> {
        self.plugins
            .values()
            .flat_map(|plugin| plugin.providers())
            .collect()
    }

    /// Get all evaluators from all plugins
    pub fn all_evaluators(&self) -> Vec<Box<dyn Evaluator>> {
        self.plugins
            .values()
            .flat_map(|plugin| plugin.evaluators())
            .collect()
    }
}

/// Plugin manager that coordinates plugin lifecycle
pub struct PluginManager {
    registry: PluginRegistry,
    actions: HashMap<String, Box<dyn Action>>,
    providers: HashMap<String, Box<dyn Provider>>,
    evaluators: HashMap<String, Box<dyn Evaluator>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            registry: PluginRegistry::new(),
            actions: HashMap::new(),
            providers: HashMap::new(),
            evaluators: HashMap::new(),
        }
    }

    /// Register a plugin and index its components
    pub async fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        // Index actions, providers, and evaluators
        for action in plugin.actions() {
            self.actions.insert(action.name().to_string(), action);
        }

        for provider in plugin.providers() {
            self.providers.insert(provider.name().to_string(), provider);
        }

        for evaluator in plugin.evaluators() {
            self.evaluators
                .insert(evaluator.name().to_string(), evaluator);
        }

        self.registry.register(plugin).await
    }

    /// Create an agent using registered plugins
    pub async fn create_agent(&self, _config: crate::AgentConfig) -> Result<Box<dyn crate::Agent>> {
        // This will be implemented by plugins that provide agent creation
        todo!("Agent creation will be handled by bootstrap plugin")
    }

    /// Get an action by name
    pub fn get_action(&self, name: &str) -> Option<&dyn Action> {
        self.actions.get(name).map(|a| a.as_ref())
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<&dyn Provider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    /// Get an evaluator by name
    pub fn get_evaluator(&self, name: &str) -> Option<&dyn Evaluator> {
        self.evaluators.get(name).map(|e| e.as_ref())
    }
}

/// Macro to help create plugin configurations
#[macro_export]
macro_rules! plugin_config {
    ($id:expr, $name:expr, $version:expr, $description:expr) => {
        PluginConfig {
            id: $id.to_string(),
            name: $name.to_string(),
            version: $version.to_string(),
            description: $description.to_string(),
            settings: std::collections::HashMap::new(),
        }
    };

    ($id:expr, $name:expr, $version:expr, $description:expr, $settings:expr) => {
        PluginConfig {
            id: $id.to_string(),
            name: $name.to_string(),
            version: $version.to_string(),
            description: $description.to_string(),
            settings: $settings,
        }
    };
}
