# Banshee Memory Plugin

A comprehensive memory and persistence layer for the Banshee emotional agents framework. Provides real-time emotional state persistence, conversation history, and flexible memory storage using PostgreSQL and Redis.

## Features

### 🧠 Emotional Intelligence Persistence
- **Real-time emotional state storage** with PostgreSQL persistence and Redis caching
- **OCC model support** for all 22 discrete emotions with proper intensity tracking
- **Temporal decay persistence** - emotional states persist across agent restarts
- **Event history tracking** - complete audit trail of emotional events and their impacts
- **Emotional trend analysis** - track valence and arousal changes over time

### 💬 Conversation Management  
- **Full conversation history** with role-based message tracking
- **Emotional context integration** - link messages to emotional states
- **Full-text search** capabilities for conversation content
- **Token usage tracking** for cost monitoring
- **Metadata support** for rich message annotations

### 🗃️ Flexible Memory Storage
- **Typed memory system** - goals, preferences, facts, skills, etc.
- **TTL support** for temporary memories with automatic cleanup
- **Importance-based prioritization** - smart memory ranking
- **Access pattern tracking** - monitor memory usage
- **Redis caching** for high-performance retrieval

### 🔧 Production Features
- **Automatic cleanup** - configurable retention policies
- **Database migrations** - schema versioning and updates  
- **Connection pooling** - optimized database performance
- **Background tasks** - non-blocking maintenance operations
- **Comprehensive indexing** - optimized queries

## Architecture

### Database Design

**PostgreSQL Tables:**
- `emotional_states` - Current emotional state snapshots
- `emotional_events` - Historical emotional event log
- `conversation_messages` - Complete conversation history  
- `memory_records` - Flexible key-value memory storage
- `agent_sessions` - Session tracking and analytics

**Redis Usage:**
- Emotional state caching (1-hour TTL)
- Recent conversation messages (50 message buffer) 
- Memory data caching with configurable TTL
- Real-time event streams for live processing

### Plugin Architecture

```rust
use banshee_plugin_memory::{MemoryPlugin, MemoryResult};
use banshee_core::emotion::{EmotionalState, Emotion};
use uuid::Uuid;

#[tokio::main]
async fn main() -> MemoryResult<()> {
    // Initialize the memory plugin
    let memory = MemoryPlugin::new(
        "postgresql://user:pass@localhost/banshee",
        "redis://localhost:6379"
    ).await?;

    let agent_id = Uuid::new_v4();
    
    // Save emotional state
    let mut state = EmotionalState::new();
    state.update_emotion(Emotion::Joy, 0.8);
    memory.save_emotional_state(agent_id, &state).await?;
    
    // Load emotional state (from cache or database)
    if let Some(loaded_state) = memory.load_emotional_state(agent_id).await? {
        println!("Restored state: {}", loaded_state.summary());
    }
    
    // Store conversation message
    let message_id = memory.save_conversation_message(
        agent_id,
        "user", 
        "Hello, how are you feeling?",
        None
    ).await?;
    
    // Store custom memory
    memory.store_memory(
        agent_id,
        "preference",
        "communication_style", 
        serde_json::json!({"style": "friendly", "formality": "casual"}),
        None // No TTL
    ).await?;
    
    Ok(())
}
```

## Configuration

### Environment Variables

```bash
# PostgreSQL connection
DATABASE_URL=postgresql://username:password@localhost:5432/banshee

# Redis connection  
REDIS_URL=redis://localhost:6379

# Optional: Redis password
REDIS_PASSWORD=your_redis_password

# Optional: Connection pool settings
DB_MAX_CONNECTIONS=20
DB_MIN_CONNECTIONS=5
```

### Database Setup

1. **Run migrations:**
```bash
# Using sqlx-cli
sqlx migrate run --database-url postgresql://user:pass@localhost/banshee

# Or manually execute the migration
psql -d banshee -f packages/plugin-memory/migrations/001_initial_schema.sql
```

2. **Verify installation:**
```sql
-- Check tables exist
\dt

-- View emotional states structure
\d emotional_states

-- Check indexes
\di
```

### Redis Setup

```bash
# Start Redis server
redis-server

# Or with Docker
docker run -d --name banshee-redis -p 6379:6379 redis:7-alpine

# Configure persistence (optional)
redis-cli CONFIG SET save "900 1 300 10 60 10000"
```

## API Reference

### EmotionStore

