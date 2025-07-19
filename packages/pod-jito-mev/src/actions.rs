//! Actions for Jito MEV pod

use async_trait::async_trait;
use banshee_core::{
    action::{Action, ActionConfig, ActionExample, ActionRequest, ActionResult, EmotionalImpact},
    emotion::{Emotion, EmotionalState},
};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

use crate::{
    bundle_builder::BundleBuilder, config::JitoMevConfig, tip_router::TipRouter, types::*,
};

/// Action to submit MEV bundle
pub struct SubmitBundleAction {
    config: JitoMevConfig,
}

impl SubmitBundleAction {
    pub fn new(config: JitoMevConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Action for SubmitBundleAction {
    fn name(&self) -> &str {
        "submit_mev_bundle"
    }

    fn description(&self) -> &str {
        "Submit MEV bundle to Jito block engine"
    }

    fn config(&self) -> ActionConfig {
        ActionConfig {
            name: self.name().to_string(),
            description: self.description().to_string(),
            enabled: true,
            similes: vec!["jito_submit", "mev_execute", "bundle_send"],
            examples: vec![],
            validates: Some(vec!["params.bundle_type", "params.transactions"]),
        }
    }

    fn validate(&self, params: &HashMap<String, Value>) -> Result<(), String> {
        params
            .get("bundle_type")
            .and_then(|v| v.as_str())
            .ok_or("Missing bundle_type parameter")?;

        params
            .get("transactions")
            .and_then(|v| v.as_array())
            .ok_or("Missing transactions array")?;

        Ok(())
    }

    fn is_available(&self, emotional_state: Option<&EmotionalState>) -> bool {
        if let Some(state) = emotional_state {
            // Only submit bundles when confident and not overly greedy
            let confidence = state.get_emotion_value(&Emotion::Confidence);
            let greed = state.get_emotion_value(&Emotion::Greed);

            confidence > 0.6 && greed < self.config.emotional_trading.greed_threshold
        } else {
            true
        }
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Submit arbitrage bundle".to_string(),
            request: ActionRequest {
                action: self.name().to_string(),
                params: json!({
                    "bundle_type": "arbitrage",
                    "transactions": ["tx1_base58", "tx2_base58"],
                    "estimated_profit_sol": 1.5,
                    "tip_sol": 0.3
                }),
            },
            expected_emotional_impact: EmotionalImpact {
                primary_emotion: Emotion::Excitement,
                intensity: 0.7,
                valence: 0.8,
            },
        }]
    }

    async fn execute(
        &self,
        request: ActionRequest,
    ) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let bundle_type = request
            .params
            .get("bundle_type")
            .and_then(|v| v.as_str())
            .ok_or("Invalid bundle_type")?;

        let estimated_profit = request
            .params
            .get("estimated_profit_sol")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Calculate dynamic tip if enabled
        let tip_sol = if self.config.tip_router.dynamic_tips {
            TipRouter::calculate_dynamic_tip(
                Decimal::from_f64(estimated_profit).unwrap_or_default(),
                self.config.tip_router.dynamic_tip_percentage,
                self.config.tip_router.min_tip_sol,
            )
        } else {
            request
                .params
                .get("tip_sol")
                .and_then(|v| v.as_f64())
                .and_then(|f| Decimal::from_f64(f))
                .unwrap_or(self.config.tip_router.min_tip_sol)
        };

        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), json!("submit_mev_bundle"));
        metadata.insert("bundle_type".to_string(), json!(bundle_type));
        metadata.insert(
            "bundle_id".to_string(),
            json!(uuid::Uuid::new_v4().to_string()),
        );
        metadata.insert("tip_sol".to_string(), json!(tip_sol.to_string()));
        metadata.insert("estimated_profit_sol".to_string(), json!(estimated_profit));
        metadata.insert("status".to_string(), json!("submitted"));

        Ok(ActionResult {
            success: true,
            message: format!("Submitted {} bundle with {} SOL tip", bundle_type, tip_sol),
            metadata,
            side_effects: vec![],
        })
    }
}

/// Action to scan for MEV opportunities
pub struct ScanMevAction {
    config: JitoMevConfig,
}

impl ScanMevAction {
    pub fn new(config: JitoMevConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Action for ScanMevAction {
    fn name(&self) -> &str {
        "scan_mev_opportunities"
    }

    fn description(&self) -> &str {
        "Scan mempool for MEV opportunities"
    }

    fn config(&self) -> ActionConfig {
        ActionConfig {
            name: self.name().to_string(),
            description: self.description().to_string(),
            enabled: self.config.auto_scan,
            similes: vec!["mev_scan", "find_opportunities", "mempool_scan"],
            examples: vec![],
            validates: Some(vec![]),
        }
    }

    fn validate(&self, _params: &HashMap<String, Value>) -> Result<(), String> {
        Ok(())
    }

    fn is_available(&self, emotional_state: Option<&EmotionalState>) -> bool {
        if let Some(state) = emotional_state {
            // Scan more aggressively when excited, less when fearful
            let excitement = state.get_emotion_value(&Emotion::Excitement);
            let fear = state.get_emotion_value(&Emotion::Fear);

            excitement > fear
        } else {
            true
        }
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Scan for arbitrage opportunities".to_string(),
            request: ActionRequest {
                action: self.name().to_string(),
                params: json!({
                    "mev_types": ["arbitrage", "liquidation"],
                    "min_profit_sol": 0.1
                }),
            },
            expected_emotional_impact: EmotionalImpact {
                primary_emotion: Emotion::Curiosity,
                intensity: 0.6,
                valence: 0.7,
            },
        }]
    }

