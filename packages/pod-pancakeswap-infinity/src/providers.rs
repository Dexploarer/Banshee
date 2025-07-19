//! Providers for PancakeSwap Infinity data and services

use crate::config::{ChainConfig, PancakeSwapConfig};
use crate::types::{
    AmmType, ArbitrageOpportunity, CrossChainPool, PoolId, PoolMetrics, PoolState, Position,
    PriceData, Token, TokenPair, Volume24h,
};
use crate::Result;
use alloy::primitives::{Address, U256};
use async_trait::async_trait;
use banshee_core::{Provider, ProviderResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main provider for PancakeSwap Infinity integration
pub struct PancakeSwapProvider {
    config: PancakeSwapConfig,
    http_client: Client,
    pool_provider: PoolProvider,
    price_provider: PriceProvider,
    liquidity_provider: LiquidityProvider,
    analytics_provider: AnalyticsProvider,
}

impl PancakeSwapProvider {
    pub fn new(config: PancakeSwapConfig) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.api.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        let pool_provider = PoolProvider::new(config.clone(), http_client.clone());
        let price_provider = PriceProvider::new(config.clone(), http_client.clone());
        let liquidity_provider = LiquidityProvider::new(config.clone(), http_client.clone());
        let analytics_provider = AnalyticsProvider::new(config.clone(), http_client.clone());

        Self {
            config,
            http_client,
            pool_provider,
            price_provider,
            liquidity_provider,
            analytics_provider,
        }
    }

    /// Get all available pools across supported chains
    pub async fn get_all_pools(&self) -> Result<Vec<PoolState>> {
        self.pool_provider.get_all_pools().await
    }

    /// Find the best pool for a token pair
    pub async fn find_best_pool(
        &self,
        token_pair: &TokenPair,
        amm_types: Option<Vec<AmmType>>,
    ) -> Result<Option<PoolState>> {
        self.pool_provider
            .find_best_pool(token_pair, amm_types)
            .await
    }

    /// Get real-time price data for a token pair
    pub async fn get_price_data(&self, token_pair: &TokenPair) -> Result<PriceData> {
        self.price_provider.get_price_data(token_pair).await
    }

    /// Find arbitrage opportunities across pools
    pub async fn find_arbitrage_opportunities(
        &self,
        token_pair: &TokenPair,
    ) -> Result<Vec<ArbitrageOpportunity>> {
        self.analytics_provider
            .find_arbitrage_opportunities(token_pair)
            .await
    }
}

#[async_trait]
impl Provider for PancakeSwapProvider {
    fn name(&self) -> &str {
        "pancakeswap_infinity"
    }

    fn description(&self) -> &str {
        "PancakeSwap Infinity provider for multi-AMM liquidity and trading"
    }

    async fn provide(&self, request: serde_json::Value) -> ProviderResult {
        let request_type = request.get("type").and_then(|v| v.as_str()).unwrap_or("");

        match request_type {
            "get_pools" => {
                let pools = self.get_all_pools().await?;
                Ok(serde_json::to_value(pools)?)
            }
            "get_price" => {
                let token_pair: TokenPair = serde_json::from_value(
                    request
                        .get("token_pair")
                        .unwrap_or(&serde_json::Value::Null)
                        .clone(),
                )?;
                let price_data = self.get_price_data(&token_pair).await?;
                Ok(serde_json::to_value(price_data)?)
            }
            "find_arbitrage" => {
                let token_pair: TokenPair = serde_json::from_value(
                    request
                        .get("token_pair")
                        .unwrap_or(&serde_json::Value::Null)
                        .clone(),
                )?;
                let opportunities = self.find_arbitrage_opportunities(&token_pair).await?;
                Ok(serde_json::to_value(opportunities)?)
            }
            _ => Err(format!("Unknown request type: {}", request_type).into()),
        }
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "get_pools".to_string(),
            "get_price".to_string(),
            "find_arbitrage".to_string(),
            "liquidity_analysis".to_string(),
            "pool_metrics".to_string(),
            "cross_chain_pools".to_string(),
        ]
    }
}

/// Provider for pool-related operations
#[derive(Clone)]
pub struct PoolProvider {
    config: PancakeSwapConfig,
    http_client: Client,
}

