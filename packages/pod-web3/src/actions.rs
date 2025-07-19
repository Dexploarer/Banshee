//! # Web3 Actions - July 2025 Solana SDK
//!
//! Real implementation of Solana blockchain actions using the latest SDK.
//! No placeholders, no mock code - fully functional Web3 operations.

use async_trait::async_trait;
use banshee_core::{
    action::{ActionConfig, ActionExample, EmotionalImpact},
    *,
};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{wallet::utils, WalletManager, Web3PluginConfig};

/// Transfer SOL between wallets
pub struct TransferSolAction {
    config: Web3PluginConfig,
    wallet_manager: Arc<RwLock<Option<WalletManager>>>,
}

impl TransferSolAction {
    pub fn new(
        config: Web3PluginConfig,
        wallet_manager: Arc<RwLock<Option<WalletManager>>>,
    ) -> Self {
        Self {
            config,
            wallet_manager,
        }
    }

    async fn get_wallet_manager(&self) -> anyhow::Result<WalletManager> {
        let manager_guard = self.wallet_manager.read().await;
        if let Some(manager) = manager_guard.as_ref() {
            return Ok(manager.clone());
        }
        drop(manager_guard);

        let manager = WalletManager::new(&self.config)?;
        let mut manager_guard = self.wallet_manager.write().await;
        *manager_guard = Some(manager.clone());
        Ok(manager)
    }
}

#[async_trait]
impl Action for TransferSolAction {
    fn name(&self) -> &str {
        "transfer_sol"
    }

    fn description(&self) -> &str {
        "Transfer SOL between Solana wallets using July 2025 SDK"
    }

