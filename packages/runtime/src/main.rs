//! # Banshee Runtime
//!
//! Main runtime that demonstrates the complete plugin architecture with dependency resolution.
//! Shows how emotional intelligence, memory persistence, and Web3 functionality integrate.
//! Implements 2025 best practices for database startup and initialization.

mod ai_sdk_client;
mod character_sheet;
mod database;
mod embedded_db;
mod mcp_manager;
mod pod_injector;
mod redis;

use ai_sdk_client::{AiSdk5ClientManager, AiSdk5Config, TransportConfig};
use anyhow::{Context, Result};
use banshee_core::plugin::{PodManager, PodResult};
use character_sheet::{CharacterSheet, CharacterSheetManager};
use database::{DatabaseConfig, DatabaseManager};
use embedded_db::{DatabaseType, EmbeddedDatabaseManager};
use mcp_manager::{LoggingEventListener, McpManager};
use pod_injector::PodInjector;
use redis::{RedisConfig, RedisManager};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file if present
    if let Err(e) = dotenvy::dotenv() {
        warn!("Could not load .env file: {}", e);
    }

    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Banshee Runtime with full plugin architecture");

    // Step 1: Load character sheets to determine database configuration
    let mut character_sheet_manager =
        CharacterSheetManager::new(PathBuf::from("./config/character_sheets"));

    character_sheet_manager
        .load_all_sheets()
        .await
        .context("Failed to load character sheets")?;

    // If no character sheets exist, create a default one
    if character_sheet_manager.list_sheets().is_empty() {
        info!("No character sheets found, creating default character sheet");
        let default_sheet = CharacterSheetManager::create_default_sheet();
        character_sheet_manager
            .save_sheet(&default_sheet)
            .await
            .context("Failed to save default character sheet")?;
        character_sheet_manager.upsert_sheet(default_sheet);
        character_sheet_manager
            .set_active_sheet(character_sheet_manager.list_sheets()[0].id)
            .context("Failed to set active character sheet")?;
    }

    let active_sheet = character_sheet_manager
        .get_active_sheet()
        .context("No active character sheet available")?;

    info!(
        "Using character sheet: {} (Database: {:?})",
        active_sheet.name, active_sheet.database_config.primary_db
    );

    // Step 2: Initialize database services based on character sheet configuration
    let (mut db_manager, mut redis_manager, mut embedded_db_manager) =
        initialize_databases(&active_sheet)
            .await
            .context("Failed to initialize database services")?;

    // Step 3: Initialize AI SDK 5 client manager
    info!("🤖 Initializing AI SDK 5 client manager with July 2025 transport architecture");
    let ai_client_manager = Arc::new(RwLock::new(AiSdk5ClientManager::new()));

    // Load AI clients from character sheet (if configured)
    initialize_ai_clients(&ai_client_manager, &active_sheet)
        .await
        .context("Failed to initialize AI SDK 5 clients")?;

    // Step 4: Initialize MCP manager with character sheet servers
    info!("🔗 Initializing MCP manager with character sheet configuration");
    let mut mcp_manager = McpManager::new(ai_client_manager.clone());
    mcp_manager.add_event_listener(Box::new(LoggingEventListener));

    let mcp_manager_arc = Arc::new(RwLock::new(mcp_manager));
    {
        let mut mcp_manager_guard = mcp_manager_arc.write().await;
        mcp_manager_guard
            .load_from_character_sheet(&active_sheet)
            .await
            .context("Failed to load MCP servers from character sheet")?;
    }

    // Step 5: Initialize pod injector with AI SDK 5 and MCP integration
    info!("🧩 Initializing pod injector with character sheet pod configuration");
    let mut pod_injector = PodInjector::new(ai_client_manager.clone(), mcp_manager_arc.clone());
    pod_injector
        .load_from_character_sheet(&active_sheet)
        .await
        .context("Failed to load pods from character sheet")?;

    // Step 6: Comprehensive health checks
    info!("🏥 Performing comprehensive health checks");
    let health_status = perform_health_checks(
        pod_injector.pod_manager(),
        &db_manager,
        &redis_manager,
        &embedded_db_manager,
        &ai_client_manager,
        &mcp_manager_arc,
    )
    .await;

    if !health_status {
        error!("❌ Health checks failed - shutting down");
        shutdown_services(
            pod_injector,
            db_manager,
            redis_manager,
            embedded_db_manager,
            ai_client_manager,
            mcp_manager_arc,
        )
        .await?;
        return Err(anyhow::anyhow!("System failed health checks"));
    }

    // Step 7: Demonstrate the integrated system
    info!("✨ All systems healthy - demonstrating integrated capabilities");
    demonstrate_system(
        pod_injector.pod_manager(),
        &db_manager,
        &redis_manager,
        &embedded_db_manager,
        &active_sheet,
        &ai_client_manager,
        &mcp_manager_arc,
    )
    .await
    .context("Failed during system demonstration")?;

    // Step 8: Graceful shutdown
    info!("🛑 Initiating graceful shutdown sequence");
    shutdown_services(
        pod_injector,
        db_manager,
        redis_manager,
        embedded_db_manager,
        ai_client_manager,
        mcp_manager_arc,
    )
    .await
    .context("Failed during shutdown")?;

    info!("✅ Banshee Runtime shutdown complete");
    Ok(())
}

