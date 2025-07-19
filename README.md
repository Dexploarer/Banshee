# Banshee - Emotional AI Agents Framework

A sophisticated emotional intelligence system for AI agents built in Rust, implementing the OCC (Ortony, Clore, and Collins) model of emotions with support for Model Context Protocol (MCP) and AI SDK 5 beta integration.

## 🧠 Features

### Core Emotional Intelligence
- **22 Discrete Emotions**: Complete implementation of the OCC model including Joy, Distress, Fear, Anger, Pride, Shame, and more
- **Cognitive Appraisal**: Events are appraised based on agent goals, personality traits, and context
- **Temporal Decay**: Realistic emotion evolution with configurable decay rates
- **Emotional Escalation**: Tool selection based on frustration levels and emotional state

### Personality & Character System
- **Big Five Traits**: Openness, Conscientiousness, Extraversion, Agreeableness, Neuroticism
- **PAD Space Mapping**: Pleasure-Arousal-Dominance emotional dimensions
- **Character Profiles**: Pre-built personalities for coding assistants, customer service, research agents

### Modern AI Integration
- **MCP 2025-06-18 Spec**: Latest Model Context Protocol with OAuth 2.1 authentication
- **AI SDK 5 Beta**: Unified content arrays, streaming responses, tool integration
- **Actor Model**: Async message-driven architecture using Actix

## 🚀 Quick Start

```rust
use emotion_engine::{OCCEmotionalState, AppraisalEngine, EmotionalEvent, PersonalityModifiers};

// Create an emotional state
let mut emotional_state = OCCEmotionalState::new();

// Create an appraisal engine
let mut appraisal_engine = AppraisalEngine::new(
    vec!["coding".to_string(), "helping_users".to_string()],
    PersonalityModifiers::default(),
);

// Process an emotional event
let event = EmotionalEvent::TaskCompleted {
    difficulty: 0.7,
    success: true,
    time_taken: 120.0,
    expected_time: 100.0,
    was_retry: false,
};

let emotional_responses = appraisal_engine.appraise_event(&event);

// Apply emotions to state
for (emotion, intensity) in emotional_responses {
    emotional_state.update_emotion(emotion, intensity);
}

// Check emotional state
println!("Current state: {}", emotional_state.summary());
```

## 📁 Project Structure

```
banshee/
├── crates/
│   ├── emotion_engine/         # Core OCC model implementation
│   ├── character_sheet/        # Personality traits and agent definitions
│   ├── mcp_manager/           # Model Context Protocol integration
│   ├── ai_sdk_client/         # AI SDK 5 beta client
│   ├── agent_runtime/         # Actor-based agent system
│   ├── persistence/           # Database and caching layer
│   ├── config/               # Configuration management
│   └── utils/                # Common utilities
├── scripts/                  # Development and deployment scripts
├── docs/                    # Documentation
└── examples/               # Example implementations
```

## 🧪 Key Components

### Emotion Engine (`emotion_engine`)

The core emotional intelligence system featuring:

#### OCC Emotions
```rust
pub enum OCCEmotion {
    // Event-based emotions
    Joy, Distress, Hope, Fear, Satisfaction, Disappointment, Relief, FearConfirmed,
    
    // Attribution emotions  
    Pride, Shame, Admiration, Reproach, Gratification, Remorse, Gratitude, Anger,
    
    // Attraction emotions
    Love, Hate,
    
    // Well-being emotions
    HappyFor, Resentment, Gloating, Pity,
}
```

#### Emotional Events
- **Task Completion**: Success/failure with difficulty and timing factors
- **Tool Failures**: Escalating frustration based on criticality and attempts
- **User Feedback**: Sentiment analysis with praise/criticism detection
- **Goal Progress**: Milestone achievements and setbacks
- **Peer Interactions**: Social emotions from collaboration and recognition

#### Personality Modifiers
```rust
pub struct PersonalityModifiers {
    pub optimism: f32,          // Affects joy/distress intensity
    pub volatility: f32,        // Affects emotional change rate  
    pub persistence: f32,       // Affects frustration buildup
    pub self_confidence: f32,   // Affects pride/shame responses
    pub social_sensitivity: f32, // Affects social emotion responses
}
```

### Character Sheet (`character_sheet`)

Agent personality and capability system:

```rust
pub struct CharacterSheet {
    pub personality: BigFiveTraits,
    pub capabilities: Vec<AgentCapability>,
    pub emotional_profile: EmotionalProfile,
    pub tool_preferences: ToolPreferences,
}
```

