//! Actions for Jito MEV pod

use async_trait::async_trait;
use banshee_core::{
    action::{
        Action, ActionConfig, ActionExample, ActionRequest, ActionResult, EmotionalImpact,
        SideEffect,
    },
    emotion::Emotion,
    Context, Result,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;

use crate::config::JitoMevConfig;

/// Action to submit MEV bundle
pub struct SubmitBundleAction {
    config: JitoMevConfig,
    action_config: ActionConfig,
}

impl SubmitBundleAction {
    pub fn new(config: JitoMevConfig) -> Self {
        let action_config = ActionConfig {
            name: "submit_mev_bundle".to_string(),
            description: "Submit MEV bundle to Jito block engine".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "bundle_type": {
                        "type": "string",
                        "enum": ["arbitrage", "liquidation", "sandwich"]
                    },
                    "transactions": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "estimated_profit_sol": { "type": "number" },
                    "tip_sol": { "type": "number" }
                },
                "required": ["bundle_type", "transactions"]
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "bundle_id": { "type": "string" },
                    "submitted": { "type": "boolean" },
                    "profit_sol": { "type": "number" }
                }
            })),
            has_side_effects: true,
            emotional_impact: Some(EmotionalImpact {
                on_success: [("Joy".to_string(), 0.6), ("Pride".to_string(), 0.4)]
                    .iter()
                    .cloned()
                    .collect(),
                on_failure: [
                    ("Disappointment".to_string(), 0.5),
                    ("Distress".to_string(), 0.3),
                ]
                .iter()
                .cloned()
                .collect(),
                intensity_multiplier: 1.2,
            }),
            settings: HashMap::new(),
        };

        Self {
            config,
            action_config,
        }
    }
}

#[async_trait]
impl Action for SubmitBundleAction {
    fn name(&self) -> &str {
        &self.action_config.name
    }

    fn description(&self) -> &str {
        &self.action_config.description
    }

    fn config(&self) -> &ActionConfig {
        &self.action_config
    }

    async fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        parameters
            .get("bundle_type")
            .and_then(|v| v.as_str())
            .ok_or("Missing bundle_type parameter")?;

        parameters
            .get("transactions")
            .and_then(|v| v.as_array())
            .ok_or("Missing transactions array")?;

