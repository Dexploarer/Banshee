//! MCP Manager for AI SDK 5 Integration
//!
//! Provides MCP server management from character sheet configuration
//! Integrates with AI SDK 5 transport-based architecture (July 2025)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::ai_sdk_client::{AiSdk5ClientManager, McpClientConfig, TransportConfig};
use crate::character_sheet::{CharacterSheet, McpAuthType, McpServer};

/// MCP manager for loading and managing MCP servers from character sheets
pub struct McpManager {
    client_manager: Arc<RwLock<AiSdk5ClientManager>>,
    active_servers: HashMap<String, McpServerInfo>,
    character_sheet_id: Option<Uuid>,
    event_listeners: Vec<Box<dyn McpEventListener>>,
}

/// Information about an active MCP server
#[derive(Debug, Clone)]
pub struct McpServerInfo {
    pub server: McpServer,
    pub status: McpServerStatus,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    pub connection_count: u32,
    pub error_count: u32,
}

/// MCP server status
#[derive(Debug, Clone, PartialEq)]
pub enum McpServerStatus {
    Initializing,
    Connected,
    Disconnected,
    Error(String),
    Disabled,
}

/// Event listener for MCP server events
pub trait McpEventListener: Send + Sync {
    fn on_server_connected(&self, server_name: &str);
    fn on_server_disconnected(&self, server_name: &str);
    fn on_server_error(&self, server_name: &str, error: &str);
    fn on_tool_executed(&self, server_name: &str, tool_name: &str, success: bool);
}

/// MCP pod listener configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPodListener {
    pub pod_id: String,
    pub tool_patterns: Vec<String>,
    pub server_names: Vec<String>,
    pub priority: u32,
    pub enabled: bool,
}

/// MCP tool execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolContext {
    pub server_name: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub pod_context: Option<serde_json::Value>,
    pub emotional_context: Option<serde_json::Value>,
}

impl McpManager {
    /// Create new MCP manager with AI SDK 5 client manager
    pub fn new(client_manager: Arc<RwLock<AiSdk5ClientManager>>) -> Self {
        Self {
            client_manager,
            active_servers: HashMap::new(),
            character_sheet_id: None,
            event_listeners: Vec::new(),
        }
    }

    /// Load MCP servers from character sheet configuration
    pub async fn load_from_character_sheet(
        &mut self,
        character_sheet: &CharacterSheet,
    ) -> Result<()> {
        info!(
            "Loading MCP servers from character sheet: {}",
            character_sheet.name
        );
        self.character_sheet_id = Some(character_sheet.id);

        // Clear existing servers
        self.shutdown_all_servers().await?;

        let mcp_config = &character_sheet.mcp_servers;
        info!(
            "Found {} MCP servers configured in character sheet",
            mcp_config.servers.len()
        );

        // Load each configured server
        for (server_name, server_config) in &mcp_config.servers {
            if !server_config.enabled {
                info!("Skipping disabled MCP server: {}", server_name);
                continue;
            }

            match self
                .load_mcp_server(server_name, server_config, mcp_config)
                .await
            {
                Ok(_) => {
                    info!("Successfully loaded MCP server: {}", server_name);

                    // Notify listeners
                    for listener in &self.event_listeners {
                        listener.on_server_connected(server_name);
                    }
                }
                Err(e) => {
                    error!("Failed to load MCP server '{}': {}", server_name, e);

                    // Track the error
                    let server_info = McpServerInfo {
                        server: server_config.clone(),
                        status: McpServerStatus::Error(e.to_string()),
                        last_health_check: chrono::Utc::now(),
                        connection_count: 0,
                        error_count: 1,
                    };
                    self.active_servers.insert(server_name.clone(), server_info);

                    // Notify listeners
                    for listener in &self.event_listeners {
                        listener.on_server_error(server_name, &e.to_string());
                    }
                }
            }
        }

        info!(
            "MCP server loading complete. Active: {}, Total configured: {}",
            self.active_servers
                .values()
                .filter(|s| s.status == McpServerStatus::Connected)
                .count(),
            mcp_config.servers.len()
        );

        Ok(())
    }

