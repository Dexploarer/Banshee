use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for messages
pub type MessageId = Uuid;

/// Message roles in conversations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    /// System messages that set context
    System,
    /// User messages from humans
    User,
    /// Assistant messages from AI agents
    Assistant,
    /// Tool call results
    Tool,
}

/// Content types for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    /// Plain text content
    Text { text: String },

    /// Image content (base64 encoded)
    Image {
        image: String,
        mime_type: Option<String>,
        description: Option<String>,
    },

    /// Tool call request
    ToolCall {
        tool_name: String,
        arguments: serde_json::Value,
        call_id: String,
    },

    /// Tool call result
    ToolResult {
        call_id: String,
        result: serde_json::Value,
        is_error: bool,
    },

    /// Emotional context
    Emotion {
        emotions: std::collections::HashMap<String, f32>,
        context: String,
    },

    /// Memory reference
    Memory {
        memory_id: String,
        relevance_score: f32,
        content: String,
    },
}

/// Core message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier
    pub id: MessageId,

    /// Message role
    pub role: MessageRole,

    /// Message content (can be multiple parts)
    pub content: Vec<MessageContent>,

    /// Optional sender name
    pub name: Option<String>,

    /// Timestamp when message was created
    pub timestamp: DateTime<Utc>,

    /// Optional metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl Message {
    /// Create a new text message
    pub fn new_text(role: MessageRole, text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role,
            content: vec![MessageContent::Text { text: text.into() }],
            name: None,
            timestamp: Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a new system message
    pub fn system(text: impl Into<String>) -> Self {
        Self::new_text(MessageRole::System, text)
    }

    /// Create a new user message
    pub fn user(text: impl Into<String>) -> Self {
        Self::new_text(MessageRole::User, text)
    }

    /// Create a new assistant message
    pub fn assistant(text: impl Into<String>) -> Self {
        Self::new_text(MessageRole::Assistant, text)
    }

    /// Add content to the message
    pub fn add_content(&mut self, content: MessageContent) {
        self.content.push(content);
    }

    /// Get all text content from the message
    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .filter_map(|content| {
                if let MessageContent::Text { text } = content {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Check if message contains emotional content
    pub fn has_emotion(&self) -> bool {
        self.content
            .iter()
            .any(|content| matches!(content, MessageContent::Emotion { .. }))
    }

    /// Get emotional content from message
    pub fn emotions(&self) -> Option<&std::collections::HashMap<String, f32>> {
        self.content.iter().find_map(|content| {
            if let MessageContent::Emotion { emotions, .. } = content {
                Some(emotions)
            } else {
                None
            }
        })
    }

    /// Add metadata to the message
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Set the sender name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Conversation context containing multiple messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique conversation identifier
    pub id: Uuid,

    /// Messages in chronological order
    pub messages: Vec<Message>,

    /// Conversation metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,

    /// When conversation was created
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Conversation {
    /// Create a new conversation
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            messages: Vec::new(),
            metadata: std::collections::HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    /// Get the latest message
    pub fn latest_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    /// Get messages from a specific role
    pub fn messages_by_role(&self, role: MessageRole) -> impl Iterator<Item = &Message> {
        self.messages.iter().filter(move |msg| msg.role == role)
    }

    /// Get the conversation context as a string
    pub fn context_summary(&self, max_messages: usize) -> String {
        self.messages
            .iter()
            .rev()
            .take(max_messages)
            .rev()
            .map(|msg| format!("{:?}: {}", msg.role, msg.text_content()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Self::new()
    }
}