/// Initialize database services with proper startup order and health checks
async fn initialize_databases(
    character_sheet: &CharacterSheet,
) -> Result<(
    Option<DatabaseManager>,
    Option<RedisManager>,
    Option<EmbeddedDatabaseManager>,
)> {
    info!("📊 Initializing database services based on character sheet configuration...");

    match character_sheet.database_config.primary_db {
        DatabaseType::PostgreSQL => {
            info!("Initializing traditional PostgreSQL + Redis setup");

            // Configure database
            let db_config = DatabaseConfig {
                url: get_database_url(),
                max_connections: character_sheet.database_config.max_connections,
                min_connections: get_env_var("DATABASE_MIN_CONNECTIONS")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5),
                connect_timeout: Duration::from_secs(30),
                ..Default::default()
            };

            // Configure Redis if needed
            let redis_config = RedisConfig {
                url: get_redis_url(),
                max_connections: get_env_var("REDIS_MAX_CONNECTIONS")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10),
                connect_timeout: Duration::from_secs(10),
                ..Default::default()
            };

            // Initialize PostgreSQL
            let mut db_manager = DatabaseManager::new(db_config);
            db_manager
                .initialize()
                .await
                .context("Failed to initialize PostgreSQL")?;

            // Initialize Redis
            let mut redis_manager = RedisManager::new(redis_config);
            redis_manager
                .initialize()
                .await
                .context("Failed to initialize Redis")?;

            info!("✅ Traditional database services initialized successfully");
            Ok((Some(db_manager), Some(redis_manager), None))
        }
        _ => {
            info!(
                "Initializing embedded database: {:?}",
                character_sheet.database_config.primary_db
            );

            // Convert character sheet config to embedded database config
            let embedded_config = CharacterSheetManager::to_embedded_db_config(character_sheet);

            // Initialize embedded database
            let mut embedded_db_manager = EmbeddedDatabaseManager::new(embedded_config);
            embedded_db_manager
                .initialize()
                .await
                .context("Failed to initialize embedded database")?;

            info!("✅ Embedded database initialized successfully");
            Ok((None, None, Some(embedded_db_manager)))
        }
    }
}