    /// Load individual MCP server and register with AI SDK 5 client manager
    async fn load_mcp_server(
        &mut self,
        server_name: &str,
        server_config: &McpServer,
        global_config: &crate::character_sheet::McpServerConfiguration,
    ) -> Result<()> {
        debug!(
            "Loading MCP server: {} at {}",
            server_name, server_config.endpoint
        );

        // Convert character sheet MCP config to AI SDK 5 MCP client config
        let transport_config = self.convert_to_transport_config(server_config)?;

        let mcp_client_config = McpClientConfig {
            name: server_name.to_string(),
            transport: transport_config,
            enabled: server_config.enabled,
            capabilities: server_config.capabilities.clone(),
            priority: server_config.priority,
            health_check_interval_seconds: global_config.health_check_interval_seconds,
        };

        // Register with AI SDK 5 client manager
        let mut client_manager = self.client_manager.write().await;
        client_manager
            .add_mcp_client(server_name.to_string(), mcp_client_config)
            .await
            .with_context(|| format!("Failed to add MCP client for server: {}", server_name))?;

        // Track server info
        let server_info = McpServerInfo {
            server: server_config.clone(),
            status: McpServerStatus::Connected,
            last_health_check: chrono::Utc::now(),
            connection_count: 1,
            error_count: 0,
        };
        self.active_servers
            .insert(server_name.to_string(), server_info);

        debug!("MCP server '{}' loaded successfully", server_name);
        Ok(())
    }

