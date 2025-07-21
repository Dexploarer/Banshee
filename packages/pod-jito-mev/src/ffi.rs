//! FFI bridge for TypeScript integration
//!
//! This module provides Foreign Function Interface (FFI) bindings to call
//! TypeScript functions from Rust using the Solana Agent Kit for Jito MEV operations.

#[cfg(not(test))]
mod ffi_impl {
    use serde::{Deserialize, Serialize};
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;

    /// FFI Result type returned from TypeScript
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FfiResult {
        pub success: bool,
        pub signature: Option<String>,
        pub data: Option<serde_json::Value>,
        pub error: Option<String>,
    }

    /// Configuration for Solana Agent Kit
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SolanaAgentConfig {
        pub private_key: String,
        pub rpc_url: String,
        pub openai_api_key: Option<String>,
    }

    /// Staking options
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StakingOptions {
        pub amount: f64,
        pub validator_vote_account: Option<String>,
    }

    /// MEV bundle options
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MevBundleOptions {
        pub bundle_type: String,
        pub transactions: Vec<String>,
        pub tip_sol: f64,
    }

    /// MEV scan options
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MevScanOptions {
        pub scan_type: String,
        pub min_profit_sol: f64,
    }

    /// Staking optimization options
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StakingOptimizationOptions {
        pub amount_sol: f64,
        pub strategy: String,
    }

    // External TypeScript functions (to be linked at runtime)
    extern "C" {
        fn initializeSolanaAgent(config_json: *const c_char) -> bool;
        fn stakeSOLFFI(options_json: *const c_char) -> *const c_char;
        fn restakeSOLFFI(options_json: *const c_char) -> *const c_char;
        fn submitMevBundleFFI(bundle_type: *const c_char, transactions_json: *const c_char, tip_sol: f64) -> *const c_char;
        fn scanMevOpportunitiesFFI(scan_type: *const c_char, min_profit_sol: f64) -> *const c_char;
        fn optimizeStakingFFI(amount_sol: f64, strategy: *const c_char) -> *const c_char;
        fn isInitializedFFI() -> bool;
        fn free_string(s: *const c_char);
    }

    /// Safe wrapper for initializing Solana Agent
    pub fn initialize_agent(config: &SolanaAgentConfig) -> Result<bool, String> {
        let config_json = serde_json::to_string(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        let c_config = CString::new(config_json)
            .map_err(|e| format!("Failed to create C string: {}", e))?;
        
        unsafe {
            Ok(initializeSolanaAgent(c_config.as_ptr()))
        }
    }

    /// Safe wrapper for staking SOL
    pub fn stake_sol(options: &StakingOptions) -> Result<FfiResult, String> {
        let options_json = serde_json::to_string(options)
            .map_err(|e| format!("Failed to serialize options: {}", e))?;
        
        let c_options = CString::new(options_json)
            .map_err(|e| format!("Failed to create C string: {}", e))?;
        
        unsafe {
            let result_ptr = stakeSOLFFI(c_options.as_ptr());
            if result_ptr.is_null() {
                return Err("FFI call returned null pointer".to_string());
            }
            
            let result_str = CStr::from_ptr(result_ptr)
                .to_str()
                .map_err(|e| format!("Failed to convert C string: {}", e))?;
            
            let result: FfiResult = serde_json::from_str(result_str)
                .map_err(|e| format!("Failed to parse result: {}", e))?;
            
            free_string(result_ptr);
            Ok(result)
        }
    }

    /// Safe wrapper for restaking SOL
    pub fn restake_sol(options: &serde_json::Value) -> Result<FfiResult, String> {
        let options_json = serde_json::to_string(options)
            .map_err(|e| format!("Failed to serialize options: {}", e))?;
        
        let c_options = CString::new(options_json)
            .map_err(|e| format!("Failed to create C string: {}", e))?;
        
        unsafe {
            let result_ptr = restakeSOLFFI(c_options.as_ptr());
            if result_ptr.is_null() {
                return Err("FFI call returned null pointer".to_string());
            }
            
            let result_str = CStr::from_ptr(result_ptr)
                .to_str()
                .map_err(|e| format!("Failed to convert C string: {}", e))?;
            
            let result: FfiResult = serde_json::from_str(result_str)
                .map_err(|e| format!("Failed to parse result: {}", e))?;
            
            free_string(result_ptr);
            Ok(result)
        }
    }

    /// Safe wrapper for submitting MEV bundle
    pub fn submit_mev_bundle(options: &MevBundleOptions) -> Result<FfiResult, String> {
        let c_bundle_type = CString::new(options.bundle_type.clone())
            .map_err(|e| format!("Failed to create C string: {}", e))?;
        
        let transactions_json = serde_json::to_string(&options.transactions)
            .map_err(|e| format!("Failed to serialize transactions: {}", e))?;
        
        let c_transactions = CString::new(transactions_json)
            .map_err(|e| format!("Failed to create C string: {}", e))?;
        
        unsafe {
            let result_ptr = submitMevBundleFFI(
                c_bundle_type.as_ptr(),
                c_transactions.as_ptr(),
                options.tip_sol
            );
            
            if result_ptr.is_null() {
                return Err("FFI call returned null pointer".to_string());
            }
            
            let result_str = CStr::from_ptr(result_ptr)
                .to_str()
                .map_err(|e| format!("Failed to convert C string: {}", e))?;
            
            let result: FfiResult = serde_json::from_str(result_str)
                .map_err(|e| format!("Failed to parse result: {}", e))?;
            
            free_string(result_ptr);
            Ok(result)
        }
    }

    /// Safe wrapper for scanning MEV opportunities
    pub fn scan_mev_opportunities(options: &MevScanOptions) -> Result<FfiResult, String> {
        let c_scan_type = CString::new(options.scan_type.clone())
            .map_err(|e| format!("Failed to create C string: {}", e))?;
        
        unsafe {
            let result_ptr = scanMevOpportunitiesFFI(
                c_scan_type.as_ptr(),
                options.min_profit_sol
            );
            
            if result_ptr.is_null() {
                return Err("FFI call returned null pointer".to_string());
            }
            
            let result_str = CStr::from_ptr(result_ptr)
                .to_str()
                .map_err(|e| format!("Failed to convert C string: {}", e))?;
            
            let result: FfiResult = serde_json::from_str(result_str)
                .map_err(|e| format!("Failed to parse result: {}", e))?;
            
            free_string(result_ptr);
            Ok(result)
        }
    }

    /// Safe wrapper for optimizing staking
    pub fn optimize_staking(options: &StakingOptimizationOptions) -> Result<FfiResult, String> {
        let c_strategy = CString::new(options.strategy.clone())
            .map_err(|e| format!("Failed to create C string: {}", e))?;
        
        unsafe {
            let result_ptr = optimizeStakingFFI(
                options.amount_sol,
                c_strategy.as_ptr()
            );
            
            if result_ptr.is_null() {
                return Err("FFI call returned null pointer".to_string());
            }
            
            let result_str = CStr::from_ptr(result_ptr)
                .to_str()
                .map_err(|e| format!("Failed to convert C string: {}", e))?;
            
            let result: FfiResult = serde_json::from_str(result_str)
                .map_err(|e| format!("Failed to parse result: {}", e))?;
            
            free_string(result_ptr);
            Ok(result)
        }
    }

    /// Check if Solana Agent is initialized
    pub fn is_agent_initialized() -> bool {
        unsafe {
            isInitializedFFI()
        }
    }
}

#[cfg(test)]
mod ffi_impl {
    use serde::{Deserialize, Serialize};