**Pre-built Personalities:**
- **Coding Assistant**: High conscientiousness, openness; stable under pressure
- **Customer Service**: High agreeableness, extraversion; extremely helpful
- **Research Assistant**: Very high openness; methodical and curious

## 🔧 Development Setup

### Prerequisites
- Rust 1.70+ (2021 edition)
- PostgreSQL 15+ (for persistence)
- Redis 7+ (for caching)

### Installation
```bash
# Clone the repository
git clone <repository-url>
cd banshee

# Install development tools
cargo install cargo-watch cargo-audit

# Run tests
cargo test

# Run with live reloading
cargo watch -x "check" -x "test" -x "run"
```

### Development Commands
```bash
# Format code
cargo fmt

# Lint code
cargo clippy --all-targets --all-features

# Run pre-commit checks
./scripts/pre-commit.sh

# Test specific crate
cargo test -p banshee-emotion-engine
```

## 🎯 Emotional Intelligence in Action

### Tool Selection Based on Emotions

The framework dynamically selects tools based on the agent's emotional state:

1. **Calm State**: Uses basic tools efficiently
2. **Frustrated State**: Escalates to more powerful tools (e.g., advanced search APIs)
3. **Confident State**: Takes on more challenging tasks
4. **Anxious State**: Seeks validation and uses safer approaches

### Example: Programming Assistant

```rust
// Agent starts with basic emotions
agent.update_emotion(OCCEmotion::Hope, 0.6); // Optimistic about helping

// User asks a difficult question - tool call fails
let failure_event = EmotionalEvent::ToolCallFailed {
    tool_name: "basic_search".to_string(),
    attempts: 2,
    error_severity: 0.8,
    is_critical: true,
    error_message: Some("API timeout".to_string()),
};

// Agent becomes frustrated
agent.process_event(failure_event);

// Now automatically escalates to more powerful tools
// frustration_level > escalation_threshold triggers advanced search
```

## 🧮 Testing & Quality

### Comprehensive Test Suite
- **Unit Tests**: 17 passing tests for emotion engine
- **Integration Tests**: Cross-crate functionality testing
- **Property-Based Tests**: Emotional state transitions
- **Performance Benchmarks**: Emotion update and decay timing

### Quality Metrics
- ✅ All clippy lints pass
- ✅ Code formatting with rustfmt
- ✅ No unsafe code
- ✅ Comprehensive error handling with `anyhow` and `thiserror`

## 🔬 Technical Details

### Architecture Principles
- **Actor Model**: Each agent runs in its own actor with message passing
- **Type Safety**: Leverages Rust's type system for emotional state guarantees
- **Performance**: Optimized for real-time emotional updates (<100ms)
- **Modularity**: Clean separation between emotion engine, personality, and behavior

### Emotional Decay Mathematics
Emotions follow exponential decay: `intensity *= (1 - decay_rate)^delta_seconds`

Decay rates are based on psychological research:
- **Fast-decaying**: Fear (0.12/s), Anger (0.15/s) - high arousal emotions
- **Medium-decaying**: Joy (0.05/s), Distress (0.08/s) - moderate persistence  
- **Slow-decaying**: Love (0.01/s), Pride (0.03/s) - deep, persistent emotions

### MCP Integration
Following the 2025-06-18 specification:
- **OAuth 2.1 Authentication**: Resource server classification
- **Tool Output Schemas**: Predictable response formats
- **Elicitation Support**: Server-driven information requests
- **Streamable HTTP**: Replacing deprecated SSE transport

## 🚦 Roadmap

### Phase 1: Foundation (Completed)
- ✅ Core OCC emotion model
- ✅ Personality trait system
- ✅ Basic MCP integration
- ✅ Development tooling

### Phase 2: Advanced Features (Next)
- 🔄 Complete MCP server implementations
- 🔄 AI SDK 5 client with streaming
- 🔄 Multi-agent coordination
- 🔄 Emotional learning and adaptation

### Phase 3: Production Ready
- 📋 Performance optimization
- 📋 Monitoring and observability
- 📋 Production deployment tools
- 📋 Advanced security features

## 🤝 Contributing

We welcome contributions! Please see:
- Code of Conduct
- Contributing Guidelines  
- Development Setup Guide

## 📄 License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- **OCC Model**: Ortony, Clore, and Collins for the foundational emotion theory
- **Rust Community**: For exceptional tooling and libraries
- **AI SDK Team**: For modern AI integration patterns
- **MCP Contributors**: For the standardized AI agent communication protocol

---

**Built with ❤️ and 🦀 for the future of emotional AI**