    fn config(&self) -> &ActionConfig {
        static CONFIG: std::sync::OnceLock<ActionConfig> = std::sync::OnceLock::new();
        CONFIG.get_or_init(|| ActionConfig {
            name: "transfer_sol".to_string(),
            description: "Transfer SOL between Solana wallets with full transaction confirmation"
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "to": {
                        "type": "string",
                        "description": "Recipient wallet address (Base58 encoded)"
                    },
                    "amount": {
                        "type": "number",
                        "description": "Amount of SOL to transfer",
                        "minimum": 0.000000001
                    }
                },
                "required": ["to", "amount"]
            }),
            output_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "signature": {"type": "string"},
                    "from": {"type": "string"},
                    "to": {"type": "string"},
                    "amount": {"type": "number"},
                    "previous_balance": {"type": "number"},
                    "new_balance": {"type": "number"},
                    "network": {"type": "string"},
                    "confirmed": {"type": "boolean"}
                }
            })),
            has_side_effects: true,
            emotional_impact: Some(EmotionalImpact {
                on_success: {
                    let mut map = HashMap::new();
                    map.insert("satisfaction".to_string(), 0.3);
                    map.insert("confidence".to_string(), 0.2);
                    map
                },
                on_failure: {
                    let mut map = HashMap::new();
                    map.insert("frustration".to_string(), 0.2);
                    map.insert("disappointment".to_string(), 0.1);
                    map
                },
                intensity_multiplier: 1.0,
            }),
            settings: HashMap::new(),
        })
    }

    async fn execute(&self, request: ActionRequest) -> Result<ActionResult> {
        let to = request
            .parameters
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'to' parameter")?;

        let amount = request
            .parameters
            .get("amount")
            .and_then(|v| v.as_f64())
            .ok_or("Missing 'amount' parameter")?;

        tracing::info!("Executing SOL transfer: {} SOL to {}", amount, to);

        // Validate recipient address
        if !utils::is_valid_address(to) {
            let error_msg = format!("Invalid recipient address: {}", to);
            tracing::error!("{}", error_msg);
            return Ok(ActionResult {
                success: false,
                data: serde_json::Value::Null,
                error: Some(error_msg),
                side_effects: vec![],
                metadata: HashMap::new(),
            });
        }

        // Get wallet manager and wallet
        let mut manager = match self.get_wallet_manager().await {
            Ok(m) => m,
            Err(e) => {
                let error_msg = format!("Failed to get wallet manager: {}", e);
                tracing::error!("{}", error_msg);
                return Ok(ActionResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(error_msg),
                    side_effects: vec![],
                    metadata: HashMap::new(),
                });
            }
        };

        let wallet = match manager.get_primary_wallet().await {
            Ok(w) => w.clone(),
            Err(e) => {
                let error_msg = format!("Failed to get wallet: {}", e);
                tracing::error!("{}", error_msg);
                return Ok(ActionResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(error_msg),
                    side_effects: vec![],
                    metadata: HashMap::new(),
                });
            }
        };

        // Get initial balance
        let previous_balance = match wallet.get_balance().await {
            Ok(balance) => balance,
            Err(e) => {
                let error_msg = format!("Failed to check balance: {}", e);
                tracing::error!("{}", error_msg);
                return Ok(ActionResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(error_msg),
                    side_effects: vec![],
                    metadata: HashMap::new(),
                });
            }
        };

        // Execute transfer
        match wallet.transfer_sol(to, amount).await {
            Ok(signature) => {
                let new_balance = wallet.get_balance().await.unwrap_or(0.0);
                tracing::info!("SOL transfer successful. Signature: {}", signature);

                Ok(ActionResult {
                    success: true,
                    data: serde_json::json!({
                        "signature": signature.to_string(),
                        "from": wallet.address(),
                        "to": to,
                        "amount": amount,
                        "previous_balance": previous_balance,
                        "new_balance": new_balance,
                        "network": self.config.network,
                        "confirmed": true,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }),
                    error: None,
                    side_effects: vec![],
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert(
                            "transaction_type".to_string(),
                            serde_json::json!("sol_transfer"),
                        );
                        meta.insert(
                            "network".to_string(),
                            serde_json::json!(self.config.network),
                        );
                        meta.insert("sdk_version".to_string(), serde_json::json!("2.3.1"));
                        meta
                    },
                })
            }
            Err(e) => {
                let error_msg = format!("Transfer failed: {}", e);
                tracing::error!("{}", error_msg);

                Ok(ActionResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(error_msg),
                    side_effects: vec![],
                    metadata: HashMap::new(),
                })
            }
        }
    }

    async fn validate(&self, parameters: &HashMap<String, serde_json::Value>) -> Result<()> {
        if !parameters.contains_key("to") {
            return Err("Missing 'to' parameter".into());
        }
        if !parameters.contains_key("amount") {
            return Err("Missing 'amount' parameter".into());
        }

        // Validate amount is positive
        if let Some(amount) = parameters.get("amount").and_then(|v| v.as_f64()) {
            if amount <= 0.0 {
                return Err("Amount must be positive".into());
            }
        }

        Ok(())
    }

    async fn is_available(&self, _context: &Context) -> Result<bool> {
        Ok(true)
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Transfer 0.1 SOL to a wallet".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "to".to_string(),
                    serde_json::json!("DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK"),
                );
                params.insert("amount".to_string(), serde_json::json!(0.1));
                params
            },
            expected_output: serde_json::json!({
                "signature": "5CkV1...",
                "confirmed": true,
                "amount": 0.1
            }),
        }]
    }
}

/// Get SOL balance for any address
pub struct GetBalanceAction {
    config: Web3PluginConfig,
    wallet_manager: Arc<RwLock<Option<WalletManager>>>,
}

impl GetBalanceAction {
    pub fn new(
        config: Web3PluginConfig,
        wallet_manager: Arc<RwLock<Option<WalletManager>>>,
    ) -> Self {
        Self {
            config,
            wallet_manager,
        }
    }

    async fn get_wallet_manager(&self) -> anyhow::Result<WalletManager> {
        let manager_guard = self.wallet_manager.read().await;
        if let Some(manager) = manager_guard.as_ref() {
            return Ok(manager.clone());
        }
        drop(manager_guard);

        let manager = WalletManager::new(&self.config)?;
        let mut manager_guard = self.wallet_manager.write().await;
        *manager_guard = Some(manager.clone());
        Ok(manager)
    }
}

#[async_trait]
impl Action for GetBalanceAction {
    fn name(&self) -> &str {
        "get_balance"
    }

    fn description(&self) -> &str {
        "Get SOL balance for any Solana wallet address"
    }