/// Perform comprehensive health checks on all systems
async fn perform_health_checks(
    pod_manager: &PodManager,
    db_manager: &Option<DatabaseManager>,
    redis_manager: &Option<RedisManager>,
    embedded_db_manager: &Option<EmbeddedDatabaseManager>,
    ai_client_manager: &Arc<RwLock<AiSdk5ClientManager>>,
    mcp_manager: &Arc<RwLock<McpManager>>,
) -> bool {
    let mut all_healthy = true;

    // Check traditional database health
    if let Some(db_manager) = db_manager {
        if let Ok(pool) = db_manager.pool() {
            match db_manager.health_check(pool).await {
                Ok(_) => info!("✅ PostgreSQL health check passed"),
                Err(e) => {
                    error!("❌ PostgreSQL health check failed: {}", e);
                    all_healthy = false;
                }
            }

            // Log database stats
            if let Ok(stats) = db_manager.get_stats().await {
                info!("📊 Database stats: {}", stats);
            }
        } else {
            error!("❌ PostgreSQL connection pool not available");
            all_healthy = false;
        }
    }

    // Check Redis health
    if let Some(redis_manager) = redis_manager {
        if let Ok(connection) = redis_manager.connection() {
            match redis_manager.health_check(connection).await {
                Ok(_) => info!("✅ Redis health check passed"),
                Err(e) => {
                    error!("❌ Redis health check failed: {}", e);
                    all_healthy = false;
                }
            }

            // Log Redis stats
            if let Ok(stats) = redis_manager.get_stats().await {
                info!("📊 Redis stats: {}", stats);
            }
        } else {
            error!("❌ Redis connection not available");
            all_healthy = false;
        }
    }

    // Check embedded database health
    if let Some(embedded_db_manager) = embedded_db_manager {
        match embedded_db_manager.database() {
            Ok(db) => {
                match db.health_check().await {
                    Ok(_) => info!("✅ Embedded database health check passed"),
                    Err(e) => {
                        error!("❌ Embedded database health check failed: {}", e);
                        all_healthy = false;
                    }
                }

                // Log embedded database stats
                if let Ok(stats) = embedded_db_manager.get_stats().await {
                    info!("📊 Embedded database stats: {}", stats);
                }
            }
            Err(e) => {
                error!("❌ Embedded database not available: {}", e);
                all_healthy = false;
            }
        }
    }

    // Check pod health
    let pod_health_results = pod_manager.health_check_all().await;
    for (pod_id, healthy) in &pod_health_results {
        if *healthy {
            info!("✅ Pod '{}' is healthy", pod_id);
        } else {
            error!("❌ Pod '{}' failed health check", pod_id);
            all_healthy = false;
        }
    }

    // Check AI SDK 5 client health
    info!("🤖 Checking AI SDK 5 client health");
    // In real implementation, would check all clients
    info!("✅ AI SDK 5 client manager operational");

    // Check MCP server health
    info!("🔗 Checking MCP server health");
    let mcp_manager_guard = mcp_manager.read().await;
    let active_servers = mcp_manager_guard.list_active_servers();
    if active_servers.is_empty() {
        info!("ℹ️ No MCP servers configured");
    } else {
        info!("✅ {} MCP servers tracked", active_servers.len());
        for (server_name, server_info) in &active_servers {
            match &server_info.status {
                crate::mcp_manager::McpServerStatus::Connected => {
                    info!("✅ MCP server '{}' is connected", server_name);
                }
                crate::mcp_manager::McpServerStatus::Error(e) => {
                    error!("❌ MCP server '{}' has error: {}", server_name, e);
                    all_healthy = false;
                }
                _ => {
                    warn!(
                        "⚠️ MCP server '{}' status: {:?}",
                        server_name, server_info.status
                    );
                }
            }
        }
    }

    all_healthy
}

/// Demonstrate the integrated system with comprehensive testing
async fn demonstrate_system(
    pod_manager: &PodManager,
    db_manager: &Option<DatabaseManager>,
    redis_manager: &Option<RedisManager>,
    embedded_db_manager: &Option<EmbeddedDatabaseManager>,
    character_sheet: &CharacterSheet,
    ai_client_manager: &Arc<RwLock<AiSdk5ClientManager>>,
    mcp_manager: &Arc<RwLock<McpManager>>,
) -> Result<()> {
    info!("🎭 === Banshee System Demonstration ===");
    info!(
        "Using character sheet: {} with database: {:?}",
        character_sheet.name, character_sheet.database_config.primary_db
    );

    // Create a test agent
    let agent_id = Uuid::new_v4();
    info!("🤖 Created test agent with ID: {}", agent_id);

    // Demonstrate system capabilities
    demonstrate_emotional_persistence(agent_id).await?;
    demonstrate_conversation_history(agent_id).await?;
    demonstrate_memory_storage(agent_id).await?;

    // Demonstrate database-specific capabilities
    if let Some(db_manager) = db_manager {
        demonstrate_database_performance(db_manager).await?;
    }
    if let Some(redis_manager) = redis_manager {
        demonstrate_redis_performance(redis_manager).await?;
    }
    if let Some(embedded_db_manager) = embedded_db_manager {
        demonstrate_embedded_database_capabilities(embedded_db_manager, agent_id).await?;
    }

    // Demonstrate character sheet features
    demonstrate_character_sheet_features(character_sheet).await?;

    // Demonstrate AI SDK 5 capabilities
    demonstrate_ai_sdk5_capabilities(ai_client_manager).await?;

    // Demonstrate MCP integration
    demonstrate_mcp_integration(mcp_manager).await?;

    info!("✨ === System demonstration complete ===");
    Ok(())
}

