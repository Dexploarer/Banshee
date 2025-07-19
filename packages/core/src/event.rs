use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{AgentId, Context, Message, Result};

/// Event identifier
pub type EventId = Uuid;

/// Core event structure for the framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique event identifier
    pub id: EventId,

    /// Event type
    pub event_type: EventType,

    /// Source agent ID
    pub source: AgentId,

    /// Target agent ID (if applicable)
    pub target: Option<AgentId>,

    /// Event payload
    pub payload: serde_json::Value,

    /// Event metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// When event occurred
    pub timestamp: DateTime<Utc>,

    /// Event priority
    pub priority: EventPriority,
}

/// Types of events in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// Message received/sent events
    MessageReceived,
    MessageSent,

    /// Emotional events
    EmotionalStateChanged,
    EmotionalEvent,

    /// Agent lifecycle events
    AgentCreated,
    AgentStarted,
    AgentStopped,
    AgentShutdown,

    /// Memory events
    MemoryStored,
    MemoryRetrieved,
    MemoryUpdated,

    /// Action events
    ActionExecuted,
    ActionFailed,

    /// Tool events
    ToolCallStarted,
    ToolCallCompleted,
    ToolCallFailed,

    /// Provider events
    ProviderExecuted,

    /// Evaluator events
    EvaluationCompleted,
    AlertTriggered,

    /// Plugin events
    PluginLoaded,
    PluginUnloaded,
    PluginError,

    /// System events
    SystemStartup,
    SystemShutdown,
    HealthCheck,

    /// Custom events
    Custom(String),
}

/// Event priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Event handler trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Get handler name
    fn name(&self) -> &str;

    /// Check if handler can process this event type
    fn can_handle(&self, event_type: &EventType) -> bool;

    /// Handle the event
    async fn handle(&self, event: &Event, context: &Context) -> Result<()>;

    /// Initialize the handler
    async fn initialize(&mut self) -> Result<()>;

    /// Shutdown the handler
    async fn shutdown(&mut self) -> Result<()>;
}

/// Event bus for managing event distribution
pub struct EventBus {
    handlers: HashMap<EventType, Vec<Box<dyn EventHandler>>>,
    global_handlers: Vec<Box<dyn EventHandler>>,
    event_history: Vec<Event>,
    max_history: usize,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            global_handlers: Vec::new(),
            event_history: Vec::new(),
            max_history: 1000,
        }
    }

    /// Register an event handler for specific event types
    pub async fn register_handler(
        &mut self,
        event_types: Vec<EventType>,
        mut handler: Box<dyn EventHandler>,
    ) -> Result<()> {
        handler.initialize().await?;

        // Note: In a real implementation, we'd use Arc<dyn EventHandler> for sharing
        // For now, we'll store only one handler per event type
        for event_type in event_types {
            self.handlers.entry(event_type).or_insert_with(Vec::new);
            // TODO: Store handlers properly with Arc for sharing
        }

        Ok(())
    }

    /// Register a global handler that receives all events
    pub async fn register_global_handler(
        &mut self,
        mut handler: Box<dyn EventHandler>,
    ) -> Result<()> {
        handler.initialize().await?;
        self.global_handlers.push(handler);
        Ok(())
    }

    /// Publish an event to all relevant handlers
    pub async fn publish(&mut self, event: Event, context: &Context) -> Result<()> {
        // Add to history
        self.event_history.push(event.clone());
        if self.event_history.len() > self.max_history {
            self.event_history.remove(0);
        }

        // Send to specific handlers
        if let Some(handlers) = self.handlers.get(&event.event_type) {
            for handler in handlers {
                if let Err(e) = handler.handle(&event, context).await {
                    tracing::error!("Handler {} failed to process event: {}", handler.name(), e);
                }
            }
        }

        // Send to global handlers
        for handler in &self.global_handlers {
            if let Err(e) = handler.handle(&event, context).await {
                tracing::error!(
                    "Global handler {} failed to process event: {}",
                    handler.name(),
                    e
                );
            }
        }

        Ok(())
    }

    /// Get recent event history
    pub fn get_history(&self, limit: usize) -> &[Event] {
        let start = self.event_history.len().saturating_sub(limit);
        &self.event_history[start..]
    }

    /// Get events by type
    pub fn get_events_by_type(&self, event_type: &EventType) -> Vec<&Event> {
        self.event_history
            .iter()
            .filter(|event| event.event_type == *event_type)
            .collect()
    }

    /// Get events by agent
    pub fn get_events_by_agent(&self, agent_id: &AgentId) -> Vec<&Event> {
        self.event_history
            .iter()
            .filter(|event| event.source == *agent_id || event.target == Some(*agent_id))
            .collect()
    }
}