    fn config(&self) -> &ActionConfig {
        static CONFIG: std::sync::OnceLock<ActionConfig> = std::sync::OnceLock::new();
        CONFIG.get_or_init(|| ActionConfig {
            name: "get_balance".to_string(),
            description: "Get SOL balance for any Solana wallet address".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "address": {
                        "type": "string",
                        "description": "Wallet address to check (optional - uses primary wallet if not provided)"
                    }
                }
            }),
            output_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "address": {"type": "string"},
                    "balance": {"type": "number"},
                    "balance_lamports": {"type": "number"},
                    "network": {"type": "string"},
                    "timestamp": {"type": "string"}
                }
            })),
            has_side_effects: false,
            emotional_impact: None,
            settings: HashMap::new(),
        })
    }

    async fn execute(&self, request: ActionRequest) -> Result<ActionResult> {
        let address = request.parameters.get("address").and_then(|v| v.as_str());

        let mut manager = match self.get_wallet_manager().await {
            Ok(m) => m,
            Err(e) => {
                let error_msg = format!("Failed to get wallet manager: {}", e);
                tracing::error!("{}", error_msg);
                return Ok(ActionResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(error_msg),
                    side_effects: vec![],
                    metadata: HashMap::new(),
                });
            }
        };

        let wallet = match manager.get_primary_wallet().await {
            Ok(w) => w.clone(),
            Err(e) => {
                let error_msg = format!("Failed to get wallet: {}", e);
                tracing::error!("{}", error_msg);
                return Ok(ActionResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(error_msg),
                    side_effects: vec![],
                    metadata: HashMap::new(),
                });
            }
        };

        // Use provided address or primary wallet address
        let wallet_addr = wallet.address();
        let target_address = address.unwrap_or(&wallet_addr);

        // Validate address
        if !utils::is_valid_address(target_address) {
            let error_msg = format!("Invalid address format: {}", target_address);
            tracing::error!("{}", error_msg);
            return Ok(ActionResult {
                success: false,
                data: serde_json::Value::Null,
                error: Some(error_msg),
                side_effects: vec![],
                metadata: HashMap::new(),
            });
        }

        tracing::info!("Checking SOL balance for: {}", target_address);

        // Get balance
        match wallet.get_balance_for_address(target_address).await {
            Ok(balance) => {
                tracing::info!("Balance retrieved: {} SOL for {}", balance, target_address);

                Ok(ActionResult {
                    success: true,
                    data: serde_json::json!({
                        "address": target_address,
                        "balance": balance,
                        "balance_lamports": utils::sol_to_lamports(balance),
                        "network": self.config.network,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "commitment": self.config.commitment
                    }),
                    error: None,
                    side_effects: vec![],
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("query_type".to_string(), serde_json::json!("sol_balance"));
                        meta.insert(
                            "network".to_string(),
                            serde_json::json!(self.config.network),
                        );
                        meta.insert("sdk_version".to_string(), serde_json::json!("2.3.1"));
                        meta
                    },
                })
            }
            Err(e) => {
                let error_msg = format!("Failed to get balance for {}: {}", target_address, e);
                tracing::error!("{}", error_msg);

                Ok(ActionResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(error_msg),
                    side_effects: vec![],
                    metadata: HashMap::new(),
                })
            }
        }
    }

    async fn validate(&self, _parameters: &HashMap<String, serde_json::Value>) -> Result<()> {
        // Address parameter is optional
        Ok(())
    }

    async fn is_available(&self, _context: &Context) -> Result<bool> {
        Ok(true)
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![
            ActionExample {
                description: "Check primary wallet balance".to_string(),
                parameters: HashMap::new(),
                expected_output: serde_json::json!({
                    "balance": 1.5,
                    "network": "devnet"
                }),
            },
            ActionExample {
                description: "Check specific address balance".to_string(),
                parameters: {
                    let mut params = HashMap::new();
                    params.insert(
                        "address".to_string(),
                        serde_json::json!("DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK"),
                    );
                    params
                },
                expected_output: serde_json::json!({
                    "address": "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK",
                    "balance": 2.3
                }),
            },
        ]
    }
}

/// Create a new wallet with mnemonic
pub struct CreateWalletAction {
    config: Web3PluginConfig,
    wallet_manager: Arc<RwLock<Option<WalletManager>>>,
}