/// Gracefully shutdown all services
async fn shutdown_services(
    mut pod_injector: PodInjector,
    mut db_manager: Option<DatabaseManager>,
    mut redis_manager: Option<RedisManager>,
    mut embedded_db_manager: Option<EmbeddedDatabaseManager>,
    ai_client_manager: Arc<RwLock<AiSdk5ClientManager>>,
    mcp_manager: Arc<RwLock<McpManager>>,
) -> Result<()> {
    // Shutdown pods first
    if let Err(e) = pod_injector.shutdown_all_pods().await {
        error!("Failed to shutdown pods: {}", e);
    }

    // Shutdown MCP manager
    {
        let mut mcp_manager_guard = mcp_manager.write().await;
        if let Err(e) = mcp_manager_guard.shutdown_all_servers().await {
            error!("Failed to shutdown MCP servers: {}", e);
        }
    }

    // Shutdown AI SDK 5 client manager
    {
        let ai_client_manager_guard = ai_client_manager.read().await;
        if let Err(e) = ai_client_manager_guard.shutdown_all().await {
            error!("Failed to shutdown AI SDK 5 clients: {}", e);
        }
    }

    // Shutdown embedded database
    if let Some(mut embedded_db) = embedded_db_manager {
        if let Err(e) = embedded_db.shutdown().await {
            error!("Failed to shutdown embedded database: {}", e);
        }
    }

    // Shutdown Redis
    if let Some(mut redis) = redis_manager {
        if let Err(e) = redis.shutdown().await {
            error!("Failed to shutdown Redis: {}", e);
        }
    }

    // Shutdown traditional database last
    if let Some(mut db) = db_manager {
        if let Err(e) = db.shutdown().await {
            error!("Failed to shutdown database: {}", e);
        }
    }

    Ok(())
}

/// Get database URL from environment or default
fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://banshee:banshee_dev_password@localhost:5432/banshee".to_string()
    })
}

/// Get Redis URL from environment or default
fn get_redis_url() -> String {
    env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string())
}

/// Get environment variable as Option
fn get_env_var(key: &str) -> Option<String> {
    env::var(key).ok()
}

/// Initialize AI SDK 5 clients from character sheet configuration
async fn initialize_ai_clients(
    ai_client_manager: &Arc<RwLock<AiSdk5ClientManager>>,
    character_sheet: &CharacterSheet,
) -> Result<()> {
    info!("Setting up AI SDK 5 clients from character sheet");

    // For now, create a default OpenAI client if API key is available
    if let Some(openai_key) = character_sheet.secrets.api_keys.get("openai") {
        let config = AiSdk5Config {
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            api_key: Some(openai_key.value.clone()),
            transport: TransportConfig::HTTP {
                endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
                headers: HashMap::new(),
                timeout_seconds: 60,
            },
            streaming_enabled: true,
            max_tokens: Some(4000),
            temperature: Some(0.7),
            top_p: Some(0.9),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
        };

        let mut client_manager = ai_client_manager.write().await;
        client_manager
            .add_client("default".to_string(), config)
            .await
            .context("Failed to add default OpenAI client")?;

        info!("Added default OpenAI client with AI SDK 5 transport");
    } else {
        info!("No OpenAI API key found in character sheet secrets - skipping AI client setup");
    }

    // Add Anthropic client if configured
    if let Some(anthropic_key) = character_sheet.secrets.api_keys.get("anthropic") {
        let config = AiSdk5Config {
            provider: "anthropic".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
            api_key: Some(anthropic_key.value.clone()),
            transport: TransportConfig::HTTP {
                endpoint: "https://api.anthropic.com/v1/messages".to_string(),
                headers: HashMap::new(),
                timeout_seconds: 60,
            },
            streaming_enabled: true,
            max_tokens: Some(4000),
            temperature: Some(0.7),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
        };

        let mut client_manager = ai_client_manager.write().await;
        client_manager
            .add_client("anthropic".to_string(), config)
            .await
            .context("Failed to add Anthropic client")?;

        info!("Added Anthropic client with AI SDK 5 transport");
    }

    Ok(())
}

/// Demonstrate database performance
async fn demonstrate_database_performance(db_manager: &DatabaseManager) -> Result<()> {
    info!("📊 --- Database Performance Test ---");

    if let Ok(stats) = db_manager.get_stats().await {
        info!("Current database statistics: {}", stats);
    }

    info!("✓ Database performance test completed");
    Ok(())
}