        Ok(())
    }

    async fn is_available(&self, context: &Context) -> Result<bool> {
        // Check if we have sufficient confidence and not overly greedy
        if let Some(state) = Some(&context.emotional_state) {
            let joy_level = state.emotions.get(&Emotion::Joy).unwrap_or(&0.0);
            let distress_level = state.emotions.get(&Emotion::Distress).unwrap_or(&0.0);

            // Only submit bundles when positive emotional state
            Ok(*joy_level > 0.4
                && *distress_level < self.config.emotional_trading.greed_threshold as f32)
        } else {
            Ok(true)
        }
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Submit arbitrage bundle".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("bundle_type".to_string(), json!("arbitrage"));
                params.insert(
                    "transactions".to_string(),
                    json!(["tx1_base58", "tx2_base58"]),
                );
                params.insert("estimated_profit_sol".to_string(), json!(1.5));
                params.insert("tip_sol".to_string(), json!(0.3));
                params
            },
            expected_output: json!({
                "bundle_id": "bundle_123",
                "submitted": true,
                "profit_sol": 1.2
            }),
        }]
    }

    async fn execute(&self, request: ActionRequest) -> Result<ActionResult> {
        let bundle_type = request
            .parameters
            .get("bundle_type")
            .and_then(|v| v.as_str())
            .ok_or("Invalid bundle_type")?;

        let transactions = request
            .parameters
            .get("transactions")
            .and_then(|v| v.as_array())
            .ok_or("Invalid transactions")?;

        let estimated_profit = request
            .parameters
            .get("estimated_profit_sol")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let tip_sol = request
            .parameters
            .get("tip_sol")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.1);

        info!(
            "Submitting {} bundle with {} transactions, estimated profit: {} SOL",
            bundle_type,
            transactions.len(),
            estimated_profit
        );

        // Check if agent is initialized
        if !crate::ffi::is_agent_initialized() {
            // Initialize agent if needed
            let config = crate::ffi::SolanaAgentConfig {
                private_key: self.config.auth_keypair.clone()
                    .unwrap_or_else(|| "".to_string()),
                rpc_url: match self.config.network {
                    crate::config::NetworkType::MainnetBeta => "https://api.mainnet-beta.solana.com".to_string(),
                    crate::config::NetworkType::Devnet => "https://api.devnet.solana.com".to_string(),
                },
                openai_api_key: None,
            };
            
            crate::ffi::initialize_agent(&config)
                .map_err(|e| crate::error::JitoError::FfiError(format!("Failed to initialize agent: {}", e)))?;
        }

        // Prepare MEV bundle options
        let bundle_options = crate::ffi::MevBundleOptions {
            bundle_type: bundle_type.to_string(),
            transactions: transactions.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect(),
            tip_sol,
        };

        // Submit bundle via FFI
        let result = crate::ffi::submit_mev_bundle(&bundle_options)
            .map_err(|e| crate::error::JitoError::FfiError(format!("Failed to submit MEV bundle: {}", e)))?;

        let success = result.success;
        let bundle_id = result.signature.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let mut side_effects = vec![];

        if success {
            side_effects.push(SideEffect::EmotionalUpdate {
                emotions: [("Joy".to_string(), 0.6), ("Pride".to_string(), 0.4)]
                    .iter()
                    .cloned()
                    .collect(),
                reason: format!(
                    "Successfully submitted MEV bundle with {} SOL profit",
                    estimated_profit
                ),
            });
        } else {
            side_effects.push(SideEffect::EmotionalUpdate {
                emotions: [
                    ("Disappointment".to_string(), 0.5),
                    ("Distress".to_string(), 0.3),
                ]
                .iter()
                .cloned()
                .collect(),
                reason: "Failed to submit profitable MEV bundle".to_string(),
            });
        }

        Ok(ActionResult {
            success,
            data: json!({
                "bundle_id": bundle_id,
                "submitted": success,
                "profit_sol": if success { estimated_profit * 0.8 } else { 0.0 },
                "tip_sol": tip_sol,
                "bundle_type": bundle_type,
            }),
            error: if !success {
                Some("Bundle submission failed".to_string())
            } else {
                None
            },
            side_effects,
            metadata: HashMap::new(),
        })
    }
}

/// Action to scan for MEV opportunities
pub struct ScanMevAction {
    config: JitoMevConfig,
    action_config: ActionConfig,
}

impl ScanMevAction {
    pub fn new(config: JitoMevConfig) -> Self {
        let action_config = ActionConfig {
            name: "scan_mev_opportunities".to_string(),
            description: "Scan blockchain for MEV opportunities".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "scan_type": {
                        "type": "string",
                        "enum": ["all", "arbitrage", "liquidation", "sandwich"]
                    },
                    "min_profit_sol": { "type": "number" }
                }
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "opportunities": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": { "type": "string" },
                                "estimated_profit": { "type": "number" },
                                "confidence": { "type": "number" }
                            }
                        }
                    }
                }
            })),
            has_side_effects: false,
            emotional_impact: Some(EmotionalImpact {
                on_success: [("Hope".to_string(), 0.4), ("Joy".to_string(), 0.3)]
                    .iter()
                    .cloned()
                    .collect(),
                on_failure: [("Disappointment".to_string(), 0.2)]
                    .iter()
                    .cloned()
                    .collect(),
                intensity_multiplier: 0.8,
            }),
            settings: HashMap::new(),
        };

        Self {
            config,
            action_config,
        }
    }
}

#[async_trait]
impl Action for ScanMevAction {
    fn name(&self) -> &str {
        &self.action_config.name
    }

