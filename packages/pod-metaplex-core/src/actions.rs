//! Actions for Metaplex Core pod

use async_trait::async_trait;
use banshee_core::{
    action::{Action, ActionConfig, ActionExample, ActionRequest, ActionResult, EmotionalImpact},
    emotion::{Emotion, EmotionalState},
};
use serde_json::{json, Value};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

use crate::{
    compression::CompressionManager, config::MetaplexCoreConfig, core_asset::CoreAssetManager,
    mpl404::Mpl404Manager, types::*,
};

/// Action to create Core asset
pub struct CreateCoreAssetAction {
    config: MetaplexCoreConfig,
}

impl CreateCoreAssetAction {
    pub fn new(config: MetaplexCoreConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Action for CreateCoreAssetAction {
    fn name(&self) -> &str {
        "create_core_asset"
    }

    fn description(&self) -> &str {
        "Create a Metaplex Core NFT with single-account design"
    }

    fn config(&self) -> ActionConfig {
        ActionConfig {
            name: self.name().to_string(),
            description: self.description().to_string(),
            enabled: true,
            similes: vec!["mint_nft", "create_asset", "deploy_nft"],
            examples: vec![],
            validates: Some(vec!["params.name", "params.uri"]),
        }
    }

    fn validate(&self, params: &HashMap<String, Value>) -> Result<(), String> {
        params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing name parameter")?;

        params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or("Missing uri parameter")?;

        Ok(())
    }

    fn is_available(&self, emotional_state: Option<&EmotionalState>) -> bool {
        if let Some(state) = emotional_state {
            // Create assets when feeling creative or excited
            let excitement = state.get_emotion_value(&Emotion::Excitement);
            let creativity = state.get_emotion_value(&Emotion::Joy); // Using Joy as proxy for creativity

            excitement > 0.5 || creativity > 0.6
        } else {
            true
        }
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Create emotional NFT".to_string(),
            request: ActionRequest {
                action: self.name().to_string(),
                params: json!({
                    "name": "Banshee Emotion #1",
                    "uri": "https://api.banshee.ai/metadata/1.json",
                    "royalty_percentage": 5.0,
                    "compress": false,
                    "emotional_theme": "Joy"
                }),
            },
            expected_emotional_impact: EmotionalImpact {
                primary_emotion: Emotion::Pride,
                intensity: 0.7,
                valence: 0.8,
            },
        }]
    }

    async fn execute(
        &self,
        request: ActionRequest,
    ) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let params: CreateAssetParams = serde_json::from_value(request.params)?;

        // Calculate costs
        let cost = if params.compress {
            CompressionManager::calculate_cost_per_nft(rust_decimal::Decimal::from(1), 16384)
        } else {
            CoreAssetManager::calculate_storage_cost()
        };

        let asset_address = Pubkey::new_unique();

        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), json!("create_core_asset"));
        metadata.insert(
            "asset_address".to_string(),
            json!(asset_address.to_string()),
        );
        metadata.insert("name".to_string(), json!(params.name));
        metadata.insert("uri".to_string(), json!(params.uri));
        metadata.insert("compressed".to_string(), json!(params.compress));
        metadata.insert("cost_sol".to_string(), json!(cost.to_string()));
        metadata.insert(
            "savings_percentage".to_string(),
            json!(CoreAssetManager::calculate_savings_percentage()),
        );

        if let Some(theme) = params.emotional_theme {
            metadata.insert("emotional_theme".to_string(), json!(theme));
        }

        Ok(ActionResult {
            success: true,
            message: format!(
                "Created {} asset '{}' for {} SOL ({}% savings)",
                if params.compress {
                    "compressed"
                } else {
                    "core"
                },
                params.name,
                cost,
                CoreAssetManager::calculate_savings_percentage()
            ),
            metadata,
            side_effects: vec![],
        })
    }
}

/// Action to create MPL-404 hybrid asset
pub struct CreateMpl404Action {
    config: MetaplexCoreConfig,
}

