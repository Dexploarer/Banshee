//! Actions for Pump.fun pod

use async_trait::async_trait;
use banshee_core::{
    action::{
        Action, ActionConfig, ActionExample, ActionRequest, ActionResult, EmotionalImpact,
        SideEffect,
    },
    Context, Result,
};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{config::PumpFunConfig, ffi};

/// Action to create a new token on Pump.fun
pub struct CreateTokenAction {
    pump_config: PumpFunConfig,
    action_config: ActionConfig,
}

impl CreateTokenAction {
    pub fn new(pump_config: PumpFunConfig) -> Self {
        let action_config = ActionConfig {
            name: "create_pump_token".to_string(),
            description: "Create a new token on Pump.fun with bonding curve".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "metadata": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "symbol": { "type": "string" },
                            "description": { "type": "string" },
                            "image_url": { "type": "string" }
                        },
                        "required": ["name", "symbol"]
                    },
                    "initial_buy_amount": { "type": "number" }
                },
                "required": ["metadata"]
            }),
            output_schema: None,
            has_side_effects: true,
            emotional_impact: Some(EmotionalImpact {
                on_success: HashMap::from([
                    ("excitement".to_string(), 0.8),
                    ("confidence".to_string(), 0.5),
                ]),
                on_failure: HashMap::from([
                    ("disappointment".to_string(), 0.6),
                    ("frustration".to_string(), 0.4),
                ]),
                intensity_multiplier: 1.0,
            }),
            settings: HashMap::new(),
        };
        Self {
            pump_config,
            action_config,
        }
    }
}

#[async_trait]
impl Action for CreateTokenAction {
    fn name(&self) -> &str {
        &self.action_config.name
    }

    fn description(&self) -> &str {
        &self.action_config.description
    }

    fn config(&self) -> &ActionConfig {
        &self.action_config
    }

    async fn execute(&self, request: ActionRequest) -> Result<ActionResult> {
        // Extract token metadata from request
        let metadata = request.parameters.get("metadata")
            .ok_or("Missing metadata in request")?;
        
        let name = metadata.get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing token name")?;
        
        let symbol = metadata.get("symbol")
            .and_then(|v| v.as_str())
            .ok_or("Missing token symbol")?;
        
        let _description = metadata.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let image_url = metadata.get("image_url")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Check if agent is initialized
        if !ffi::is_agent_initialized() {
            // Initialize agent if needed
            let config = ffi::SolanaAgentConfig {
                private_key: self.pump_config.wallet_private_key.clone()
                    .unwrap_or_else(|| "".to_string()),
                rpc_url: self.pump_config.rpc_endpoint.clone(),
                openai_api_key: None,
            };
            
            ffi::initialize_agent(&config)
                .map_err(|e| crate::error::PumpFunError::FfiError(format!("Failed to initialize agent: {}", e)))?;
        }

        // Create token deployment options
        let options = ffi::TokenDeployOptions {
            name: name.to_string(),
            symbol: symbol.to_string(),
            uri: image_url.to_string(),
            decimals: 9, // Standard SOL decimals
            initial_supply: 1_000_000_000, // 1 billion tokens
        };

        // Deploy token via FFI
        let result = ffi::deploy_token(&options)
            .map_err(|e| crate::error::PumpFunError::FfiError(format!("Failed to deploy token: {}", e)))?;

        if result.success {
            Ok(ActionResult {
                success: true,
                data: json!({
                    "token_mint": result.mint.unwrap_or_else(|| "pending".to_string()),
                    "signature": result.signature,
                    "status": "created",
                    "details": result.data
                }),
                error: None,
                side_effects: vec![SideEffect::LogEvent {
                    level: "info".to_string(),
                    message: format!("Created new Pump.fun token: {} ({})", name, symbol),
                    data: HashMap::new(),
                }],
                metadata: HashMap::new(),
            })
        } else {
            Err(crate::error::PumpFunError::TransactionFailed(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ).into())
        }
    }

    async fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        let metadata = parameters
            .get("metadata")
            .ok_or("Missing metadata parameter")?;

        metadata
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing or invalid token name")?;

        metadata
            .get("symbol")
            .and_then(|v| v.as_str())
            .ok_or("Missing or invalid token symbol")?;

        Ok(())
    }

    async fn is_available(&self, _context: &Context) -> Result<bool> {
        Ok(true)
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Create a meme token".to_string(),
            parameters: HashMap::from([
                (
                    "metadata".to_string(),
                    json!({
                        "name": "Banshee Coin",
                        "symbol": "BNSH",
                        "description": "The emotional AI trading token",
                        "image_url": "https://example.com/banshee.png"
                    }),
                ),
                ("initial_buy_amount".to_string(), json!(1.0)),
            ]),
            expected_output: json!({
                "token_mint": "11111111111111111111111111111111",
                "status": "created"
            }),
        }]
    }
}

