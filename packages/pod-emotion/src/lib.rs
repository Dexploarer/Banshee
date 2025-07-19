//! # Emotion Plugin
//!
//! Simple emotion plugin that provides basic emotional state tracking
//! for agents using the emotion_engine crate.

use async_trait::async_trait;
use banshee_core::plugin::{self, Pod, PodConfig};
use banshee_core::*;

/// Main emotion plugin
pub struct EmotionPlugin {
    config: PodConfig,
}

impl EmotionPlugin {
    pub fn new() -> Self {
        Self {
            config: PodConfig {
                id: "emotion".to_string(),
                name: "Emotion Plugin".to_string(),
                version: plugin::Version::new(0, 1, 0),
                description: "Provides basic emotional state tracking and processing".to_string(),
                dependencies: vec![],
                provides: vec![],
                settings: std::collections::HashMap::new(),
            },
        }
    }
}

impl Default for EmotionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Pod for EmotionPlugin {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn initialize(&mut self) -> plugin::PodResult<()> {
        tracing::info!("Emotion plugin initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> plugin::PodResult<()> {
        tracing::info!("Emotion plugin shutting down");
        Ok(())
    }

    fn actions(&self) -> Vec<Box<dyn Action>> {
        vec![]
    }

    fn providers(&self) -> Vec<Box<dyn Provider>> {
        vec![]
    }

    fn evaluators(&self) -> Vec<Box<dyn Evaluator>> {
        vec![]
    }

    async fn health_check(&self) -> plugin::PodResult<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_initialization() {
        let mut plugin = EmotionPlugin::new();

        assert!(plugin.initialize().await.is_ok());
        assert!(plugin.health_check().await.unwrap());
        assert!(plugin.shutdown().await.is_ok());
    }

    #[test]
    fn test_plugin_config() {
        let plugin = EmotionPlugin::new();
        assert_eq!(plugin.config().name, "emotion");
        assert_eq!(plugin.config().version, "0.1.0");
    }
}