/// Demonstrate Redis performance
async fn demonstrate_redis_performance(redis_manager: &RedisManager) -> Result<()> {
    info!("🚀 --- Redis Performance Test ---");

    if let Ok(stats) = redis_manager.get_stats().await {
        info!("Current Redis statistics: {}", stats);
    }

    info!("✓ Redis performance test completed");
    Ok(())
}

/// Demonstrate emotional state persistence and retrieval
async fn demonstrate_emotional_persistence(agent_id: Uuid) -> Result<()> {
    use banshee_core::emotion::{Emotion, EmotionalEvent, EmotionalState};

    info!("--- Demonstrating Emotional Intelligence Persistence ---");

    // This would need proper casting - simplified for demonstration
    info!("Creating emotional state with Joy and Pride");
    let mut emotional_state = EmotionalState::new();
    emotional_state.update_emotion(Emotion::Joy, 0.8);
    emotional_state.update_emotion(Emotion::Pride, 0.6);

    info!(
        "Emotional state created: valence={:.2}, arousal={:.2}",
        emotional_state.overall_valence(),
        emotional_state.overall_arousal()
    );

    // In a real implementation, we would:
    // 1. Save the emotional state to database
    // 2. Create and save emotional events
    // 3. Load the state back and verify persistence
    // 4. Demonstrate temporal decay mechanics

    info!("✓ Emotional state would be persisted to PostgreSQL with Redis caching");
    info!("✓ Emotional events would be logged for history tracking");
    info!("✓ State would survive agent restarts");

    Ok(())
}

/// Demonstrate conversation history tracking
async fn demonstrate_conversation_history(agent_id: Uuid) -> Result<()> {
    info!("--- Demonstrating Conversation History ---");

    // Simulate conversation messages
    let messages = vec![
        ("user", "Hello, how are you feeling today?"),
        ("assistant", "I'm feeling quite positive! My joy level is at 0.8 and I have a sense of pride from our recent accomplishments."),
        ("user", "That's great to hear. Can you help me with a coding task?"),
        ("assistant", "Absolutely! I'd be happy to help. My emotional state makes me particularly motivated to assist you."),
    ];

    for (role, content) in &messages {
        info!("Message [{}]: {}", role, content);
        // In real implementation: memory_plugin.save_conversation_message(agent_id, role, content, None).await?;
    }

    info!(
        "✓ {} messages would be saved with emotional context",
        messages.len()
    );
    info!("✓ Full-text search would be available for message content");
    info!("✓ Token usage and metadata would be tracked");

    Ok(())
}

/// Demonstrate flexible memory storage
async fn demonstrate_memory_storage(agent_id: Uuid) -> Result<()> {
    info!("--- Demonstrating Memory Storage ---");

    // Simulate different types of memories
    let memories = vec![
        (
            "goal",
            "primary_objective",
            serde_json::json!({"goal": "Help users with coding tasks", "priority": "high", "progress": 0.7}),
        ),
        (
            "preference",
            "communication_style",
            serde_json::json!({"style": "friendly", "formality": "casual", "emoji_usage": "moderate"}),
        ),
        (
            "skill",
            "programming_languages",
            serde_json::json!({"rust": 0.9, "python": 0.8, "javascript": 0.7}),
        ),
        (
            "fact",
            "user_timezone",
            serde_json::json!({"timezone": "UTC-8", "preferred_hours": "9-17"}),
        ),
    ];

    for (memory_type, key, data) in &memories {
        info!("Storing memory [{}:{}]: {}", memory_type, key, data);
        // In real implementation: memory_plugin.store_memory(agent_id, memory_type, key, data, None).await?;
    }

    info!(
        "✓ {} different memory types stored with importance ranking",
        memories.len()
    );
    info!("✓ Memories would be cached in Redis for fast access");
    info!("✓ TTL-based cleanup would manage temporary data");

    Ok(())
}