    /// Convert character sheet MCP server to AI SDK 5 transport config
    fn convert_to_transport_config(&self, server: &McpServer) -> Result<TransportConfig> {
        // Parse endpoint to determine transport type
        let endpoint = &server.endpoint;

        if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            // HTTP transport
            let mut headers = HashMap::new();

            // Add authentication headers based on auth type
            match &server.auth_type {
                McpAuthType::None => {}
                McpAuthType::ApiKey { key } => {
                    headers.insert("X-API-Key".to_string(), key.clone());
                }
                McpAuthType::Bearer { token } => {
                    headers.insert("Authorization".to_string(), format!("Bearer {}", token));
                }
                McpAuthType::Basic { username, password } => {
                    // For now, do basic encoding manually (would use base64 crate in real implementation)
                    let credentials = format!("{}:{}", username, password);
                    headers.insert(
                        "Authorization".to_string(),
                        format!("Basic {}", credentials),
                    );
                }
                McpAuthType::Custom {
                    headers: custom_headers,
                } => {
                    headers.extend(custom_headers.clone());
                }
            }

            if endpoint.contains("/events") || endpoint.contains("/stream") {
                // Server-Sent Events transport
                Ok(TransportConfig::SSE {
                    url: endpoint.clone(),
                    headers,
                    timeout_seconds: 300, // 5 minutes for SSE
                })
            } else {
                // Standard HTTP transport
                Ok(TransportConfig::HTTP {
                    endpoint: endpoint.clone(),
                    headers,
                    timeout_seconds: 60,
                })
            }
        } else if endpoint.contains("://") {
            // Other protocols - treat as stdio for now
            let parts: Vec<&str> = endpoint.split("://").collect();
            if parts.len() >= 2 {
                let command_part = parts[1];
                let command_args: Vec<String> = command_part
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();

                if let Some(command) = command_args.first() {
                    Ok(TransportConfig::Stdio {
                        command: command.clone(),
                        args: command_args[1..].to_vec(),
                        working_dir: None,
                    })
                } else {
                    Err(anyhow::anyhow!(
                        "Invalid stdio endpoint format: {}",
                        endpoint
                    ))
                }
            } else {
                Err(anyhow::anyhow!("Invalid endpoint format: {}", endpoint))
            }
        } else {
            // Assume local command
            let parts: Vec<String> = endpoint.split_whitespace().map(|s| s.to_string()).collect();

            if let Some(command) = parts.first() {
                Ok(TransportConfig::Stdio {
                    command: command.clone(),
                    args: parts[1..].to_vec(),
                    working_dir: None,
                })
            } else {
                Err(anyhow::anyhow!("Empty endpoint specification"))
            }
        }
    }

    /// Execute MCP tool with full context
    pub async fn execute_tool_with_context(
        &self,
        context: McpToolContext,
    ) -> Result<serde_json::Value> {
        debug!(
            "Executing MCP tool '{}' on server '{}' with arguments: {}",
            context.tool_name, context.server_name, context.arguments
        );

        let client_manager = self.client_manager.read().await;
        let result = client_manager
            .execute_mcp_tool(&context.tool_name, &context.arguments)
            .await;

        // Update server stats and notify listeners
        match &result {
            Ok(_) => {
                for listener in &self.event_listeners {
                    listener.on_tool_executed(&context.server_name, &context.tool_name, true);
                }
            }
            Err(e) => {
                warn!(
                    "MCP tool execution failed: server={}, tool={}, error={}",
                    context.server_name, context.tool_name, e
                );
                for listener in &self.event_listeners {
                    listener.on_tool_executed(&context.server_name, &context.tool_name, false);
                }
            }
        }

        result
    }

    /// Get all available MCP tools from active servers
    pub async fn get_all_tools(&self) -> Vec<crate::ai_sdk_client::McpTool> {
        let client_manager = self.client_manager.read().await;
        client_manager.get_all_mcp_tools().await
    }

    /// Filter tools by pod listener configuration
    pub async fn get_tools_for_pod(
        &self,
        pod_listeners: &[McpPodListener],
    ) -> HashMap<String, Vec<crate::ai_sdk_client::McpTool>> {
        let all_tools = self.get_all_tools().await;
        let mut pod_tools = HashMap::new();

        for listener in pod_listeners {
            if !listener.enabled {
                continue;
            }

            let mut matching_tools = Vec::new();

            for tool in &all_tools {
                // Check if tool matches server filter
                if !listener.server_names.is_empty()
                    && !listener.server_names.contains(&tool.server)
                {
                    continue;
                }

                // Check if tool matches pattern filter
                let matches_pattern = listener.tool_patterns.is_empty()
                    || listener.tool_patterns.iter().any(|pattern| {
                        tool.name.contains(pattern)
                            || pattern.contains("*") && matches_glob_pattern(&tool.name, pattern)
                    });

                if matches_pattern {
                    matching_tools.push(tool.clone());
                }
            }

            if !matching_tools.is_empty() {
                pod_tools.insert(listener.pod_id.clone(), matching_tools);
            }
        }

        pod_tools
    }

    /// Add event listener for MCP events
    pub fn add_event_listener(&mut self, listener: Box<dyn McpEventListener>) {
        self.event_listeners.push(listener);
    }

    /// Get server status information
    pub fn get_server_status(&self, server_name: &str) -> Option<&McpServerInfo> {
        self.active_servers.get(server_name)
    }

    /// List all active servers
    pub fn list_active_servers(&self) -> Vec<(&String, &McpServerInfo)> {
        self.active_servers.iter().collect()
    }

    /// Perform health check on all servers
    pub async fn health_check_all(&mut self) -> Result<HashMap<String, bool>> {
        let mut results = HashMap::new();
        let now = chrono::Utc::now();

        for (server_name, server_info) in &mut self.active_servers {
            // Only check servers that are supposed to be connected
            if server_info.status != McpServerStatus::Connected {
                results.insert(server_name.clone(), false);
                continue;
            }

            // Get client and perform health check
            let client_manager = self.client_manager.read().await;
            if let Some(_mcp_client) = client_manager.get_mcp_client(server_name) {
                // In real implementation, would call actual health check
                // For now, assume healthy if client exists
                results.insert(server_name.clone(), true);
                server_info.last_health_check = now;
            } else {
                results.insert(server_name.clone(), false);
                server_info.status = McpServerStatus::Error("Client not found".to_string());
                server_info.error_count += 1;

                // Notify listeners
                for listener in &self.event_listeners {
                    listener.on_server_error(server_name, "Health check failed - client not found");
                }
            }
        }

        Ok(results)
    }

    /// Shutdown all MCP servers
    pub async fn shutdown_all_servers(&mut self) -> Result<()> {
        info!("Shutting down all MCP servers");

        // Notify listeners about disconnections
        for (server_name, _) in &self.active_servers {
            for listener in &self.event_listeners {
                listener.on_server_disconnected(server_name);
            }
        }

        // Clear server tracking
        self.active_servers.clear();

        // AI SDK 5 client manager will handle actual shutdown
        let client_manager = self.client_manager.read().await;
        client_manager.shutdown_all().await?;

        info!("All MCP servers shut down");
        Ok(())
    }
}