/// Event builder for easy construction
pub struct EventBuilder {
    event_type: EventType,
    source: AgentId,
    target: Option<AgentId>,
    payload: serde_json::Value,
    metadata: HashMap<String, serde_json::Value>,
    priority: EventPriority,
}

impl EventBuilder {
    pub fn new(event_type: EventType, source: AgentId) -> Self {
        Self {
            event_type,
            source,
            target: None,
            payload: serde_json::Value::Null,
            metadata: HashMap::new(),
            priority: EventPriority::Normal,
        }
    }

    pub fn target(mut self, target: AgentId) -> Self {
        self.target = Some(target);
        self
    }

    pub fn payload<T: Serialize>(mut self, payload: T) -> Self {
        self.payload = serde_json::to_value(payload).unwrap_or(serde_json::Value::Null);
        self
    }

    pub fn metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn build(self) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: self.event_type,
            source: self.source,
            target: self.target,
            payload: self.payload,
            metadata: self.metadata,
            timestamp: Utc::now(),
            priority: self.priority,
        }
    }
}

/// Specific event payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEventPayload {
    pub message: Message,
    pub conversation_id: String,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalEventPayload {
    pub emotion: String,
    pub intensity: f32,
    pub trigger: String,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEventPayload {
    pub action_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub duration_ms: u64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEventPayload {
    pub memory_id: crate::memory::MemoryId,
    pub memory_type: String,
    pub content_preview: String,
    pub importance: f32,
}

/// Built-in event handlers
pub struct LoggingEventHandler {
    name: String,
}

impl LoggingEventHandler {
    pub fn new() -> Self {
        Self {
            name: "logging_handler".to_string(),
        }
    }
}

#[async_trait]
impl EventHandler for LoggingEventHandler {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, _event_type: &EventType) -> bool {
        true // Log all events
    }

    async fn handle(&self, event: &Event, _context: &Context) -> Result<()> {
        tracing::info!(
            event_id = %event.id,
            event_type = ?event.event_type,
            source = %event.source,
            priority = ?event.priority,
            "Event occurred"
        );
        Ok(())
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Logging event handler initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Logging event handler shutdown");
        Ok(())
    }
}

/// Metrics event handler for collecting statistics
pub struct MetricsEventHandler {
    name: String,
    event_counts: HashMap<EventType, u64>,
}

impl MetricsEventHandler {
    pub fn new() -> Self {
        Self {
            name: "metrics_handler".to_string(),
            event_counts: HashMap::new(),
        }
    }

    pub fn get_event_count(&self, event_type: &EventType) -> u64 {
        self.event_counts.get(event_type).copied().unwrap_or(0)
    }

    pub fn get_all_counts(&self) -> &HashMap<EventType, u64> {
        &self.event_counts
    }
}

#[async_trait]
impl EventHandler for MetricsEventHandler {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, _event_type: &EventType) -> bool {
        true
    }

    async fn handle(&self, event: &Event, _context: &Context) -> Result<()> {
        // Note: This would need thread-safe access in a real implementation
        // For now, this is a simplified version
        tracing::debug!("Metrics handler processing event: {:?}", event.event_type);
        Ok(())
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Metrics event handler initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Metrics event handler shutdown");
        Ok(())
    }
}

/// Macro to create simple events
#[macro_export]
macro_rules! create_event {
    ($event_type:expr, $source:expr) => {
        EventBuilder::new($event_type, $source).build()
    };

    ($event_type:expr, $source:expr, $payload:expr) => {
        EventBuilder::new($event_type, $source)
            .payload($payload)
            .build()
    };

    ($event_type:expr, $source:expr, $target:expr, $payload:expr) => {
        EventBuilder::new($event_type, $source)
            .target($target)
            .payload($payload)
            .build()
    };
}