/// Action to buy tokens on Pump.fun
pub struct BuyTokenAction {
    pump_config: PumpFunConfig,
    action_config: ActionConfig,
}

impl BuyTokenAction {
    pub fn new(pump_config: PumpFunConfig) -> Self {
        let action_config = ActionConfig {
            name: "buy_pump_token".to_string(),
            description: "Buy tokens on Pump.fun bonding curve".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "token_mint": { "type": "string" },
                    "sol_amount": { "type": "number" },
                    "min_tokens_out": { "type": "number" }
                },
                "required": ["token_mint", "sol_amount"]
            }),
            output_schema: None,
            has_side_effects: true,
            emotional_impact: Some(EmotionalImpact {
                on_success: HashMap::from([
                    ("satisfaction".to_string(), 0.6),
                    ("excitement".to_string(), 0.4),
                ]),
                on_failure: HashMap::from([
                    ("regret".to_string(), 0.5),
                    ("anxiety".to_string(), 0.3),
                ]),
                intensity_multiplier: 0.8,
            }),
            settings: HashMap::new(),
        };
        Self {
            pump_config,
            action_config,
        }
    }
}

#[async_trait]
impl Action for BuyTokenAction {
    fn name(&self) -> &str {
        &self.action_config.name
    }

    fn description(&self) -> &str {
        &self.action_config.description
    }

    fn config(&self) -> &ActionConfig {
        &self.action_config
    }

    async fn execute(&self, _request: ActionRequest) -> Result<ActionResult> {
        // Mock implementation
        Ok(ActionResult {
            success: true,
            data: json!({
                "tokens_bought": 1000000,
                "sol_spent": 0.5,
                "price_per_token": 0.0000005
            }),
            error: None,
            side_effects: vec![],
            metadata: HashMap::new(),
        })
    }

    async fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("token_mint")
            .and_then(|v| v.as_str())
            .ok_or("Missing or invalid token_mint")?;

        parameters
            .get("sol_amount")
            .and_then(|v| v.as_f64())
            .ok_or("Missing or invalid sol_amount")?;

        Ok(())
    }

    async fn is_available(&self, _context: &Context) -> Result<bool> {
        Ok(true)
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![]
    }
}

/// Action to sell tokens on Pump.fun
pub struct SellTokenAction {
    pump_config: PumpFunConfig,
    action_config: ActionConfig,
}

impl SellTokenAction {
    pub fn new(pump_config: PumpFunConfig) -> Self {
        let action_config = ActionConfig {
            name: "sell_pump_token".to_string(),
            description: "Sell tokens on Pump.fun bonding curve".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "token_mint": { "type": "string" },
                    "token_amount": { "type": "number" },
                    "min_sol_out": { "type": "number" }
                },
                "required": ["token_mint", "token_amount"]
            }),
            output_schema: None,
            has_side_effects: true,
            emotional_impact: Some(EmotionalImpact {
                on_success: HashMap::from([
                    ("relief".to_string(), 0.5),
                    ("satisfaction".to_string(), 0.4),
                ]),
                on_failure: HashMap::from([
                    ("frustration".to_string(), 0.6),
                    ("disappointment".to_string(), 0.4),
                ]),
                intensity_multiplier: 0.7,
            }),
            settings: HashMap::new(),
        };
        Self {
            pump_config,
            action_config,
        }
    }
}

#[async_trait]
impl Action for SellTokenAction {
    fn name(&self) -> &str {
        &self.action_config.name
    }

    fn description(&self) -> &str {
        &self.action_config.description
    }

    fn config(&self) -> &ActionConfig {
        &self.action_config
    }

    async fn execute(&self, _request: ActionRequest) -> Result<ActionResult> {
        // Mock implementation
        Ok(ActionResult {
            success: true,
            data: json!({
                "tokens_sold": 1000000,
                "sol_received": 0.6,
                "price_per_token": 0.0000006
            }),
            error: None,
            side_effects: vec![],
            metadata: HashMap::new(),
        })
    }

    async fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("token_mint")
            .and_then(|v| v.as_str())
            .ok_or("Missing or invalid token_mint")?;

        parameters
            .get("token_amount")
            .and_then(|v| v.as_f64())
            .ok_or("Missing or invalid token_amount")?;

        Ok(())
    }

    async fn is_available(&self, _context: &Context) -> Result<bool> {
        Ok(true)
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![]
    }
}