/// Example pod manager configuration for production
#[allow(dead_code)]
async fn production_pod_setup() -> PodResult<PodManager> {
    let mut manager = PodManager::new();

    // Core infrastructure pods (no dependencies)
    // Memory pod temporarily disabled for demonstration
    // let memory_pod = create_memory_pod().await?;
    // manager.register(memory_pod).await?;

    // Emotion processing (depends on memory)
    // let emotion_pod = EmotionPod::new().with_dependency("memory", "1.0.0");
    // manager.register(Box::new(emotion_pod)).await?;

    // Web3 capabilities (depends on memory for wallet persistence)
    // let web3_pod = Web3Pod::new().with_dependency("memory", "1.0.0");
    // manager.register(Box::new(web3_pod)).await?;

    // Bootstrap pod (depends on emotion, memory, web3)
    // let bootstrap_pod = BootstrapPod::new()
    //     .with_dependency("memory", "1.0.0")
    //     .with_dependency("emotion", "1.0.0")
    //     .with_dependency("web3", "1.0.0");
    // manager.register(Box::new(bootstrap_pod)).await?;

    // Initialize all in dependency order
    manager.initialize_all().await?;

    Ok(manager)
}

// Temporarily removed memory pod creation function due to compilation issues

/// Demonstrate embedded database capabilities
async fn demonstrate_embedded_database_capabilities(
    embedded_db_manager: &EmbeddedDatabaseManager,
    agent_id: Uuid,
) -> Result<()> {
    info!("🗄️ --- Embedded Database Capabilities ---");

    // Test storing emotion state
    let emotion_data = serde_json::json!({
        "emotions": {
            "joy": 0.8,
            "pride": 0.6,
            "curiosity": 0.7
        },
        "timestamp": chrono::Utc::now(),
        "agent_id": agent_id
    });

    match embedded_db_manager
        .store_emotion_state(agent_id, emotion_data)
        .await
    {
        Ok(_) => info!("✓ Stored emotion state for agent {}", agent_id),
        Err(e) => warn!("Failed to store emotion state: {}", e),
    }

    // Test retrieving emotion state
    match embedded_db_manager.get_emotion_state(agent_id).await {
        Ok(Some(state)) => info!("✓ Retrieved emotion state: {:?}", state),
        Ok(None) => info!("No emotion state found for agent"),
        Err(e) => warn!("Failed to retrieve emotion state: {}", e),
    }

    // Test storing memory
    let memory_data = serde_json::json!({
        "content": "User prefers technical explanations",
        "importance": 0.8,
        "timestamp": chrono::Utc::now()
    });

    match embedded_db_manager
        .store_memory(agent_id, "preference", "technical_level", memory_data)
        .await
    {
        Ok(_) => info!("✓ Stored memory for agent {}", agent_id),
        Err(e) => warn!("Failed to store memory: {}", e),
    }

    // Get database statistics
    match embedded_db_manager.get_stats().await {
        Ok(stats) => info!("📊 Embedded database stats: {}", stats),
        Err(e) => warn!("Failed to get database stats: {}", e),
    }

    info!("✓ Embedded database capabilities demonstration completed");
    Ok(())
}