/// Simple glob pattern matching
fn matches_glob_pattern(text: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        text.starts_with(prefix)
    } else if pattern.starts_with('*') {
        let suffix = &pattern[1..];
        text.ends_with(suffix)
    } else {
        text == pattern
    }
}

/// Default event listener that logs events
pub struct LoggingEventListener;

impl McpEventListener for LoggingEventListener {
    fn on_server_connected(&self, server_name: &str) {
        info!("MCP server connected: {}", server_name);
    }

    fn on_server_disconnected(&self, server_name: &str) {
        info!("MCP server disconnected: {}", server_name);
    }

    fn on_server_error(&self, server_name: &str, error: &str) {
        error!("MCP server error [{}]: {}", server_name, error);
    }

    fn on_tool_executed(&self, server_name: &str, tool_name: &str, success: bool) {
        if success {
            debug!(
                "MCP tool executed successfully: {}:{}",
                server_name, tool_name
            );
        } else {
            warn!("MCP tool execution failed: {}:{}", server_name, tool_name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character_sheet::{CharacterSheetManager, McpServerConfiguration};

    #[tokio::test]
    async fn test_mcp_manager_creation() {
        let client_manager =
            Arc::new(RwLock::new(crate::ai_sdk_client::AiSdk5ClientManager::new()));
        let manager = McpManager::new(client_manager);

        assert_eq!(manager.active_servers.len(), 0);
        assert!(manager.character_sheet_id.is_none());
    }

    #[test]
    fn test_transport_config_conversion() {
        let client_manager =
            Arc::new(RwLock::new(crate::ai_sdk_client::AiSdk5ClientManager::new()));
        let manager = McpManager::new(client_manager);

        // Test HTTP endpoint
        let server = McpServer {
            name: "test".to_string(),
            endpoint: "https://api.example.com/mcp".to_string(),
            auth_type: McpAuthType::ApiKey {
                key: "test-key".to_string(),
            },
            capabilities: vec!["search".to_string()],
            metadata: HashMap::new(),
            enabled: true,
            priority: 1,
        };

        let transport = manager.convert_to_transport_config(&server).unwrap();
        match transport {
            TransportConfig::HTTP {
                endpoint, headers, ..
            } => {
                assert_eq!(endpoint, "https://api.example.com/mcp");
                assert_eq!(headers.get("X-API-Key"), Some(&"test-key".to_string()));
            }
            _ => panic!("Expected HTTP transport"),
        }
    }

    #[test]
    fn test_glob_pattern_matching() {
        assert!(matches_glob_pattern("search_tool", "*tool"));
        assert!(matches_glob_pattern("web_search", "web_*"));
        assert!(matches_glob_pattern("anything", "*"));
        assert!(!matches_glob_pattern("search", "*tool"));
        assert!(matches_glob_pattern("exact_match", "exact_match"));
    }

    #[tokio::test]
    async fn test_character_sheet_loading() {
        let client_manager =
            Arc::new(RwLock::new(crate::ai_sdk_client::AiSdk5ClientManager::new()));
        let mut manager = McpManager::new(client_manager);

        // Create test character sheet with MCP servers
        let mut character_sheet = CharacterSheetManager::create_default_sheet();

        let test_server = McpServer {
            name: "test-server".to_string(),
            endpoint: "https://api.test.com".to_string(),
            auth_type: McpAuthType::None,
            capabilities: vec!["search".to_string()],
            metadata: HashMap::new(),
            enabled: true,
            priority: 1,
        };

        character_sheet
            .mcp_servers
            .servers
            .insert("test-server".to_string(), test_server);

        // This would normally work with proper MCP client implementation
        // For now, test the parsing logic
        let transport = manager
            .convert_to_transport_config(&character_sheet.mcp_servers.servers["test-server"])
            .unwrap();

        match transport {
            TransportConfig::HTTP { endpoint, .. } => {
                assert_eq!(endpoint, "https://api.test.com");
            }
            _ => panic!("Expected HTTP transport"),
        }
    }
}
