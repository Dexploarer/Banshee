//! Providers for Jito MEV pod

use async_trait::async_trait;
use banshee_core::{
    provider::{Provider, ProviderConfig, ProviderResult},
    Context, Result,
};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde_json::json;
use std::collections::HashMap;

use crate::{config::JitoMevConfig, tip_router::TipRouter, types::*};

/// Provider for MEV analytics
pub struct MevAnalyticsProvider {
    config: JitoMevConfig,
    provider_config: ProviderConfig,
}

impl MevAnalyticsProvider {
    pub fn new(config: JitoMevConfig) -> Self {
        let provider_config = ProviderConfig {
            name: "jito_mev_analytics".to_string(),
            description: "Get MEV extraction analytics and performance metrics".to_string(),
            priority: 50,
            enabled: true,
            settings: HashMap::new(),
        };

        Self {
            config,
            provider_config,
        }
    }
}

#[async_trait]
impl Provider for MevAnalyticsProvider {
    fn name(&self) -> &str {
        &self.provider_config.name
    }

    fn description(&self) -> &str {
        &self.provider_config.description
    }

    fn config(&self) -> &ProviderConfig {
        &self.provider_config
    }

    async fn provide(&self, _context: &Context) -> Result<Vec<ProviderResult>> {
        // Get 24h analytics
        let analytics = MevAnalytics {
            total_opportunities_24h: 342,
            successful_bundles_24h: 287,
            total_profit_24h_sol: Decimal::from(125),
            total_tips_paid_24h_sol: Decimal::from(25),
            average_profit_per_bundle_sol: Decimal::new(435, 3), // 0.435 SOL
            success_rate: 83.9,
            top_mev_types: vec![
                (MevType::Arbitrage, 156),
                (MevType::Liquidation, 89),
                (MevType::Backrun, 42),
            ],
        };

        Ok(vec![ProviderResult {
            provider: self.name().to_string(),
            data: serde_json::to_value(analytics)?,
            relevance: 0.8,
            confidence: 0.9,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }])
    }

    async fn is_relevant(&self, context: &Context) -> Result<bool> {
        // MEV analytics are relevant when the agent is considering trading actions
        if let Some(msg) = context.latest_message() {
            let content = msg.text_content().to_lowercase();
            Ok(content.contains("mev")
                || content.contains("profit")
                || content.contains("analytics"))
        } else {
            Ok(true)
        }
    }

    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Provider for validator performance metrics
pub struct ValidatorMetricsProvider {
    config: JitoMevConfig,
    provider_config: ProviderConfig,
}

impl ValidatorMetricsProvider {
    pub fn new(config: JitoMevConfig) -> Self {
        let provider_config = ProviderConfig {
            name: "jito_validator_metrics".to_string(),
            description: "Get validator performance metrics for MEV optimization".to_string(),
            priority: 45,
            enabled: true,
            settings: HashMap::new(),
        };

        Self {
            config,
            provider_config,
        }
    }
}

#[async_trait]
impl Provider for ValidatorMetricsProvider {
    fn name(&self) -> &str {
        &self.provider_config.name
    }

    fn description(&self) -> &str {
        &self.provider_config.description
    }

    fn config(&self) -> &ProviderConfig {
        &self.provider_config
    }

    async fn provide(&self, _context: &Context) -> Result<Vec<ProviderResult>> {
        // Mock top validators
        let mut validators = Vec::new();
        for i in 0..5 {
            let metrics = ValidatorMetrics {
                validator: format!("JitoValidator{}...", i + 1),
                slots_processed_24h: 8640 - (i as u32 * 100),
                bundles_landed_24h: 342 - (i as u32 * 20),
                mev_tips_earned_24h_sol: Decimal::from(12 - i),
                average_slot_time_ms: 405.2 + (i as f64 * 2.5),
                reliability_score: 0.94 - (i as f64 * 0.01),
            };
            validators.push(metrics);
        }

        Ok(vec![ProviderResult {
            provider: self.name().to_string(),
            data: json!(validators),
            relevance: 0.7,
            confidence: 0.85,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }])
    }

    async fn is_relevant(&self, context: &Context) -> Result<bool> {
        if let Some(msg) = context.latest_message() {
            let content = msg.text_content().to_lowercase();
            Ok(content.contains("validator")
                || content.contains("stake")
                || content.contains("performance"))
        } else {
            Ok(false)
        }
    }

    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Provider for staking rewards with MEV boost
pub struct StakingRewardsProvider {
    config: JitoMevConfig,
    provider_config: ProviderConfig,
}

impl StakingRewardsProvider {
    pub fn new(config: JitoMevConfig) -> Self {
        let provider_config = ProviderConfig {
            name: "jito_staking_rewards".to_string(),
            description: "Calculate staking rewards with MEV boost".to_string(),
            priority: 40,
            enabled: true,
            settings: HashMap::new(),
        };

        Self {
            config,
            provider_config,
        }
    }
}

#[async_trait]
impl Provider for StakingRewardsProvider {
    fn name(&self) -> &str {
        &self.provider_config.name
    }

    fn description(&self) -> &str {
        &self.provider_config.description
    }

