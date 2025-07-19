//! Actions for Pump.fun pod

use async_trait::async_trait;
use banshee_core::{
    Context, Result,
    action::{Action, ActionConfig, ActionExample, ActionRequest, ActionResult, EmotionalImpact, SideEffect},
};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{config::PumpFunConfig, types::*};

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
        Self { pump_config, action_config }
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
        // Mock implementation
        let token_mint = solana_sdk::pubkey::Pubkey::new_unique();
        
        Ok(ActionResult {
            success: true,
            data: json!({
                "token_mint": token_mint.to_string(),
                "status": "created"
            }),
            error: None,
            side_effects: vec![
                SideEffect::LogEvent {
                    level: "info".to_string(),
                    message: "Created new Pump.fun token".to_string(),
                    data: HashMap::new(),
                }
            ],
            metadata: HashMap::new(),
        })
    }

    async fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        let metadata = parameters.get("metadata")
            .ok_or("Missing metadata parameter")?;
        
        metadata.get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing or invalid token name")?;
        
        metadata.get("symbol")
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
                ("metadata".to_string(), json!({
                    "name": "Banshee Coin",
                    "symbol": "BNSH",
                    "description": "The emotional AI trading token",
                    "image_url": "https://example.com/banshee.png"
                })),
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
        Self { pump_config, action_config }
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

    async fn execute(&self, request: ActionRequest) -> Result<ActionResult> {
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
        parameters.get("token_mint")
            .and_then(|v| v.as_str())
            .ok_or("Missing or invalid token_mint")?;
        
        parameters.get("sol_amount")
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
        Self { pump_config, action_config }
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

    async fn execute(&self, request: ActionRequest) -> Result<ActionResult> {
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
        parameters.get("token_mint")
            .and_then(|v| v.as_str())
            .ok_or("Missing or invalid token_mint")?;
        
        parameters.get("token_amount")
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