//! # Emotional Agents Runtime
//!
//! Advanced runtime system integrating AI SDK 5, MCP servers, emotion-based
//! decision making, and sophisticated memory/relationship management.

#![allow(dead_code)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::new_without_default)]
#![allow(clippy::unwrap_or_default)]

pub mod ai_sdk_client;
pub mod config;
pub mod decision;
pub mod integration;
pub mod mcp_manager;
pub mod memory;
pub mod relationships;
pub mod runtime;
pub mod utils;

// Re-export specific items to avoid conflicts
pub use config::{
    AiSdkConfig, CacheConfig, ConsolidationConfig, DecisionConfig, ErrorHandlingConfig,
    ExecutionLimits, KnowledgeGraphConfig, McpAuth, McpConfig, McpServerConfig, MemoryConfig,
    MonitoringConfig, RelationshipConfig, RelationshipDecayConfig, RuntimeConfig, RuntimeSettings,
    StandingMethod, StorageConfig, ToolDiscoveryConfig, TrustConfig,
};
pub use decision::{
    ConstraintPriority, ConstraintType, DecisionChoice, DecisionConstraint, DecisionContext,
    EmotionalDecisionEngine, ExpectedOutcome, RiskLevel, ScoredOption,
};
pub use integration::{AiSdkIntegration, McpIntegration};
pub use memory::{
    KnowledgeEdge, KnowledgeEdgeType, KnowledgeGraph, KnowledgeGraphMemory, KnowledgeNode,
    KnowledgeNodeType,
};
pub use relationships::{
    InteractionRecord, InteractionType, Relationship, RelationshipManager, RelationshipStanding,
    StandingCategory,
};
pub use runtime::{
    AgentInfo, AgentInstance, AiResponse, DecisionFactors, DecisionOption, DecisionRecord,
    EmotionalAgentsRuntime, ResponseContext, ToolCall, ToolResult,
};

use emotional_agents_core::Result;