    fn description(&self) -> &str {
        &self.action_config.description
    }

    fn config(&self) -> &ActionConfig {
        &self.action_config
    }

    async fn validate(&self, _parameters: &HashMap<String, Value>) -> Result<()> {
        Ok(())
    }

    async fn is_available(&self, context: &Context) -> Result<bool> {
        // Available when not too distressed
        if let Some(state) = Some(&context.emotional_state) {
            let distress = state.emotions.get(&Emotion::Distress).unwrap_or(&0.0);
            Ok(*distress < 0.7)
        } else {
            Ok(true)
        }
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Scan for all MEV opportunities".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("scan_type".to_string(), json!("all"));
                params.insert("min_profit_sol".to_string(), json!(0.5));
                params
            },
            expected_output: json!({
                "opportunities": [
                    {
                        "type": "arbitrage",
                        "estimated_profit": 1.2,
                        "confidence": 0.8
                    }
                ]
            }),
        }]
    }

    async fn execute(&self, request: ActionRequest) -> Result<ActionResult> {
        let scan_type = request
            .parameters
            .get("scan_type")
            .and_then(|v| v.as_str())
            .unwrap_or("all");

        let min_profit = request
            .parameters
            .get("min_profit_sol")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.1);

        info!(
            "Scanning for {} MEV opportunities with min profit {} SOL",
            scan_type, min_profit
        );

        // Check if agent is initialized
        if !crate::ffi::is_agent_initialized() {
            // Initialize agent if needed
            let config = crate::ffi::SolanaAgentConfig {
                private_key: self.config.auth_keypair.clone()
                    .unwrap_or_else(|| "".to_string()),
                rpc_url: match self.config.network {
                    crate::config::NetworkType::MainnetBeta => "https://api.mainnet-beta.solana.com".to_string(),
                    crate::config::NetworkType::Devnet => "https://api.devnet.solana.com".to_string(),
                },
                openai_api_key: None,
            };
            
            crate::ffi::initialize_agent(&config)
                .map_err(|e| crate::error::JitoError::FfiError(format!("Failed to initialize agent: {}", e)))?;
        }

        // Prepare scan options
        let scan_options = crate::ffi::MevScanOptions {
            scan_type: scan_type.to_string(),
            min_profit_sol: min_profit,
        };

        // Scan for opportunities via FFI
        let result = crate::ffi::scan_mev_opportunities(&scan_options)
            .map_err(|e| crate::error::JitoError::FfiError(format!("Failed to scan MEV opportunities: {}", e)))?;

        let success = result.success;
        let opportunities = result.data.as_ref()
            .and_then(|d| d.get("opportunities"))
            .and_then(|o| o.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(ActionResult {
            success,
            data: result.data.unwrap_or_else(|| json!({
                "opportunities": opportunities,
                "scan_type": scan_type,
                "total_found": opportunities.len()
            })),
            error: result.error,
            side_effects: vec![],
            metadata: HashMap::new(),
        })
    }
}

/// Action to optimize staking via TipRouter
pub struct OptimizeStakingAction {
    config: JitoMevConfig,
    action_config: ActionConfig,
}

impl OptimizeStakingAction {
    pub fn new(config: JitoMevConfig) -> Self {
        let action_config = ActionConfig {
            name: "optimize_staking".to_string(),
            description: "Optimize staking allocation using TipRouter".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "amount_sol": { "type": "number" },
                    "strategy": {
                        "type": "string",
                        "enum": ["maximize_yield", "minimize_risk", "balanced"]
                    }
                }
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "validators": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "address": { "type": "string" },
                                "allocation": { "type": "number" },
                                "expected_apy": { "type": "number" }
                            }
                        }
                    },
                    "total_expected_apy": { "type": "number" }
                }
            })),
            has_side_effects: true,
            emotional_impact: Some(EmotionalImpact {
                on_success: [("Satisfaction".to_string(), 0.5), ("Hope".to_string(), 0.4)]
                    .iter()
                    .cloned()
                    .collect(),
                on_failure: [("Disappointment".to_string(), 0.3)]
                    .iter()
                    .cloned()
                    .collect(),
                intensity_multiplier: 0.9,
            }),
            settings: HashMap::new(),
        };

        Self {
            config,
            action_config,
        }
    }
}

