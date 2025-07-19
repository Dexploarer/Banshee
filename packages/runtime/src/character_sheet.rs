//! Character Sheet Runtime Loading System
//!
//! Provides dynamic loading and management of character sheets at runtime.
//! Character sheets can store secrets, MCP servers, templates, knowledge, and database configuration.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::embedded_db::{DatabaseType, EmbeddedDatabaseConfig};

/// Character sheet definition with full configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSheet {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub active: bool,

    // Core configuration
    pub database_config: DatabaseConfiguration,
    pub secrets: SecretsConfiguration,
    pub mcp_servers: McpServerConfiguration,
    pub templates: TemplateConfiguration,
    pub knowledge: KnowledgeConfiguration,

    // Agent behavior
    pub personality: PersonalityConfiguration,
    pub capabilities: CapabilityConfiguration,
    pub memory_settings: MemoryConfiguration,

    // Runtime settings
    pub performance: PerformanceConfiguration,
    pub logging: LoggingConfiguration,
    pub security: SecurityConfiguration,

    // Pod configuration
    pub pods: PodConfiguration,
}

/// Database configuration within character sheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfiguration {
    pub primary_db: DatabaseType,
    pub fallback_db: Option<DatabaseType>,
    pub data_path: PathBuf,
    pub memory_limit_mb: u64,
    pub max_connections: u32,
    pub enable_encryption: bool,
    pub backup_enabled: bool,
    pub sync_settings: Option<DatabaseSyncSettings>,
}

/// Database sync settings for LibSQL replication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSyncSettings {
    pub remote_url: String,
    pub auth_token: String,
    pub sync_interval_seconds: u64,
    pub conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    LastWriteWins,
    RemoteWins,
    LocalWins,
    Manual,
}

/// Secrets configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsConfiguration {
    pub vault_path: Option<PathBuf>,
    pub encryption_key: Option<String>,
    pub api_keys: HashMap<String, SecretValue>,
    pub wallets: HashMap<String, WalletSecret>,
    pub credentials: HashMap<String, CredentialSecret>,
    pub certificates: HashMap<String, CertificateSecret>,
}

/// Secret value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretValue {
    pub value: String,
    pub encrypted: bool,
    pub description: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub scopes: Vec<String>,
}

/// Wallet secret for Web3 operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSecret {
    pub mnemonic: Option<String>,
    pub private_key: Option<String>,
    pub public_key: String,
    pub address: String,
    pub network: String,
    pub encrypted: bool,
}

/// Credential secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSecret {
    pub username: String,
    pub password: String,
    pub endpoint: Option<String>,
    pub additional_fields: HashMap<String, String>,
}

/// Certificate secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSecret {
    pub certificate: String,
    pub private_key: String,
    pub ca_bundle: Option<String>,
    pub format: CertificateFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CertificateFormat {
    Pem,
    Der,
    Pkcs12,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfiguration {
    pub servers: HashMap<String, McpServer>,
    pub default_timeout_seconds: u64,
    pub max_concurrent_connections: u32,
    pub retry_attempts: u32,
    pub health_check_interval_seconds: u64,
}

/// Individual MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub endpoint: String,
    pub auth_type: McpAuthType,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub enabled: bool,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpAuthType {
    None,
    ApiKey { key: String },
    Bearer { token: String },
    Basic { username: String, password: String },
    Custom { headers: HashMap<String, String> },
}

/// Template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfiguration {
    pub prompt_templates: HashMap<String, PromptTemplate>,
    pub response_templates: HashMap<String, ResponseTemplate>,
    pub workflow_templates: HashMap<String, WorkflowTemplate>,
    pub default_language: String,
    pub template_engine: TemplateEngine,
}

/// Prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub template: String,
    pub variables: Vec<TemplateVariable>,
    pub category: String,
    pub description: Option<String>,
    pub version: String,
}

/// Response template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTemplate {
    pub name: String,
    pub template: String,
    pub format: ResponseFormat,
    pub conditions: Vec<TemplateCondition>,
}