impl PoolProvider {
    pub fn new(config: PancakeSwapConfig, http_client: Client) -> Self {
        Self {
            config,
            http_client,
        }
    }

    /// Get all pools across supported chains
    pub async fn get_all_pools(&self) -> Result<Vec<PoolState>> {
        let mut all_pools = Vec::new();

        for (chain_id, chain_config) in &self.config.chains {
            let pools = self.get_pools_for_chain(*chain_id).await?;
            all_pools.extend(pools);
        }

        Ok(all_pools)
    }

    /// Get pools for a specific chain
    pub async fn get_pools_for_chain(&self, chain_id: u64) -> Result<Vec<PoolState>> {
        let chain_config = self
            .config
            .chains
            .get(&chain_id)
            .ok_or_else(|| format!("Chain {} not configured", chain_id))?;

        let url = format!("{}/pools?chain_id={}", self.config.api.base_url, chain_id);

        let response = self
            .http_client
            .get(&url)
            .header("User-Agent", "Banshee-PancakeSwap-Pod/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("API request failed: {}", response.status()).into());
        }

        let pools_response: PoolsApiResponse = response.json().await?;

        // Convert API response to internal pool states
        let pools = pools_response
            .pools
            .into_iter()
            .map(|api_pool| self.convert_api_pool_to_state(api_pool, chain_config))
            .collect::<Result<Vec<_>>>()?;

        Ok(pools)
    }