#[async_trait]
impl Action for OptimizeStakingAction {
    fn name(&self) -> &str {
        &self.action_config.name
    }

    fn description(&self) -> &str {
        &self.action_config.description
    }

    fn config(&self) -> &ActionConfig {
        &self.action_config
    }

    async fn validate(&self, parameters: &HashMap<String, Value>) -> Result<()> {
        let amount = parameters
            .get("amount_sol")
            .and_then(|v| v.as_f64())
            .ok_or("Missing or invalid amount_sol")?;

        if amount <= 0.0 {
            return Err("Amount must be positive".into());
        }

        Ok(())
    }

    async fn is_available(&self, _context: &Context) -> Result<bool> {
        Ok(true)
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Optimize staking for maximum yield".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("amount_sol".to_string(), json!(100.0));
                params.insert("strategy".to_string(), json!("maximize_yield"));
                params
            },
            expected_output: json!({
                "validators": [
                    {
                        "address": "validator1...",
                        "allocation": 60.0,
                        "expected_apy": 8.5
                    },
                    {
                        "address": "validator2...",
                        "allocation": 40.0,
                        "expected_apy": 7.8
                    }
                ],
                "total_expected_apy": 8.22
            }),
        }]
    }

    async fn execute(&self, request: ActionRequest) -> Result<ActionResult> {
        let amount = request
            .parameters
            .get("amount_sol")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let strategy = request
            .parameters
            .get("strategy")
            .and_then(|v| v.as_str())
            .unwrap_or("balanced");

        info!(
            "Optimizing staking for {} SOL with {} strategy",
            amount, strategy
        );

        // Check if agent is initialized
        if !crate::ffi::is_agent_initialized() {
            // Initialize agent if needed
            let config = crate::ffi::SolanaAgentConfig {
                private_key: self.config.auth_keypair.clone()
                    .unwrap_or_else(|| "".to_string()),
                rpc_url: match self.config.network {
                    crate::config::NetworkType::MainnetBeta => "https://api.mainnet-beta.solana.com".to_string(),
                    crate::config::NetworkType::Devnet => "https://api.devnet.solana.com".to_string(),
                },
                openai_api_key: None,
            };
            
            crate::ffi::initialize_agent(&config)
                .map_err(|e| crate::error::JitoError::FfiError(format!("Failed to initialize agent: {}", e)))?;
        }

        // Prepare optimization options
        let optimization_options = crate::ffi::StakingOptimizationOptions {
            amount_sol: amount,
            strategy: strategy.to_string(),
        };

        // Optimize staking via FFI
        let result = crate::ffi::optimize_staking(&optimization_options)
            .map_err(|e| crate::error::JitoError::FfiError(format!("Failed to optimize staking: {}", e)))?;

        let validators = result.data.as_ref()
            .and_then(|d| d.get("validators"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let total_apy = result.data.as_ref()
            .and_then(|d| d.get("totalExpectedApy"))
            .and_then(|a| a.as_f64())
            .unwrap_or(0.0);

        Ok(ActionResult {
            success: result.success,
            data: result.data.unwrap_or_else(|| json!({
                "validators": validators,
                "total_expected_apy": total_apy,
                "strategy": strategy,
                "amount_sol": amount
            })),
            error: result.error,
            side_effects: vec![SideEffect::LogEvent {
                level: "info".to_string(),
                message: format!("Optimized staking allocation for {} SOL", amount),
                data: HashMap::new(),
            }],
            metadata: HashMap::new(),
        })
    }
}