```rust
// Save current emotional state
async fn save_state(&self, agent_id: Uuid, state: &EmotionalState) -> MemoryResult<()>

// Load emotional state (cache-first)
async fn load_state(&self, agent_id: Uuid) -> MemoryResult<Option<EmotionalState>>

// Record emotional event
async fn save_event(
    &self, 
    agent_id: Uuid, 
    event: &EmotionalEvent,
    resulting_emotions: &HashMap<Emotion, EmotionalIntensity>
) -> MemoryResult<()>

// Get emotional history
async fn get_event_history(&self, agent_id: Uuid, limit: Option<i64>) -> MemoryResult<Vec<EmotionalEventRecord>>

// Analyze emotional trends
async fn get_emotional_trends(&self, agent_id: Uuid, hours: i32) -> MemoryResult<Vec<(DateTime<Utc>, f32, f32)>>
```

### ConversationStore

```rust
// Save conversation message
async fn save_message(
    &self,
    agent_id: Uuid,
    role: &str,
    content: &str, 
    metadata: Option<Value>
) -> MemoryResult<Uuid>

// Get conversation history
async fn get_history(&self, agent_id: Uuid, limit: Option<i64>) -> MemoryResult<Vec<ConversationMessage>>

// Search messages by content
async fn search_messages(&self, agent_id: Uuid, query: &str, limit: Option<i64>) -> MemoryResult<Vec<ConversationMessage>>

// Get conversation statistics
async fn get_conversation_stats(&self, agent_id: Uuid) -> MemoryResult<ConversationStats>
```

### MemoryManager

```rust
// Store custom memory
async fn store_memory(
    &self,
    agent_id: Uuid,
    memory_type: &str,
    key: &str,
    data: Value,
    ttl_seconds: Option<i64>
) -> MemoryResult<()>

// Retrieve memory (cache-first)
async fn retrieve_memory(&self, agent_id: Uuid, memory_type: &str, key: &str) -> MemoryResult<Option<Value>>

// Get memories by type
async fn get_memories_by_type(&self, agent_id: Uuid, memory_type: &str, limit: Option<i64>, offset: Option<i64>) -> MemoryResult<Vec<MemoryRecord>>

// Delete memory
async fn delete_memory(&self, agent_id: Uuid, memory_type: &str, key: &str) -> MemoryResult<bool>
```

## Memory Types

The plugin supports typed memory storage for different kinds of agent data:

- **`goal`** - Agent objectives and targets (high importance)
- **`preference`** - User preferences and settings
- **`personality`** - Personality traits and modifiers  
- **`skill`** - Learned capabilities and knowledge
- **`fact`** - Factual information and knowledge
- **`experience`** - Past experiences and learnings
- **`relationship`** - Social connections and context
- **`context`** - Situational and environmental context
- **`temp`** - Temporary data with TTL
- **`cache`** - Cached computations (low importance)

## Performance Optimization

### Indexing Strategy
- **Agent-based partitioning** - all queries are agent-scoped
- **Time-based indexing** - optimized for recent data access
- **Compound indexes** - multi-column queries optimized
- **Full-text search** - GIN indexes for content search

### Caching Strategy  
- **Emotional states** - 1-hour Redis cache
- **Recent conversations** - 50-message rolling buffer
- **Memory data** - configurable TTL per memory type
- **Event streams** - Redis streams for real-time processing

### Cleanup Policies
- **Emotional events** - 30-day retention by default
- **Conversation messages** - 90-day retention by default  
- **Expired memories** - automatic TTL-based cleanup
- **Background maintenance** - hourly cleanup tasks

## Testing

```bash
# Run all tests
cargo test -p banshee-plugin-memory

# Run with database (requires test DB)
DATABASE_URL=postgresql://test:test@localhost/banshee_test cargo test

# Run integration tests
cargo test --features integration-tests
```

## Monitoring

### Metrics to Track
- Emotional state save/load latency
- Cache hit/miss ratios  
- Database connection pool usage
- Memory cleanup effectiveness
- Conversation message growth rate

### Health Checks
```rust
// Check database connectivity
let health = memory.health_check().await?;
assert!(health.postgres_connected);
assert!(health.redis_connected);
```

## Migration Guide

### From Legacy Emotion Engine

```rust
// Old way (in-memory only)
let mut state = OCCEmotionalState::new();
state.update_emotion(OCCEmotion::Joy, 0.8);
// Lost on restart

// New way (persistent)  
let mut state = EmotionalState::new();
state.update_emotion(Emotion::Joy, 0.8);
memory.save_emotional_state(agent_id, &state).await?;
// Persists across restarts
```

### Schema Evolution
- Migration files in `migrations/` directory
- Backward-compatible changes preferred
- Data migration scripts for breaking changes

## Contributing

1. **Database changes** - add migration files
2. **New memory types** - update type system  
3. **Performance improvements** - benchmark before/after
4. **Tests** - integration tests for database operations

## License

Same as the main Banshee project.