    /// FFI Result type returned from TypeScript
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FfiResult {
        pub success: bool,
        pub signature: Option<String>,
        pub data: Option<serde_json::Value>,
        pub error: Option<String>,
    }

    /// Configuration for Solana Agent Kit
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SolanaAgentConfig {
        pub private_key: String,
        pub rpc_url: String,
        pub openai_api_key: Option<String>,
    }

    /// Staking options
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StakingOptions {
        pub amount: f64,
        pub validator_vote_account: Option<String>,
    }

    /// MEV bundle options
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MevBundleOptions {
        pub bundle_type: String,
        pub transactions: Vec<String>,
        pub tip_sol: f64,
    }

    /// MEV scan options
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MevScanOptions {
        pub scan_type: String,
        pub min_profit_sol: f64,
    }

    /// Staking optimization options
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StakingOptimizationOptions {
        pub amount_sol: f64,
        pub strategy: String,
    }

    /// Test stub
    pub fn initialize_agent(_config: &SolanaAgentConfig) -> Result<bool, String> {
        Ok(true)
    }

    /// Test stub
    pub fn stake_sol(_options: &StakingOptions) -> Result<FfiResult, String> {
        Ok(FfiResult {
            success: true,
            signature: Some("test_signature".to_string()),
            data: Some(serde_json::json!({"message": "Test staking"})),
            error: None,
        })
    }

    /// Test stub
    pub fn restake_sol(_options: &serde_json::Value) -> Result<FfiResult, String> {
        Ok(FfiResult {
            success: true,
            signature: Some("test_signature".to_string()),
            data: Some(serde_json::json!({"message": "Test restaking"})),
            error: None,
        })
    }

    /// Test stub
    pub fn submit_mev_bundle(_options: &MevBundleOptions) -> Result<FfiResult, String> {
        Ok(FfiResult {
            success: true,
            signature: Some("test_bundle_id".to_string()),
            data: Some(serde_json::json!({
                "bundleId": "test_bundle_id",
                "estimatedProfit": 0.5
            })),
            error: None,
        })
    }

    /// Test stub
    pub fn scan_mev_opportunities(_options: &MevScanOptions) -> Result<FfiResult, String> {
        Ok(FfiResult {
            success: true,
            signature: None,
            data: Some(serde_json::json!({
                "opportunities": []
            })),
            error: None,
        })
    }

    /// Test stub
    pub fn optimize_staking(_options: &StakingOptimizationOptions) -> Result<FfiResult, String> {
        Ok(FfiResult {
            success: true,
            signature: None,
            data: Some(serde_json::json!({
                "validators": [],
                "totalExpectedApy": 7.5
            })),
            error: None,
        })
    }

    /// Test stub
    pub fn is_agent_initialized() -> bool {
        true
    }
}

// Re-export everything from the implementation module
pub use ffi_impl::*;