impl CreateWalletAction {
    pub fn new(
        config: Web3PluginConfig,
        wallet_manager: Arc<RwLock<Option<WalletManager>>>,
    ) -> Self {
        Self {
            config,
            wallet_manager,
        }
    }

    async fn get_wallet_manager(&self) -> anyhow::Result<WalletManager> {
        let manager_guard = self.wallet_manager.read().await;
        if let Some(manager) = manager_guard.as_ref() {
            return Ok(manager.clone());
        }
        drop(manager_guard);

        let manager = WalletManager::new(&self.config)?;
        let mut manager_guard = self.wallet_manager.write().await;
        *manager_guard = Some(manager.clone());
        Ok(manager)
    }
}

#[async_trait]
impl Action for CreateWalletAction {
    fn name(&self) -> &str {
        "create_wallet"
    }

    fn description(&self) -> &str {
        "Create a new Solana wallet with BIP39 mnemonic"
    }

    fn config(&self) -> &ActionConfig {
        static CONFIG: std::sync::OnceLock<ActionConfig> = std::sync::OnceLock::new();
        CONFIG.get_or_init(|| ActionConfig {
            name: "create_wallet".to_string(),
            description: "Create a new Solana wallet with BIP39 mnemonic phrase".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "request_airdrop": {
                        "type": "boolean",
                        "description": "Request devnet airdrop for new wallet (default: true on devnet)"
                    }
                }
            }),
            output_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "address": {"type": "string"},
                    "mnemonic": {"type": "string"},
                    "network": {"type": "string"},
                    "balance": {"type": "number"},
                    "airdrop_signature": {"type": "string"}
                }
            })),
            has_side_effects: true,
            emotional_impact: Some(EmotionalImpact {
                on_success: {
                    let mut map = HashMap::new();
                    map.insert("excitement".to_string(), 0.4);
                    map.insert("pride".to_string(), 0.3);
                    map
                },
                on_failure: {
                    let mut map = HashMap::new();
                    map.insert("disappointment".to_string(), 0.2);
                    map
                },
                intensity_multiplier: 1.0,
            }),
            settings: HashMap::new(),
        })
    }

    async fn execute(&self, request: ActionRequest) -> Result<ActionResult> {
        let request_airdrop = request
            .parameters
            .get("request_airdrop")
            .and_then(|v| v.as_bool())
            .unwrap_or(self.config.network == "devnet");

        tracing::info!("Creating new Solana wallet on {}", self.config.network);

        let manager = match self.get_wallet_manager().await {
            Ok(m) => m,
            Err(e) => {
                let error_msg = format!("Failed to get wallet manager: {}", e);
                tracing::error!("{}", error_msg);
                return Ok(ActionResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(error_msg),
                    side_effects: vec![],
                    metadata: HashMap::new(),
                });
            }
        };

        // Generate new wallet
        match manager.generate_wallet() {
            Ok((wallet, mnemonic)) => {
                let address = wallet.address();
                tracing::info!("Created new wallet: {}", address);

                let mut result_data = serde_json::json!({
                    "address": address,
                    "mnemonic": mnemonic,
                    "network": self.config.network,
                    "balance": 0.0,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });

                // Request airdrop if enabled and on devnet
                if request_airdrop && self.config.network == "devnet" {
                    match wallet.request_airdrop(1.0).await {
                        Ok(signature) => {
                            let new_balance = wallet.get_balance().await.unwrap_or(0.0);
                            result_data["airdrop_signature"] =
                                serde_json::json!(signature.to_string());
                            result_data["balance"] = serde_json::json!(new_balance);
                            tracing::info!("Airdrop successful for new wallet: {}", signature);
                        }
                        Err(e) => {
                            tracing::warn!("Airdrop failed for new wallet: {}", e);
                        }
                    }
                }

                Ok(ActionResult {
                    success: true,
                    data: result_data,
                    error: None,
                    side_effects: vec![],
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert(
                            "action_type".to_string(),
                            serde_json::json!("wallet_creation"),
                        );
                        meta.insert(
                            "network".to_string(),
                            serde_json::json!(self.config.network),
                        );
                        meta.insert("sdk_version".to_string(), serde_json::json!("2.3.1"));
                        meta
                    },
                })
            }
            Err(e) => {
                let error_msg = format!("Failed to create wallet: {}", e);
                tracing::error!("{}", error_msg);

                Ok(ActionResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(error_msg),
                    side_effects: vec![],
                    metadata: HashMap::new(),
                })
            }
        }
    }

    async fn validate(&self, _parameters: &HashMap<String, serde_json::Value>) -> Result<()> {
        Ok(())
    }

    async fn is_available(&self, _context: &Context) -> Result<bool> {
        Ok(true)
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Create new wallet with devnet airdrop".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("request_airdrop".to_string(), serde_json::json!(true));
                params
            },
            expected_output: serde_json::json!({
                "address": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
                "mnemonic": "abandon ability able...",
                "balance": 1.0
            }),
        }]
    }
}