impl CreateMpl404Action {
    pub fn new(config: MetaplexCoreConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Action for CreateMpl404Action {
    fn name(&self) -> &str {
        "create_mpl404"
    }

    fn description(&self) -> &str {
        "Create an MPL-404 hybrid fungible/NFT asset"
    }

    fn config(&self) -> ActionConfig {
        ActionConfig {
            name: self.name().to_string(),
            description: self.description().to_string(),
            enabled: true,
            similes: vec!["deploy_404", "create_hybrid", "mint_404"],
            examples: vec![],
            validates: Some(vec!["params.name", "params.symbol", "params.total_supply"]),
        }
    }

    fn validate(&self, params: &HashMap<String, Value>) -> Result<(), String> {
        params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("Missing name parameter")?;

        params
            .get("symbol")
            .and_then(|v| v.as_str())
            .ok_or("Missing symbol parameter")?;

        let total_supply = params
            .get("total_supply")
            .and_then(|v| v.as_u64())
            .ok_or("Missing or invalid total_supply")?;

        if total_supply == 0 || total_supply > self.config.mpl404.max_supply {
            return Err(format!(
                "Total supply must be between 1 and {}",
                self.config.mpl404.max_supply
            ));
        }

        Ok(())
    }

    fn is_available(&self, _emotional_state: Option<&EmotionalState>) -> bool {
        true
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Create MPL-404 token".to_string(),
            request: ActionRequest {
                action: self.name().to_string(),
                params: json!({
                    "name": "Banshee 404",
                    "symbol": "B404",
                    "total_supply": 1000000000,
                    "decimals": 6,
                    "nft_threshold": 1000000,
                    "base_uri": "https://api.banshee.ai/404/",
                    "royalty_percentage": 5.0
                }),
            },
            expected_emotional_impact: EmotionalImpact {
                primary_emotion: Emotion::Excitement,
                intensity: 0.8,
                valence: 0.9,
            },
        }]
    }

    async fn execute(
        &self,
        request: ActionRequest,
    ) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let params: CreateMpl404Params = serde_json::from_value(request.params)?;

        let mint_address = Pubkey::new_unique();

        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), json!("create_mpl404"));
        metadata.insert("mint_address".to_string(), json!(mint_address.to_string()));
        metadata.insert("name".to_string(), json!(params.name));
        metadata.insert("symbol".to_string(), json!(params.symbol));
        metadata.insert("total_supply".to_string(), json!(params.total_supply));
        metadata.insert("nft_threshold".to_string(), json!(params.nft_threshold));
        metadata.insert("auto_swap".to_string(), json!(self.config.mpl404.auto_swap));

        Ok(ActionResult {
            success: true,
            message: format!(
                "Created MPL-404 token '{}' ({}) with {} supply",
                params.name, params.symbol, params.total_supply
            ),
            metadata,
            side_effects: vec![],
        })
    }
}

/// Action to swap MPL-404 assets
pub struct SwapMpl404Action {
    config: MetaplexCoreConfig,
}

impl SwapMpl404Action {
    pub fn new(config: MetaplexCoreConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Action for SwapMpl404Action {
    fn name(&self) -> &str {
        "swap_mpl404"
    }

    fn description(&self) -> &str {
        "Swap between fungible and NFT states of MPL-404 asset"
    }

    fn config(&self) -> ActionConfig {
        ActionConfig {
            name: self.name().to_string(),
            description: self.description().to_string(),
            enabled: true,
            similes: vec!["404_swap", "convert_404", "transform_asset"],
            examples: vec![],
            validates: Some(vec!["params.mint", "params.direction"]),
        }
    }

    fn validate(&self, params: &HashMap<String, Value>) -> Result<(), String> {
        params
            .get("mint")
            .and_then(|v| v.as_str())
            .ok_or("Missing mint parameter")?;

        let direction = params
            .get("direction")
            .and_then(|v| v.as_str())
            .ok_or("Missing direction parameter")?;

        if !["fungible_to_nft", "nft_to_fungible"].contains(&direction) {
            return Err("Direction must be 'fungible_to_nft' or 'nft_to_fungible'".to_string());
        }

        Ok(())
    }

    fn is_available(&self, emotional_state: Option<&EmotionalState>) -> bool {
        if let Some(state) = emotional_state {
            // Swap to NFT when feeling unique/special
            // Swap to fungible when feeling practical
            let uniqueness = state.get_emotion_value(&Emotion::Pride);
            uniqueness > 0.3
        } else {
            true
        }
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Convert tokens to NFT".to_string(),
            request: ActionRequest {
                action: self.name().to_string(),
                params: json!({
                    "mint": "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
                    "direction": "fungible_to_nft",
                    "amount": 1000000
                }),
            },
            expected_emotional_impact: EmotionalImpact {
                primary_emotion: Emotion::Pride,
                intensity: 0.6,
                valence: 0.7,
            },
        }]
    }

