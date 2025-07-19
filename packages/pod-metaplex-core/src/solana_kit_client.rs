/**
 * Rust client for Solana Agent Kit integration
 * 
 * This module provides a Rust interface to the TypeScript Solana Agent Kit
 * by spawning Node.js processes and communicating via JSON
 */

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::collections::HashMap;
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{debug, error, info};

/// Configuration for Solana Agent Kit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaAgentConfig {
    pub private_key: String,
    pub rpc_url: String,
    pub openai_api_key: Option<String>,
}

/// NFT Collection deployment options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionDeployOptions {
    pub name: String,
    pub uri: String,
    pub royalty_basis_points: u16,
    pub creators: Vec<Creator>,
}

/// Creator information for NFTs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creator {
    pub address: String,
    pub percentage: u8,
}

/// Result from Solana Agent Kit operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaAgentResult {
    pub success: bool,
    pub signature: Option<String>,
    pub mint: Option<String>,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Solana Agent Kit client that communicates with TypeScript bridge
pub struct SolanaKitClient {
    node_path: String,
    bridge_script_path: String,
    config: Option<SolanaAgentConfig>,
}

impl SolanaKitClient {
    /// Create a new Solana Kit client
    pub fn new() -> Result<Self> {
        let node_path = which::which("node")
            .context("Node.js not found in PATH")?
            .to_string_lossy()
            .to_string();

        // Assume the bridge script is in the same directory as src
        let bridge_script_path = std::env::current_dir()?
            .join("packages/pod-metaplex-core/dist/solana_agent_bridge.js")
            .to_string_lossy()
            .to_string();

        Ok(Self {
            node_path,
            bridge_script_path,
            config: None,
        })
    }

    /// Initialize the Solana Agent Kit with configuration
    pub async fn initialize(&mut self, config: SolanaAgentConfig) -> Result<bool> {
        info!("Initializing Solana Agent Kit with config");
        
        let config_json = serde_json::to_string(&config)
            .context("Failed to serialize config")?;

        let result = self.call_bridge_function("initializeSolanaAgent", &[&config_json]).await?;
        
        if result.success {
            self.config = Some(config);
            info!("Solana Agent Kit initialized successfully");
            Ok(true)
        } else {
            error!("Failed to initialize Solana Agent Kit: {:?}", result.error);
            Ok(false)
        }
    }

    /// Deploy an NFT collection
    pub async fn deploy_collection(&self, options: CollectionDeployOptions) -> Result<SolanaAgentResult> {
        self.ensure_initialized()?;
        
        let options_json = serde_json::to_string(&options)
            .context("Failed to serialize collection options")?;

        info!("Deploying NFT collection: {}", options.name);
        
        self.call_bridge_function("deployCollectionFFI", &[&options_json]).await
    }

    /// Get asset information by ID
    pub async fn get_asset(&self, asset_id: &str) -> Result<SolanaAgentResult> {
        self.ensure_initialized()?;
        
        info!("Getting asset information for: {}", asset_id);
        
        self.call_bridge_function("getAssetFFI", &[asset_id]).await
    }

    /// Get wallet address
    pub async fn get_wallet_address(&self) -> Result<Option<String>> {
        self.ensure_initialized()?;
        
        let result = self.call_bridge_function("getWalletAddressFFI", &[]).await?;
        
        if result.success {
            Ok(result.data.and_then(|d| d.get("address").and_then(|a| a.as_str().map(String::from))))
        } else {
            Ok(None)
        }
    }

    /// Check if the agent is initialized
    pub async fn is_initialized(&self) -> Result<bool> {
        let output = TokioCommand::new(&self.node_path)
            .arg("-e")
            .arg(&format!(
                r#"
                const {{ isInitializedFFI }} = require('{}');
                console.log(JSON.stringify({{ result: isInitializedFFI() }}));
                "#,
                self.bridge_script_path
            ))
            .output()
            .await
            .context("Failed to execute Node.js command")?;

        if !output.status.success() {
            return Ok(false);
        }

        let stdout = String::from_utf8(output.stdout)
            .context("Failed to parse Node.js output")?;

        let response: serde_json::Value = serde_json::from_str(stdout.trim())
            .context("Failed to parse JSON response")?;

        Ok(response.get("result").and_then(|r| r.as_bool()).unwrap_or(false))
    }

    /// Helper function to call bridge functions
    async fn call_bridge_function(&self, function_name: &str, args: &[&str]) -> Result<SolanaAgentResult> {
        let args_js = args.iter()
            .map(|arg| format!("'{}'", arg.replace("'", "\\'")))
            .collect::<Vec<_>>()
            .join(", ");

        let script = format!(
            r#"
            const {{ {} }} = require('{}');
            {}({}).then(result => {{
                console.log(result);
            }}).catch(error => {{
                console.log(JSON.stringify({{ success: false, error: error.message }}));
            }});
            "#,
            function_name,
            self.bridge_script_path,
            function_name,
            args_js
        );

        debug!("Executing Node.js script: {}", script);

        let output = TokioCommand::new(&self.node_path)
            .arg("-e")
            .arg(&script)
            .output()
            .await
            .context("Failed to execute Node.js command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Node.js execution failed: {}", stderr));
        }

        let stdout = String::from_utf8(output.stdout)
            .context("Failed to parse Node.js output")?;

        debug!("Node.js output: {}", stdout);

        serde_json::from_str(stdout.trim())
            .context("Failed to parse JSON response from bridge")
    }

    /// Ensure the client is initialized
    fn ensure_initialized(&self) -> Result<()> {
        if self.config.is_none() {
            return Err(anyhow::anyhow!("Solana Agent Kit not initialized"));
        }
        Ok(())
    }
}

impl Default for SolanaKitClient {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            panic!("Failed to create SolanaKitClient")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = SolanaKitClient::new();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_initialization_check() {
        let client = SolanaKitClient::new().unwrap();
        // This will fail if Node.js/bridge isn't set up, but shouldn't panic
        let _ = client.is_initialized().await;
    }
}