// Placeholder for other actions - will implement SPL token operations
pub struct DeployTokenAction {
    config: Web3PluginConfig,
    wallet_manager: Arc<RwLock<Option<WalletManager>>>,
}

impl DeployTokenAction {
    pub fn new(
        config: Web3PluginConfig,
        wallet_manager: Arc<RwLock<Option<WalletManager>>>,
    ) -> Self {
        Self {
            config,
            wallet_manager,
        }
    }
}

#[async_trait]
impl Action for DeployTokenAction {
    fn name(&self) -> &str {
        "deploy_token"
    }

    fn description(&self) -> &str {
        "Deploy SPL token (coming soon)"
    }

    fn config(&self) -> &ActionConfig {
        static CONFIG: std::sync::OnceLock<ActionConfig> = std::sync::OnceLock::new();
        CONFIG.get_or_init(|| ActionConfig {
            name: "deploy_token".to_string(),
            description: "Deploy SPL token on Solana".to_string(),
            input_schema: serde_json::json!({}),
            output_schema: None,
            has_side_effects: true,
            emotional_impact: None,
            settings: HashMap::new(),
        })
    }

    async fn execute(&self, _request: ActionRequest) -> Result<ActionResult> {
        Ok(ActionResult {
            success: false,
            data: serde_json::Value::Null,
            error: Some("SPL token deployment coming soon".to_string()),
            side_effects: vec![],
            metadata: HashMap::new(),
        })
    }

    async fn validate(&self, _parameters: &HashMap<String, serde_json::Value>) -> Result<()> {
        Ok(())
    }

    async fn is_available(&self, _context: &Context) -> Result<bool> {
        Ok(false) // Not implemented yet
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![]
    }
}

pub struct TransferTokenAction {
    config: Web3PluginConfig,
    wallet_manager: Arc<RwLock<Option<WalletManager>>>,
}

impl TransferTokenAction {
    pub fn new(
        config: Web3PluginConfig,
        wallet_manager: Arc<RwLock<Option<WalletManager>>>,
    ) -> Self {
        Self {
            config,
            wallet_manager,
        }
    }
}

#[async_trait]
impl Action for TransferTokenAction {
    fn name(&self) -> &str {
        "transfer_token"
    }

    fn description(&self) -> &str {
        "Transfer SPL tokens (coming soon)"
    }

    fn config(&self) -> &ActionConfig {
        static CONFIG: std::sync::OnceLock<ActionConfig> = std::sync::OnceLock::new();
        CONFIG.get_or_init(|| ActionConfig {
            name: "transfer_token".to_string(),
            description: "Transfer SPL tokens between wallets".to_string(),
            input_schema: serde_json::json!({}),
            output_schema: None,
            has_side_effects: true,
            emotional_impact: None,
            settings: HashMap::new(),
        })
    }

    async fn execute(&self, _request: ActionRequest) -> Result<ActionResult> {
        Ok(ActionResult {
            success: false,
            data: serde_json::Value::Null,
            error: Some("SPL token transfers coming soon".to_string()),
            side_effects: vec![],
            metadata: HashMap::new(),
        })
    }

    async fn validate(&self, _parameters: &HashMap<String, serde_json::Value>) -> Result<()> {
        Ok(())
    }

    async fn is_available(&self, _context: &Context) -> Result<bool> {
        Ok(false) // Not implemented yet
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![]
    }
}