    /// Find the best pool for a token pair
    pub async fn find_best_pool(
        &self,
        token_pair: &TokenPair,
        amm_types: Option<Vec<AmmType>>,
    ) -> Result<Option<PoolState>> {
        let all_pools = self.get_all_pools().await?;

        let matching_pools: Vec<_> = all_pools
            .into_iter()
            .filter(|pool| {
                // Check if pool contains the token pair
                let tokens_match = (pool.token0.address == token_pair.token0.address
                    && pool.token1.address == token_pair.token1.address)
                    || (pool.token0.address == token_pair.token1.address
                        && pool.token1.address == token_pair.token0.address);

                // Check AMM type filter if provided
                let amm_type_match = if let Some(ref types) = amm_types {
                    if let crate::types::PoolType::Standard { amm_type, .. } = &pool.pool_type {
                        types.contains(amm_type)
                    } else {
                        true // Include custom pools
                    }
                } else {
                    true
                };

                tokens_match && amm_type_match
            })
            .collect();

        if matching_pools.is_empty() {
            return Ok(None);
        }

        // Score pools based on configuration weights
        let scoring = &self.config.trading.pool_selection.scoring;
        let best_pool = matching_pools.into_iter().max_by(|a, b| {
            let score_a = self.calculate_pool_score(a, scoring);
            let score_b = self.calculate_pool_score(b, scoring);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(best_pool)
    }

    /// Calculate pool score based on configured weights
    fn calculate_pool_score(
        &self,
        pool: &PoolState,
        scoring: &crate::config::PoolScoringWeights,
    ) -> f64 {
        let liquidity_score = pool.tvl_usd * scoring.liquidity;
        let volume_score = pool.total_volume_24h.to::<f64>() * scoring.volume;
        let fees_score = pool.total_fees_24h.to::<f64>() * scoring.fees;
        let stability_score = (1.0 / (1.0 + pool.apr.abs())) * scoring.price_stability;

        liquidity_score + volume_score + fees_score + stability_score
    }

    /// Convert API pool response to internal PoolState
    fn convert_api_pool_to_state(
        &self,
        api_pool: ApiPoolData,
        chain_config: &ChainConfig,
    ) -> Result<PoolState> {
        // TODO: Implement proper conversion from API response to PoolState
        // This is a placeholder implementation

        let pool_id = PoolId(api_pool.id);

        Ok(PoolState {
            pool_id,
            pool_type: crate::types::PoolType::Standard {
                amm_type: AmmType::CLAMM, // Default, should be determined from API
                fee_tier: api_pool.fee_tier,
            },
            token0: Token {
                address: api_pool.token0_address.parse()?,
                symbol: api_pool.token0_symbol,
                name: api_pool.token0_name,
                decimals: api_pool.token0_decimals,
                chain_id: chain_config.chain_id,
            },
            token1: Token {
                address: api_pool.token1_address.parse()?,
                symbol: api_pool.token1_symbol,
                name: api_pool.token1_name,
                decimals: api_pool.token1_decimals,
                chain_id: chain_config.chain_id,
            },
            reserve0: U256::from_str_radix(&api_pool.reserve0, 10)?,
            reserve1: U256::from_str_radix(&api_pool.reserve1, 10)?,
            sqrt_price: U256::from_str_radix(&api_pool.sqrt_price, 10)?,
            tick: api_pool.tick,
            liquidity: U256::from_str_radix(&api_pool.liquidity, 10)?,
            fee_tier: api_pool.fee_tier,
            total_volume_24h: U256::from_str_radix(&api_pool.volume_24h, 10)?,
            total_fees_24h: U256::from_str_radix(&api_pool.fees_24h, 10)?,
            tvl_usd: api_pool.tvl_usd,
            apr: api_pool.apr,
            last_updated: chrono::Utc::now(),
        })
    }
}

/// Provider for price-related operations
#[derive(Clone)]
pub struct PriceProvider {
    config: PancakeSwapConfig,
    http_client: Client,
}

impl PriceProvider {
    pub fn new(config: PancakeSwapConfig, http_client: Client) -> Self {
        Self {
            config,
            http_client,
        }
    }

    /// Get real-time price data for a token pair
    pub async fn get_price_data(&self, token_pair: &TokenPair) -> Result<PriceData> {
        let url = format!(
            "{}/price?token0={}&token1={}&chain_id={}",
            self.config.api.base_url,
            token_pair.token0.address,
            token_pair.token1.address,
            self.config.default_chain
        );

        let response = self.http_client.get(&url).send().await?;

        let price_response: PriceApiResponse = response.json().await?;

        Ok(PriceData {
            pool_id: PoolId(price_response.pool_id),
            token_pair: token_pair.clone(),
            price: price_response.price,
            price_24h_ago: price_response.price_24h_ago,
            price_change_24h: price_response.price_change_24h,
            price_change_24h_percent: price_response.price_change_24h_percent,
            volume_24h: Volume24h {
                volume_usd: price_response.volume_24h_usd,
                volume_token0: U256::from_str_radix(&price_response.volume_24h_token0, 10)?,
                volume_token1: U256::from_str_radix(&price_response.volume_24h_token1, 10)?,
                txn_count: price_response.txn_count_24h,
            },
            timestamp: chrono::Utc::now(),
        })
    }
}

/// Provider for liquidity-related operations
#[derive(Clone)]
pub struct LiquidityProvider {
    config: PancakeSwapConfig,
    http_client: Client,
}

impl LiquidityProvider {
    pub fn new(config: PancakeSwapConfig, http_client: Client) -> Self {
        Self {
            config,
            http_client,
        }
    }

    /// Get user's liquidity positions
    pub async fn get_user_positions(
        &self,
        user: Address,
        chain_id: Option<u64>,
    ) -> Result<Vec<Position>> {
        let chain = chain_id.unwrap_or(self.config.default_chain);

        let url = format!(
            "{}/positions?user={}&chain_id={}",
            self.config.api.base_url, user, chain
        );

        let response = self.http_client.get(&url).send().await?;

        let positions_response: PositionsApiResponse = response.json().await?;

        // Convert API positions to internal format
        let positions = positions_response
            .positions
            .into_iter()
            .map(|api_pos| Position {
                id: api_pos.id,
                pool_id: PoolId(api_pos.pool_id),
                owner: user,
                tick_lower: api_pos.tick_lower,
                tick_upper: api_pos.tick_upper,
                liquidity: U256::from_str_radix(&api_pos.liquidity, 10).unwrap_or_default(),
                amount0: U256::from_str_radix(&api_pos.amount0, 10).unwrap_or_default(),
                amount1: U256::from_str_radix(&api_pos.amount1, 10).unwrap_or_default(),
                fees_earned0: U256::from_str_radix(&api_pos.fees_earned0, 10).unwrap_or_default(),
                fees_earned1: U256::from_str_radix(&api_pos.fees_earned1, 10).unwrap_or_default(),
                created_at: chrono::DateTime::parse_from_rfc3339(&api_pos.created_at)
                    .unwrap_or_else(|_| chrono::Utc::now().into())
                    .with_timezone(&chrono::Utc),
                last_updated: chrono::Utc::now(),
            })
            .collect();

        Ok(positions)
    }
}

/// Provider for analytics and advanced data
#[derive(Clone)]
pub struct AnalyticsProvider {
    config: PancakeSwapConfig,
    http_client: Client,
}

impl AnalyticsProvider {
    pub fn new(config: PancakeSwapConfig, http_client: Client) -> Self {
        Self {
            config,
            http_client,
        }
    }

    /// Find arbitrage opportunities across pools
    pub async fn find_arbitrage_opportunities(
        &self,
        token_pair: &TokenPair,
    ) -> Result<Vec<ArbitrageOpportunity>> {
        // TODO: Implement arbitrage opportunity detection
        // - Compare prices across different pools
        // - Calculate potential profits
        // - Account for gas costs
        // - Score opportunities by confidence

        Ok(vec![])
    }

    /// Get cross-chain pool information
    pub async fn get_cross_chain_pools(&self, token_pair: &TokenPair) -> Result<CrossChainPool> {
        let mut pools = HashMap::new();
        let mut total_tvl = 0.0;

        for (chain_id, _) in &self.config.chains {
            // Get pools for this chain
            // TODO: Implement actual cross-chain pool fetching
        }

        Ok(CrossChainPool {
            pools,
            total_tvl_usd: total_tvl,
            arbitrage_opportunities: Vec::new(),
            last_sync: chrono::Utc::now(),
        })
    }

    /// Get detailed pool metrics
    pub async fn get_pool_metrics(&self, pool_id: &PoolId) -> Result<PoolMetrics> {
        let url = format!("{}/metrics?pool_id={}", self.config.api.base_url, pool_id);

        let response = self.http_client.get(&url).send().await?;

        let metrics_response: MetricsApiResponse = response.json().await?;

        Ok(PoolMetrics {
            pool_id: pool_id.clone(),
            tvl_usd: metrics_response.tvl_usd,
            volume_24h_usd: metrics_response.volume_24h_usd,
            fees_24h_usd: metrics_response.fees_24h_usd,
            apr: metrics_response.apr,
            utilization: metrics_response.utilization,
            impermanent_loss: metrics_response.impermanent_loss,
            price_impact: metrics_response.price_impact,
            last_updated: chrono::Utc::now(),
        })
    }
}

// API Response Types

#[derive(Debug, Deserialize)]
struct PoolsApiResponse {
    pools: Vec<ApiPoolData>,
}

#[derive(Debug, Deserialize)]
struct ApiPoolData {
    id: String,
    token0_address: String,
    token0_symbol: String,
    token0_name: String,
    token0_decimals: u8,
    token1_address: String,
    token1_symbol: String,
    token1_name: String,
    token1_decimals: u8,
    fee_tier: u32,
    reserve0: String,
    reserve1: String,
    sqrt_price: String,
    tick: Option<i32>,
    liquidity: String,
    volume_24h: String,
    fees_24h: String,
    tvl_usd: f64,
    apr: f64,
}

#[derive(Debug, Deserialize)]
struct PriceApiResponse {
    pool_id: String,
    price: f64,
    price_24h_ago: f64,
    price_change_24h: f64,
    price_change_24h_percent: f64,
    volume_24h_usd: f64,
    volume_24h_token0: String,
    volume_24h_token1: String,
    txn_count_24h: u64,
}

#[derive(Debug, Deserialize)]
struct PositionsApiResponse {
    positions: Vec<ApiPositionData>,
}

#[derive(Debug, Deserialize)]
struct ApiPositionData {
    id: u64,
    pool_id: String,
    tick_lower: Option<i32>,
    tick_upper: Option<i32>,
    liquidity: String,
    amount0: String,
    amount1: String,
    fees_earned0: String,
    fees_earned1: String,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct MetricsApiResponse {
    tvl_usd: f64,
    volume_24h_usd: f64,
    fees_24h_usd: f64,
    apr: f64,
    utilization: f64,
    impermanent_loss: f64,
    price_impact: f64,
}