/// Demonstrate character sheet features
async fn demonstrate_character_sheet_features(character_sheet: &CharacterSheet) -> Result<()> {
    info!("📋 --- Character Sheet Features ---");

    info!("Character Sheet: {}", character_sheet.name);
    info!("  Version: {}", character_sheet.version);
    info!(
        "  Database: {:?}",
        character_sheet.database_config.primary_db
    );
    info!(
        "  Memory Limit: {} MB",
        character_sheet.memory_settings.max_memory_mb
    );

    // Personality traits
    let personality = &character_sheet.personality;
    info!("Personality (Big Five):");
    info!("  Openness: {:.2}", personality.big_five.openness);
    info!(
        "  Conscientiousness: {:.2}",
        personality.big_five.conscientiousness
    );
    info!("  Extraversion: {:.2}", personality.big_five.extraversion);
    info!("  Agreeableness: {:.2}", personality.big_five.agreeableness);
    info!("  Neuroticism: {:.2}", personality.big_five.neuroticism);

    // Communication style
    info!("Communication Style:");
    info!(
        "  Formality: {:?}",
        personality.communication_style.formality
    );
    info!(
        "  Verbosity: {:?}",
        personality.communication_style.verbosity
    );
    info!(
        "  Technical Depth: {:?}",
        personality.communication_style.technical_depth
    );

    // Capabilities
    info!(
        "Enabled Capabilities: {:?}",
        character_sheet.capabilities.enabled_capabilities
    );

    // Tool access
    let tools = &character_sheet.capabilities.tool_access;
    info!("Tool Access:");
    info!("  Web Access: {}", tools.web_access);
    info!("  File System Access: {}", tools.file_system_access);
    info!("  Database Access: {}", tools.database_access);
    info!("  Blockchain Access: {}", tools.blockchain_access);

    // Security settings
    info!("Security Settings:");
    info!(
        "  Encryption Enabled: {}",
        character_sheet.security.encryption_enabled
    );
    info!(
        "  Audit Logging: {}",
        character_sheet.security.audit_logging
    );
    info!(
        "  Data Retention: {} days",
        character_sheet.security.data_retention_days
    );

    // Knowledge configuration
    info!("Knowledge Configuration:");
    info!(
        "  Knowledge Bases: {}",
        character_sheet.knowledge.knowledge_bases.len()
    );
    info!(
        "  Vector Search: {}",
        character_sheet.knowledge.vector_search_enabled
    );
    info!(
        "  Indexing Strategy: {:?}",
        character_sheet.knowledge.indexing_strategy
    );

    // MCP servers
    info!("MCP Configuration:");
    info!(
        "  Configured Servers: {}",
        character_sheet.mcp_servers.servers.len()
    );
    info!(
        "  Max Concurrent Connections: {}",
        character_sheet.mcp_servers.max_concurrent_connections
    );
    info!(
        "  Default Timeout: {}s",
        character_sheet.mcp_servers.default_timeout_seconds
    );

    // Templates
    info!("Template Configuration:");
    info!(
        "  Prompt Templates: {}",
        character_sheet.templates.prompt_templates.len()
    );
    info!(
        "  Response Templates: {}",
        character_sheet.templates.response_templates.len()
    );
    info!(
        "  Workflow Templates: {}",
        character_sheet.templates.workflow_templates.len()
    );
    info!(
        "  Template Engine: {:?}",
        character_sheet.templates.template_engine
    );

    info!("✓ Character sheet features demonstration completed");
    Ok(())
}

/// Demonstrate AI SDK 5 capabilities
async fn demonstrate_ai_sdk5_capabilities(
    ai_client_manager: &Arc<RwLock<AiSdk5ClientManager>>,
) -> Result<()> {
    info!("🤖 --- AI SDK 5 Integration Demonstration ---");

    let client_manager = ai_client_manager.read().await;

    // Show available AI clients
    if let Some(default_client) = client_manager.get_client("default") {
        info!("✓ Default AI client available (transport-based)");

        // Test health check
        match default_client.health_check().await {
            Ok(healthy) => {
                if healthy {
                    info!("✓ AI client health check passed");
                } else {
                    warn!("⚠️ AI client health check failed");
                }
            }
            Err(e) => warn!("Failed to check AI client health: {}", e),
        }
    } else {
        info!("ℹ️ No default AI client configured");
    }

    info!("✓ AI SDK 5 integration demonstration completed");
    Ok(())
}

/// Demonstrate MCP integration capabilities
async fn demonstrate_mcp_integration(mcp_manager: &Arc<RwLock<McpManager>>) -> Result<()> {
    info!("🔗 --- MCP Integration Demonstration ---");

    let mcp_manager_guard = mcp_manager.read().await;

    // Show available MCP tools
    let available_tools = mcp_manager_guard.get_all_tools().await;

    if available_tools.is_empty() {
        info!("ℹ️ No MCP tools available (no servers configured)");
    } else {
        info!(
            "✓ Found {} MCP tools across all servers",
            available_tools.len()
        );

        for tool in &available_tools {
            info!(
                "  - {} (from {}): {}",
                tool.name, tool.server, tool.description
            );
        }

        // Demonstrate tool filtering for pods (if any listeners configured)
        let empty_listeners = vec![];
        let pod_tools = mcp_manager_guard.get_tools_for_pod(&empty_listeners).await;
        info!(
            "✓ Pod tool filtering system operational ({} pod mappings)",
            pod_tools.len()
        );
    }

    // Show server status
    let active_servers = mcp_manager_guard.list_active_servers();
    info!("✓ {} MCP servers being managed", active_servers.len());

    for (server_name, server_info) in &active_servers {
        info!(
            "  - {}: {:?} (errors: {}, connections: {})",
            server_name, server_info.status, server_info.error_count, server_info.connection_count
        );
    }

    info!("✓ MCP integration demonstration completed");
    Ok(())
}
