//! Bootstrap plugin providing core agent functionality and runtime services
//!
//! This plugin provides the essential components for creating and running
//! emotional AI agents within the framework.

#![allow(clippy::new_without_default)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::redundant_closure)]

use async_trait::async_trait;
use emotional_agents_core::*;

pub mod actions;
pub mod agent;
pub mod providers;

pub use actions::*;
pub use agent::*;
pub use providers::*;

/// Bootstrap plugin that provides core agent functionality
pub struct BootstrapPlugin {
    config: PluginConfig,
}

impl BootstrapPlugin {
    pub fn new() -> Self {
        Self {
            config: plugin_config!(
                "bootstrap",
                "Bootstrap Plugin",
                "0.1.0",
                "Provides core agent functionality and runtime services"
            ),
        }
    }
}

#[async_trait]
impl Plugin for BootstrapPlugin {
    fn config(&self) -> &PluginConfig {
        &self.config
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Bootstrap plugin initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Bootstrap plugin shutting down");
        Ok(())
    }

    fn actions(&self) -> Vec<Box<dyn Action>> {
        vec![
            Box::new(ThinkAction::new()),
            Box::new(RespondAction::new()),
            Box::new(ReflectAction::new()),
        ]
    }

    fn providers(&self) -> Vec<Box<dyn Provider>> {
        vec![
            Box::new(ConversationProvider::new()),
            Box::new(UserProvider::new()),
        ]
    }

    fn evaluators(&self) -> Vec<Box<dyn Evaluator>> {
        vec![Box::new(BasicPerformanceEvaluator::new())]
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

impl Default for BootstrapPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_initialization() {
        let mut plugin = BootstrapPlugin::new();

        assert!(plugin.initialize().await.is_ok());
        assert!(plugin.health_check().await.unwrap());
        assert!(plugin.shutdown().await.is_ok());
    }

    #[test]
    fn test_plugin_components() {
        let plugin = BootstrapPlugin::new();

        assert!(!plugin.actions().is_empty());
        assert!(!plugin.providers().is_empty());
        assert!(!plugin.evaluators().is_empty());
    }
}
