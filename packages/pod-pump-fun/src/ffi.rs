//! FFI bridge for TypeScript integration
//!
//! This module provides Foreign Function Interface (FFI) bindings to call
//! TypeScript functions from Rust using the Solana Agent Kit.

use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// FFI Result type returned from TypeScript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FfiResult {
    pub success: bool,
    pub signature: Option<String>,
    pub mint: Option<String>,
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

/// Token deployment options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDeployOptions {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
    pub initial_supply: u64,
}

/// Token swap options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapOptions {
    pub from_token_mint: String,
    pub to_token_mint: String,
    pub amount: f64,
    pub slippage: f64,
}

// External TypeScript functions (to be linked at runtime)
extern "C" {
    fn initializeSolanaAgent(config_json: *const c_char) -> bool;
    fn deployTokenFFI(options_json: *const c_char) -> *const c_char;
    fn swapTokensFFI(options_json: *const c_char) -> *const c_char;
    fn getTokenInfoFFI(token_address: *const c_char) -> *const c_char;
    fn getTrendingTokensFFI() -> *const c_char;
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

/// Safe wrapper for deploying token
pub fn deploy_token(options: &TokenDeployOptions) -> Result<FfiResult, String> {
    let options_json = serde_json::to_string(options)
        .map_err(|e| format!("Failed to serialize options: {}", e))?;
    
    let c_options = CString::new(options_json)
        .map_err(|e| format!("Failed to create C string: {}", e))?;
    
    unsafe {
        let result_ptr = deployTokenFFI(c_options.as_ptr());
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

/// Safe wrapper for swapping tokens
pub fn swap_tokens(options: &SwapOptions) -> Result<FfiResult, String> {
    let options_json = serde_json::to_string(options)
        .map_err(|e| format!("Failed to serialize options: {}", e))?;
    
    let c_options = CString::new(options_json)
        .map_err(|e| format!("Failed to create C string: {}", e))?;
    
    unsafe {
        let result_ptr = swapTokensFFI(c_options.as_ptr());
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

/// Safe wrapper for getting token info
pub fn get_token_info(token_address: &str) -> Result<FfiResult, String> {
    let c_address = CString::new(token_address)
        .map_err(|e| format!("Failed to create C string: {}", e))?;
    
    unsafe {
        let result_ptr = getTokenInfoFFI(c_address.as_ptr());
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

/// Safe wrapper for getting trending tokens
pub fn get_trending_tokens() -> Result<FfiResult, String> {
    unsafe {
        let result_ptr = getTrendingTokensFFI();
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

/// Check if agent is initialized
pub fn is_agent_initialized() -> bool {
    unsafe {
        isInitializedFFI()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_serialization() {
        let config = SolanaAgentConfig {
            private_key: "test_key".to_string(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
            openai_api_key: Some("test_api_key".to_string()),
        };
        
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("test_key"));
        assert!(json.contains("https://api.devnet.solana.com"));
    }
}