    async fn execute(
        &self,
        request: ActionRequest,
    ) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let mint = request
            .params
            .get("mint")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let direction = request
            .params
            .get("direction")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let swap_fee =
            Mpl404Manager::calculate_swap_fee(1_000_000, self.config.mpl404.swap_fee_percentage, 6);

        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), json!("swap_mpl404"));
        metadata.insert("mint".to_string(), json!(mint));
        metadata.insert("direction".to_string(), json!(direction));
        metadata.insert("swap_fee".to_string(), json!(swap_fee));
        metadata.insert("nft_id".to_string(), json!(42)); // Mock NFT ID

        Ok(ActionResult {
            success: true,
            message: format!("Swapped MPL-404 asset ({})", direction),
            metadata,
            side_effects: vec![],
        })
    }
}

/// Action to create compressed NFT collection
pub struct CreateCompressedCollectionAction {
    config: MetaplexCoreConfig,
}

impl CreateCompressedCollectionAction {
    pub fn new(config: MetaplexCoreConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Action for CreateCompressedCollectionAction {
    fn name(&self) -> &str {
        "create_compressed_collection"
    }

    fn description(&self) -> &str {
        "Create a compressed NFT collection using Merkle trees"
    }

    fn config(&self) -> ActionConfig {
        ActionConfig {
            name: self.name().to_string(),
            description: self.description().to_string(),
            enabled: true,
            similes: vec!["compressed_mint", "merkle_collection", "batch_nft"],
            examples: vec![],
            validates: Some(vec!["params.collection_size"]),
        }
    }

    fn validate(&self, params: &HashMap<String, Value>) -> Result<(), String> {
        let size = params
            .get("collection_size")
            .and_then(|v| v.as_u64())
            .ok_or("Missing collection_size parameter")?;

        if size == 0 || size > 1_000_000 {
            return Err("Collection size must be between 1 and 1,000,000".to_string());
        }

        Ok(())
    }

    fn is_available(&self, _emotional_state: Option<&EmotionalState>) -> bool {
        true
    }

    fn examples(&self) -> Vec<ActionExample> {
        vec![ActionExample {
            description: "Create 10k compressed collection".to_string(),
            request: ActionRequest {
                action: self.name().to_string(),
                params: json!({
                    "collection_size": 10000,
                    "name": "Banshee Emotions",
                    "symbol": "BEMO",
                    "base_uri": "https://api.banshee.ai/compressed/"
                }),
            },
            expected_emotional_impact: EmotionalImpact {
                primary_emotion: Emotion::Satisfaction,
                intensity: 0.8,
                valence: 0.9,
            },
        }]
    }

    async fn execute(
        &self,
        request: ActionRequest,
    ) -> Result<ActionResult, Box<dyn std::error::Error + Send + Sync>> {
        let collection_size = request
            .params
            .get("collection_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(10_000) as u32;

        // Calculate optimal tree depth
        let max_depth = (collection_size as f64).log2().ceil() as u32;
        let tree_cost = CompressionManager::calculate_tree_cost(
            max_depth,
            self.config.compression.canopy_depth,
        );
        let per_nft_cost = CompressionManager::calculate_cost_per_nft(
            tree_cost,
            CompressionManager::get_max_capacity(max_depth),
        );

        let mut metadata = HashMap::new();
        metadata.insert("action".to_string(), json!("create_compressed_collection"));
        metadata.insert("collection_size".to_string(), json!(collection_size));
        metadata.insert("tree_depth".to_string(), json!(max_depth));
        metadata.insert("tree_cost_sol".to_string(), json!(tree_cost.to_string()));
        metadata.insert(
            "per_nft_cost_sol".to_string(),
            json!(per_nft_cost.to_string()),
        );
        metadata.insert(
            "total_capacity".to_string(),
            json!(CompressionManager::get_max_capacity(max_depth)),
        );

        Ok(ActionResult {
            success: true,
            message: format!(
                "Created compressed collection for {} NFTs ({}x cost reduction)",
                collection_size,
                (rust_decimal::Decimal::new(12, 3) / per_nft_cost).round() // ~0.012 SOL for traditional
            ),
            metadata,
            side_effects: vec![],
        })
    }
}