/// Workflow template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub name: String,
    pub steps: Vec<WorkflowStep>,
    pub parallel_execution: bool,
    pub error_handling: ErrorHandlingStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub action: WorkflowAction,
    pub depends_on: Vec<String>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowAction {
    McpCall {
        server: String,
        method: String,
        params: serde_json::Value,
    },
    DatabaseQuery {
        query: String,
        params: serde_json::Value,
    },
    EmotionUpdate {
        emotion: String,
        intensity: f64,
    },
    MemoryStore {
        collection: String,
        data: serde_json::Value,
    },
    WebRequest {
        url: String,
        method: String,
        body: Option<serde_json::Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandlingStrategy {
    Stop,
    Continue,
    Retry {
        max_attempts: u32,
        delay_seconds: u64,
    },
    Fallback {
        action: Box<WorkflowAction>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub var_type: VariableType,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub validation: Option<VariableValidation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    DateTime,
    Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableValidation {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub allowed_values: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseFormat {
    Text,
    Json,
    Markdown,
    Html,
    Custom { format: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateCondition {
    pub condition: String,
    pub action: ConditionAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionAction {
    UseTemplate {
        template: String,
    },
    ModifyResponse {
        modifications: Vec<String>,
    },
    AddMetadata {
        metadata: HashMap<String, serde_json::Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateEngine {
    Handlebars,
    Tera,
    Liquid,
    Custom { engine: String },
}

/// Knowledge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeConfiguration {
    pub knowledge_bases: HashMap<String, KnowledgeBase>,
    pub vector_search_enabled: bool,
    pub embedding_model: Option<String>,
    pub chunk_size: usize,
    pub overlap_size: usize,
    pub indexing_strategy: IndexingStrategy,
}

/// Knowledge base definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub name: String,
    pub description: Option<String>,
    pub sources: Vec<KnowledgeSource>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub update_frequency: UpdateFrequency,
    pub access_level: AccessLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSource {
    File {
        path: PathBuf,
        format: FileFormat,
    },
    Url {
        url: String,
        headers: HashMap<String, String>,
    },
    Database {
        query: String,
        connection: String,
    },
    Api {
        endpoint: String,
        auth: McpAuthType,
    },
    Memory {
        collection: String,
        filters: HashMap<String, serde_json::Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileFormat {
    Text,
    Markdown,
    Json,
    Yaml,
    Toml,
    Csv,
    Pdf,
    Docx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateFrequency {
    Manual,
    OnStartup,
    Hourly,
    Daily,
    Weekly,
    OnChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    Public,
    Private,
    Restricted { allowed_scopes: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexingStrategy {
    FullText,
    Vector,
    Hybrid,
    Semantic,
}

/// Personality configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityConfiguration {
    pub big_five: BigFiveTraits,
    pub communication_style: CommunicationStyle,
    pub emotional_profile: EmotionalProfile,
    pub decision_making: DecisionMakingStyle,
    pub learning_preferences: LearningPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigFiveTraits {
    pub openness: f64,
    pub conscientiousness: f64,
    pub extraversion: f64,
    pub agreeableness: f64,
    pub neuroticism: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStyle {
    pub formality: FormalityLevel,
    pub verbosity: VerbosityLevel,
    pub emotion_expression: EmotionExpression,
    pub humor_level: HumorLevel,
    pub technical_depth: TechnicalDepth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormalityLevel {
    VeryFormal,
    Formal,
    Neutral,
    Casual,
    VeryCasual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerbosityLevel {
    Terse,
    Concise,
    Moderate,
    Detailed,
    Verbose,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmotionExpression {
    Suppressed,
    Minimal,
    Moderate,
    Expressive,
    VeryExpressive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HumorLevel {
    None,
    Subtle,
    Moderate,
    Frequent,
    Constant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TechnicalDepth {
    Layperson,
    Basic,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalProfile {
    pub default_mood: f64,
    pub emotional_volatility: f64,
    pub empathy_level: f64,
    pub stress_tolerance: f64,
    pub optimism_bias: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionMakingStyle {
    Analytical,
    Intuitive,
    Collaborative,
    Directive,
    Conceptual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPreferences {
    pub learning_rate: f64,
    pub memory_retention: f64,
    pub adaptation_speed: f64,
    pub curiosity_level: f64,
    pub risk_tolerance: f64,
}

/// Capability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityConfiguration {
    pub enabled_capabilities: Vec<String>,
    pub capability_limits: HashMap<String, CapabilityLimit>,
    pub tool_access: ToolAccessConfiguration,
    pub permissions: PermissionConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityLimit {
    pub max_calls_per_minute: Option<u32>,
    pub max_calls_per_hour: Option<u32>,
    pub max_data_size_mb: Option<u64>,
    pub timeout_seconds: Option<u64>,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAccessConfiguration {
    pub web_access: bool,
    pub file_system_access: bool,
    pub database_access: bool,
    pub network_access: bool,
    pub external_apis: bool,
    pub blockchain_access: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfiguration {
    pub read_permissions: Vec<String>,
    pub write_permissions: Vec<String>,
    pub execute_permissions: Vec<String>,
    pub admin_permissions: Vec<String>,
    pub restricted_paths: Vec<PathBuf>,
    pub allowed_domains: Vec<String>,
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfiguration {
    pub max_memory_mb: u64,
    pub conversation_history_limit: u64,
    pub emotional_memory_days: u64,
    pub knowledge_cache_size_mb: u64,
    pub auto_cleanup_enabled: bool,
    pub compression_enabled: bool,
    pub backup_frequency: BackupFrequency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupFrequency {
    Never,
    OnShutdown,
    Hourly,
    Daily,
    Weekly,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfiguration {
    pub max_concurrent_tasks: u32,
    pub request_timeout_seconds: u64,
    pub batch_processing_enabled: bool,
    pub cache_strategy: CacheStrategy,
    pub optimization_level: OptimizationLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheStrategy {
    None,
    Memory,
    Disk,
    Hybrid,
    Distributed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    None,
    Basic,
    Moderate,
    Aggressive,
    Maximum,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfiguration {
    pub log_level: LogLevel,
    pub log_to_file: bool,
    pub log_file_path: Option<PathBuf>,
    pub max_log_file_size_mb: u64,
    pub log_rotation_count: u32,
    pub structured_logging: bool,
    pub sensitive_data_filtering: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfiguration {
    pub encryption_enabled: bool,
    pub secure_communication: bool,
    pub certificate_validation: bool,
    pub access_control_enabled: bool,
    pub audit_logging: bool,
    pub data_retention_days: u64,
    pub anonymization_enabled: bool,
}

/// Pod configuration within character sheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodConfiguration {
    pub enabled_pods: HashMap<String, PodInstanceConfig>,
    pub pod_priorities: HashMap<String, u32>,
    pub auto_dependency_resolution: bool,
    pub pod_health_check_interval_seconds: u64,
    pub pod_listeners: Vec<PodListener>,
}

/// Configuration for a specific pod instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodInstanceConfig {
    pub enabled: bool,
    pub config_overrides: HashMap<String, serde_json::Value>,
    pub dependencies: Vec<String>,
    pub priority: u32,
    pub health_check_enabled: bool,
    pub restart_on_failure: bool,
    pub defi_config: Option<DefiPodConfig>,
}

/// DeFi-specific pod configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefiPodConfig {
    pub networks: Vec<String>,
    pub slippage_tolerance: f64,
    pub max_gas_price_gwei: u64,
    pub wallet_config: WalletConfig,
    pub risk_settings: RiskSettings,
    pub emotional_trading: EmotionalTradingConfig,
}

/// Wallet configuration for DeFi operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub wallet_name: String,
    pub auto_approve_small_amounts: bool,
    pub daily_spending_limit_usd: f64,
    pub require_confirmation_above_usd: f64,
}

/// Risk management settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSettings {
    pub max_position_size_percent: f64,
    pub stop_loss_percent: Option<f64>,
    pub take_profit_percent: Option<f64>,
    pub diversification_required: bool,
    pub blacklisted_tokens: Vec<String>,
    pub whitelisted_tokens: Vec<String>,
}

/// Emotional trading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalTradingConfig {
    pub enabled: bool,
    pub fear_greed_threshold: f64,
    pub excitement_trade_multiplier: f64,
    pub anxiety_pause_trading: bool,
    pub confidence_leverage_factor: f64,
}

/// Pod listener configuration for MCP integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodListener {
    pub pod_id: String,
    pub tool_patterns: Vec<String>,
    pub server_names: Vec<String>,
    pub priority: u32,
    pub enabled: bool,
}

/// Character sheet manager
pub struct CharacterSheetManager {
    sheets: HashMap<Uuid, CharacterSheet>,
    active_sheet: Option<Uuid>,
    config_directory: PathBuf,
}

impl CharacterSheetManager {
    /// Create new character sheet manager
    pub fn new(config_directory: PathBuf) -> Self {
        Self {
            sheets: HashMap::new(),
            active_sheet: None,
            config_directory,
        }
    }

    /// Load all character sheets from directory
    pub async fn load_all_sheets(&mut self) -> Result<()> {
        info!("Loading character sheets from {:?}", self.config_directory);

        // Ensure config directory exists
        if !self.config_directory.exists() {
            std::fs::create_dir_all(&self.config_directory)
                .context("Failed to create config directory")?;
            info!("Created config directory: {:?}", self.config_directory);
        }

        // Load all .toml files in the directory
        let entries =
            std::fs::read_dir(&self.config_directory).context("Failed to read config directory")?;

        let mut loaded_count = 0;
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                match self.load_sheet_from_file(&path).await {
                    Ok(sheet) => {
                        info!("Loaded character sheet: {} ({})", sheet.name, sheet.id);
                        self.sheets.insert(sheet.id, sheet);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        error!("Failed to load character sheet from {:?}: {}", path, e);
                    }
                }
            }
        }

        info!("Loaded {} character sheets", loaded_count);

        // Set first active sheet as default if none set
        if self.active_sheet.is_none() && !self.sheets.is_empty() {
            let first_id = *self.sheets.keys().next().unwrap();
            self.set_active_sheet(first_id)?;
        }

        Ok(())
    }

    /// Load character sheet from file
    async fn load_sheet_from_file(&self, path: &PathBuf) -> Result<CharacterSheet> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {:?}", path))?;

        let sheet: CharacterSheet = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML: {:?}", path))?;

        Ok(sheet)
    }

    /// Save character sheet to file
    pub async fn save_sheet(&self, sheet: &CharacterSheet) -> Result<()> {
        let filename = format!("{}.toml", sheet.name.replace(' ', "_").to_lowercase());
        let path = self.config_directory.join(filename);

        let content =
            toml::to_string_pretty(sheet).context("Failed to serialize character sheet")?;

        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write file: {:?}", path))?;

        info!("Saved character sheet: {} to {:?}", sheet.name, path);
        Ok(())
    }

    /// Create default character sheet
    pub fn create_default_sheet() -> CharacterSheet {
        let now = chrono::Utc::now();
        let id = Uuid::new_v4();

        CharacterSheet {
            id,
            name: "Default Agent".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Default character sheet with basic configuration".to_string()),
            created_at: now,
            updated_at: now,
            active: true,

            database_config: DatabaseConfiguration {
                primary_db: DatabaseType::SurrealRocks,
                fallback_db: Some(DatabaseType::LibSQL),
                data_path: PathBuf::from("./data/agent.db"),
                memory_limit_mb: 512,
                max_connections: 10,
                enable_encryption: false,
                backup_enabled: true,
                sync_settings: None,
            },

            secrets: SecretsConfiguration {
                vault_path: None,
                encryption_key: None,
                api_keys: HashMap::new(),
                wallets: HashMap::new(),
                credentials: HashMap::new(),
                certificates: HashMap::new(),
            },

            mcp_servers: McpServerConfiguration {
                servers: HashMap::new(),
                default_timeout_seconds: 30,
                max_concurrent_connections: 5,
                retry_attempts: 3,
                health_check_interval_seconds: 60,
            },

            templates: TemplateConfiguration {
                prompt_templates: HashMap::new(),
                response_templates: HashMap::new(),
                workflow_templates: HashMap::new(),
                default_language: "en".to_string(),
                template_engine: TemplateEngine::Handlebars,
            },

            knowledge: KnowledgeConfiguration {
                knowledge_bases: HashMap::new(),
                vector_search_enabled: false,
                embedding_model: None,
                chunk_size: 1000,
                overlap_size: 200,
                indexing_strategy: IndexingStrategy::FullText,
            },

            personality: PersonalityConfiguration {
                big_five: BigFiveTraits {
                    openness: 0.7,
                    conscientiousness: 0.8,
                    extraversion: 0.6,
                    agreeableness: 0.7,
                    neuroticism: 0.3,
                },
                communication_style: CommunicationStyle {
                    formality: FormalityLevel::Neutral,
                    verbosity: VerbosityLevel::Moderate,
                    emotion_expression: EmotionExpression::Moderate,
                    humor_level: HumorLevel::Subtle,
                    technical_depth: TechnicalDepth::Intermediate,
                },
                emotional_profile: EmotionalProfile {
                    default_mood: 0.6,
                    emotional_volatility: 0.4,
                    empathy_level: 0.7,
                    stress_tolerance: 0.6,
                    optimism_bias: 0.5,
                },
                decision_making: DecisionMakingStyle::Analytical,
                learning_preferences: LearningPreferences {
                    learning_rate: 0.7,
                    memory_retention: 0.8,
                    adaptation_speed: 0.6,
                    curiosity_level: 0.8,
                    risk_tolerance: 0.4,
                },
            },

            capabilities: CapabilityConfiguration {
                enabled_capabilities: vec![
                    "conversation".to_string(),
                    "memory".to_string(),
                    "emotion".to_string(),
                ],
                capability_limits: HashMap::new(),
                tool_access: ToolAccessConfiguration {
                    web_access: false,
                    file_system_access: false,
                    database_access: true,
                    network_access: false,
                    external_apis: false,
                    blockchain_access: false,
                },
                permissions: PermissionConfiguration {
                    read_permissions: vec!["memory".to_string()],
                    write_permissions: vec!["memory".to_string()],
                    execute_permissions: vec![],
                    admin_permissions: vec![],
                    restricted_paths: vec![],
                    allowed_domains: vec![],
                },
            },

            memory_settings: MemoryConfiguration {
                max_memory_mb: 256,
                conversation_history_limit: 1000,
                emotional_memory_days: 30,
                knowledge_cache_size_mb: 128,
                auto_cleanup_enabled: true,
                compression_enabled: true,
                backup_frequency: BackupFrequency::Daily,
            },

            performance: PerformanceConfiguration {
                max_concurrent_tasks: 5,
                request_timeout_seconds: 30,
                batch_processing_enabled: true,
                cache_strategy: CacheStrategy::Memory,
                optimization_level: OptimizationLevel::Moderate,
            },

            logging: LoggingConfiguration {
                log_level: LogLevel::Info,
                log_to_file: true,
                log_file_path: Some(PathBuf::from("./logs/agent.log")),
                max_log_file_size_mb: 100,
                log_rotation_count: 5,
                structured_logging: true,
                sensitive_data_filtering: true,
            },

            security: SecurityConfiguration {
                encryption_enabled: false,
                secure_communication: true,
                certificate_validation: true,
                access_control_enabled: true,
                audit_logging: true,
                data_retention_days: 90,
                anonymization_enabled: false,
            },

            pods: PodConfiguration {
                enabled_pods: HashMap::new(),
                pod_priorities: HashMap::new(),
                auto_dependency_resolution: true,
                pod_health_check_interval_seconds: 30,
                pod_listeners: vec![],
            },
        }
    }

    /// Set active character sheet
    pub fn set_active_sheet(&mut self, sheet_id: Uuid) -> Result<()> {
        if !self.sheets.contains_key(&sheet_id) {
            return Err(anyhow::anyhow!("Character sheet not found: {}", sheet_id));
        }

        self.active_sheet = Some(sheet_id);
        info!("Set active character sheet: {}", sheet_id);
        Ok(())
    }

    /// Get active character sheet
    pub fn get_active_sheet(&self) -> Result<&CharacterSheet> {
        let sheet_id = self
            .active_sheet
            .ok_or_else(|| anyhow::anyhow!("No active character sheet"))?;

        self.sheets
            .get(&sheet_id)
            .ok_or_else(|| anyhow::anyhow!("Active character sheet not found"))
    }

    /// Get character sheet by ID
    pub fn get_sheet(&self, sheet_id: Uuid) -> Option<&CharacterSheet> {
        self.sheets.get(&sheet_id)
    }

    /// List all character sheets
    pub fn list_sheets(&self) -> Vec<&CharacterSheet> {
        self.sheets.values().collect()
    }

    /// Add or update character sheet
    pub fn upsert_sheet(&mut self, sheet: CharacterSheet) {
        let sheet_id = sheet.id;
        self.sheets.insert(sheet_id, sheet);
    }

    /// Remove character sheet
    pub fn remove_sheet(&mut self, sheet_id: Uuid) -> Result<()> {
        if self.active_sheet == Some(sheet_id) {
            self.active_sheet = None;
        }

        self.sheets
            .remove(&sheet_id)
            .ok_or_else(|| anyhow::anyhow!("Character sheet not found: {}", sheet_id))?;

        Ok(())
    }

    /// Convert character sheet database config to embedded database config
    pub fn to_embedded_db_config(sheet: &CharacterSheet) -> EmbeddedDatabaseConfig {
        EmbeddedDatabaseConfig {
            db_type: sheet.database_config.primary_db.clone(),
            data_path: sheet.database_config.data_path.clone(),
            memory_limit_mb: Some(sheet.database_config.memory_limit_mb),
            enable_wal: true,
            max_connections: sheet.database_config.max_connections,
            query_timeout: std::time::Duration::from_secs(30),
            sync_interval: sheet
                .database_config
                .sync_settings
                .as_ref()
                .map(|s| std::time::Duration::from_secs(s.sync_interval_seconds)),
            encryption_key: sheet.secrets.encryption_key.clone(),
            backup_interval: if sheet.database_config.backup_enabled {
                Some(std::time::Duration::from_secs(3600))
            } else {
                None
            },
            compress_data: sheet.memory_settings.compression_enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_character_sheet_creation() {
        let sheet = CharacterSheetManager::create_default_sheet();
        assert_eq!(sheet.name, "Default Agent");
        assert_eq!(sheet.version, "1.0.0");
        assert!(sheet.active);
        assert_eq!(sheet.database_config.primary_db, DatabaseType::SurrealRocks);
    }

    #[tokio::test]
    async fn test_character_sheet_serialization() {
        let sheet = CharacterSheetManager::create_default_sheet();
        let serialized = toml::to_string(&sheet).unwrap();
        let deserialized: CharacterSheet = toml::from_str(&serialized).unwrap();
        assert_eq!(sheet.id, deserialized.id);
        assert_eq!(sheet.name, deserialized.name);
    }

    #[tokio::test]
    async fn test_character_sheet_manager() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = CharacterSheetManager::new(temp_dir.path().to_path_buf());

        // Should have no sheets initially
        assert!(manager.get_active_sheet().is_err());
        assert_eq!(manager.list_sheets().len(), 0);

        // Add a sheet
        let sheet = CharacterSheetManager::create_default_sheet();
        let sheet_id = sheet.id;
        manager.upsert_sheet(sheet);

        // Set as active
        manager.set_active_sheet(sheet_id).unwrap();
        assert_eq!(manager.get_active_sheet().unwrap().id, sheet_id);
    }

    #[tokio::test]
    async fn test_database_config_conversion() {
        let sheet = CharacterSheetManager::create_default_sheet();
        let db_config = CharacterSheetManager::to_embedded_db_config(&sheet);

        assert_eq!(db_config.db_type, sheet.database_config.primary_db);
        assert_eq!(db_config.data_path, sheet.database_config.data_path);
        assert_eq!(
            db_config.max_connections,
            sheet.database_config.max_connections
        );
    }
}