    async fn execute(
        &self,
        request: ActionRequest,
    ) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mev_types = request
            .params
            .get("mev_types")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["arbitrage", "liquidation", "backrun"]);

        // Mock MEV opportunities
        let opportunities = vec![
            json!({
                "id": uuid::Uuid::new_v4().to_string(),
                "type": "arbitrage",
                "profit_sol": 1.2,
                "confidence": 0.85,
                "expiry_slot": 123456789,
            }),
            json!({
                "id": uuid::Uuid::new_v4().to_string(),
                "type": "liquidation",
                "profit_sol": 3.5,
                "confidence": 0.92,
                "expiry_slot": 123456790,
            }),
        ];

        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), json!("scan_mev_opportunities"));
        metadata.insert("mev_types".to_string(), json!(mev_types));
        metadata.insert(
            "opportunities_found".to_string(),
            json!(opportunities.len()),
        );
        metadata.insert("opportunities".to_string(), json!(opportunities));

        Ok(ActionResult {
            success: true,
            message: format!("Found {} MEV opportunities", opportunities.len()),
            metadata,
            side_effects: vec![],
        })
    }
}

/// Action to optimize staking for MEV rewards
pub struct OptimizeStakingAction {
    config: JitoMevConfig,
}

impl OptimizeStakingAction {
    pub fn new(config: JitoMevConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Action for OptimizeStakingAction {
    fn name(&self) -> &str {
        "optimize_staking_mev"
    }

    fn description(&self) -> &str {
        "Optimize staking allocation for maximum MEV rewards"
    }

    fn config(&self) -> ActionConfig {
        ActionConfig {
            name: self.name().to_string(),
            description: self.description().to_string(),
            enabled: true,
            similes: vec!["stake_optimize", "mev_staking", "maximize_rewards"],
            examples: vec![],
            validates: Some(vec!["params.stake_amount_sol"]),
        }
    }

    fn validate(&self, params: &HashMap<String, Value>) -> Result<(), String> {
        let stake_amount = params
            .get("stake_amount_sol")
            .and_then(|v| v.as_f64())
            .ok_or("Missing stake_amount_sol parameter")?;

        if stake_amount <= 0.0 {
            return Err("Stake amount must be positive".to_string());
        }

        Ok(())
    }

    fn is_available(&self, _emotional_state: Option<&EmotionalState>) -> bool {
        true
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Optimize 1000 SOL staking".to_string(),
            request: ActionRequest {
                action: self.name().to_string(),
                params: json!({
                    "stake_amount_sol": 1000.0,
                    "rebalance": true
                }),
            },
            expected_emotional_impact: EmotionalImpact {
                primary_emotion: Emotion::Confidence,
                intensity: 0.6,
                valence: 0.8,
            },
        }]
    }

    async fn execute(
        &self,
        request: ActionRequest,
    ) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let stake_amount = Decimal::from_f64(
            request
                .params
                .get("stake_amount_sol")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        )
        .unwrap_or_default();

        // Calculate optimal distribution
        let base_apy = 5.5; // Mock base APY
        let mev_boost_apy = 7.0; // Typical MEV boost
        let total_apy = base_apy + mev_boost_apy;

        let estimated_annual_rewards =
            stake_amount * Decimal::from_f64(total_apy / 100.0).unwrap_or_default();

        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), json!("optimize_staking_mev"));
        metadata.insert(
            "stake_amount_sol".to_string(),
            json!(stake_amount.to_string()),
        );
        metadata.insert("base_apy".to_string(), json!(base_apy));
        metadata.insert("mev_boost_apy".to_string(), json!(mev_boost_apy));
        metadata.insert("total_apy".to_string(), json!(total_apy));
        metadata.insert(
            "estimated_annual_rewards_sol".to_string(),
            json!(estimated_annual_rewards.to_string()),
        );
        metadata.insert(
            "recommended_validators".to_string(),
            json!(["Jito1", "Jito2", "Jito3"]),
        );

        Ok(ActionResult {
            success: true,
            message: format!(
                "Optimized staking for {}% APY ({} SOL annual rewards)",
                total_apy, estimated_annual_rewards
            ),
            metadata,
            side_effects: vec![],
        })
    }
}