    fn config(&self) -> &ProviderConfig {
        &self.provider_config
    }

    async fn provide(&self, _context: &Context) -> Result<Vec<ProviderResult>> {
        // Default stake amount
        let stake_amount = Decimal::from(1000);

        // Calculate rewards with MEV boost
        let base_apy = 5.5;
        let daily_tips_estimate = stake_amount * Decimal::new(2, 4); // 0.02% daily
        let mev_boost_apy = TipRouter::estimate_apy_boost(
            stake_amount,
            daily_tips_estimate,
            0.0, // Just calculate boost
        );

        let rewards = StakingRewards {
            validator: "JitoValidator1...".to_string(),
            base_apy,
            mev_boost_apy,
            total_apy: base_apy + mev_boost_apy,
            stake_amount_sol: stake_amount,
            estimated_yearly_rewards_sol: stake_amount
                * Decimal::from_f64((base_apy + mev_boost_apy) / 100.0).unwrap_or_default(),
            mev_tips_received_24h: daily_tips_estimate,
        };

        Ok(vec![ProviderResult {
            provider: self.name().to_string(),
            data: serde_json::to_value(rewards)?,
            relevance: 0.75,
            confidence: 0.9,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }])
    }

    async fn is_relevant(&self, context: &Context) -> Result<bool> {
        if let Some(msg) = context.latest_message() {
            let content = msg.text_content().to_lowercase();
            Ok(content.contains("stake") || content.contains("reward") || content.contains("apy"))
        } else {
            Ok(false)
        }
    }

    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Provider for current MEV opportunities
pub struct MevOpportunityProvider {
    config: JitoMevConfig,
    provider_config: ProviderConfig,
}

impl MevOpportunityProvider {
    pub fn new(config: JitoMevConfig) -> Self {
        let provider_config = ProviderConfig {
            name: "jito_mev_opportunities".to_string(),
            description: "Get current MEV opportunities from mempool".to_string(),
            priority: 60,
            enabled: config.auto_scan,
            settings: HashMap::new(),
        };

        Self {
            config,
            provider_config,
        }
    }
}

#[async_trait]
impl Provider for MevOpportunityProvider {
    fn name(&self) -> &str {
        &self.provider_config.name
    }

    fn description(&self) -> &str {
        &self.provider_config.description
    }

    fn config(&self) -> &ProviderConfig {
        &self.provider_config
    }

    async fn provide(&self, context: &Context) -> Result<Vec<ProviderResult>> {
        // Check emotional state for trading decisions
        let emotional_signal = {
            let joy = context
                .emotional_state
                .emotions
                .get(&banshee_core::Emotion::Joy)
                .unwrap_or(&0.0);
            let fear = context
                .emotional_state
                .emotions
                .get(&banshee_core::Emotion::Fear)
                .unwrap_or(&0.0);
            let hope = context
                .emotional_state
                .emotions
                .get(&banshee_core::Emotion::Hope)
                .unwrap_or(&0.0);

            EmotionalSignal {
                greed: (*joy * 0.6 + *hope * 0.4).min(1.0) as f64,
                fear: *fear as f64,
                confidence: (1.0 - *fear) as f64,
            }
        };

        // Mock current opportunities
        let opportunities = vec![
            MevOpportunity {
                id: uuid::Uuid::new_v4().to_string(),
                mev_type: MevType::Arbitrage,
                target_transaction: Some("mock_tx_1".to_string()),
                estimated_profit_sol: Decimal::new(12, 1), // 1.2 SOL
                required_capital_sol: Decimal::from(10),
                confidence_score: 0.85,
                expiry_slot: 123456789,
                risk_level: RiskLevel::Medium,
                emotional_signal,
            },
            MevOpportunity {
                id: uuid::Uuid::new_v4().to_string(),
                mev_type: MevType::Liquidation,
                target_transaction: None,
                estimated_profit_sol: Decimal::new(35, 1), // 3.5 SOL
                required_capital_sol: Decimal::from(50),
                confidence_score: 0.92,
                expiry_slot: 123456790,
                risk_level: RiskLevel::High,
                emotional_signal,
            },
        ];

        // Filter based on config
        let filtered: Vec<_> = opportunities
            .into_iter()
            .filter(|opp| {
                opp.estimated_profit_sol >= self.config.min_profit_sol
                    && opp.confidence_score >= self.config.risk_management.min_confidence_score
                    && self
                        .config
                        .risk_management
                        .accepted_risk_levels
                        .contains(&opp.risk_level)
            })
            .collect();

        Ok(vec![ProviderResult {
            provider: self.name().to_string(),
            data: json!(filtered),
            relevance: 0.95,
            confidence: 0.8,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }])
    }

    async fn is_relevant(&self, context: &Context) -> Result<bool> {
        // Always relevant if auto-scan is enabled
        if self.config.auto_scan {
            return Ok(true);
        }

        if let Some(msg) = context.latest_message() {
            let content = msg.text_content().to_lowercase();
            Ok(content.contains("opportunity")
                || content.contains("mev")
                || content.contains("scan"))
        } else {
            Ok(false)
        }
    }

    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
