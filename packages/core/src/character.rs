use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Character sheet defining an agent's personality and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSheet {
    /// Unique character identifier
    pub id: Uuid,

    /// Character name
    pub name: String,

    /// Character description
    pub description: String,

    /// Personality traits
    pub personality: Personality,

    /// Available capabilities
    pub capabilities: Vec<Capability>,

    /// Knowledge domains
    pub knowledge_domains: Vec<String>,

    /// Communication style
    pub communication_style: CommunicationStyle,

    /// Behavioral patterns
    pub behavioral_patterns: BehavioralPatterns,

    /// Character metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Personality traits based on various models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    /// Big Five personality traits
    pub big_five: BigFiveTraits,

    /// Emotional tendencies
    pub emotional_tendencies: EmotionalTendencies,

    /// Cognitive style
    pub cognitive_style: CognitiveStyle,

    /// Social orientation
    pub social_orientation: SocialOrientation,
}

/// Big Five personality model (OCEAN)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigFiveTraits {
    /// Openness to experience (0.0 to 1.0)
    pub openness: f32,

    /// Conscientiousness (0.0 to 1.0)
    pub conscientiousness: f32,

    /// Extraversion (0.0 to 1.0)
    pub extraversion: f32,

    /// Agreeableness (0.0 to 1.0)
    pub agreeableness: f32,

    /// Neuroticism (0.0 to 1.0)
    pub neuroticism: f32,
}

/// Emotional tendencies and patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalTendencies {
    /// How quickly emotions change (0.0 to 1.0)
    pub volatility: f32,

    /// Baseline optimism level (0.0 to 1.0)
    pub optimism: f32,

    /// Emotional resilience (0.0 to 1.0)
    pub resilience: f32,

    /// Empathy level (0.0 to 1.0)
    pub empathy: f32,

    /// Emotional expressiveness (0.0 to 1.0)
    pub expressiveness: f32,
}

/// Cognitive processing style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveStyle {
    /// Analytical vs. intuitive thinking (0.0 = intuitive, 1.0 = analytical)
    pub analytical_thinking: f32,

    /// Processing speed preference (0.0 = deliberate, 1.0 = fast)
    pub processing_speed: f32,

    /// Attention to detail (0.0 to 1.0)
    pub attention_to_detail: f32,

    /// Creativity level (0.0 to 1.0)
    pub creativity: f32,

    /// Risk tolerance (0.0 to 1.0)
    pub risk_tolerance: f32,
}

/// Social interaction preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialOrientation {
    /// Preference for collaboration (0.0 to 1.0)
    pub collaboration_preference: f32,

    /// Leadership tendency (0.0 to 1.0)
    pub leadership: f32,

    /// Social sensitivity (0.0 to 1.0)
    pub social_sensitivity: f32,

    /// Assertiveness level (0.0 to 1.0)
    pub assertiveness: f32,

    /// Trust in others (0.0 to 1.0)
    pub trust: f32,
}

/// Communication style preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStyle {
    /// Formality level (0.0 = casual, 1.0 = formal)
    pub formality: f32,

    /// Verbosity (0.0 = concise, 1.0 = verbose)
    pub verbosity: f32,

    /// Directness (0.0 = indirect, 1.0 = direct)
    pub directness: f32,

    /// Emotional expression in communication (0.0 to 1.0)
    pub emotional_expression: f32,

    /// Use of humor (0.0 to 1.0)
    pub humor: f32,

    /// Technical language preference (0.0 = simple, 1.0 = technical)
    pub technical_language: f32,
}

/// Behavioral patterns and tendencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralPatterns {
    /// Goal orientation (0.0 to 1.0)
    pub goal_orientation: f32,

    /// Persistence level (0.0 to 1.0)
    pub persistence: f32,

    /// Adaptability (0.0 to 1.0)
    pub adaptability: f32,

    /// Curiosity level (0.0 to 1.0)
    pub curiosity: f32,

    /// Helpfulness (0.0 to 1.0)
    pub helpfulness: f32,

    /// Initiative taking (0.0 to 1.0)
    pub initiative: f32,
}

/// Agent capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    /// Text processing and generation
    TextProcessing,

    /// Emotional intelligence
    EmotionalIntelligence,

    /// Memory and recall
    Memory,

    /// Tool usage
    ToolUse,

    /// Web browsing
    WebAccess,

    /// Blockchain interaction
    Web3,

    /// Image processing
    ImageProcessing,

    /// Image generation
    ImageGeneration,

    /// Audio processing
    AudioProcessing,

    /// Speech generation
    SpeechGeneration,

    /// Code execution
    CodeExecution,

    /// Mathematical computation
    Mathematics,

    /// Data analysis
    DataAnalysis,

    /// Creative writing
    CreativeWriting,

    /// Research
    Research,

    /// Planning and scheduling
    Planning,

    /// Learning and adaptation
    Learning,

    /// Custom capability
    Custom(String),
}

/// Character archetype for common patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CharacterArchetype {
    /// Technical assistant
    TechnicalAssistant,

    /// Creative collaborator
    CreativeCollaborator,

    /// Customer service representative
    CustomerService,

    /// Research assistant
    ResearchAssistant,

    /// Educational tutor
    EducationalTutor,

    /// Personal assistant
    PersonalAssistant,

    /// Domain expert
    DomainExpert(String),

    /// Custom archetype
    Custom(String),
}

impl CharacterSheet {
    /// Create a new character sheet with basic traits
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            personality: Personality::default(),
            capabilities: Vec::new(),
            knowledge_domains: Vec::new(),
            communication_style: CommunicationStyle::default(),
            behavioral_patterns: BehavioralPatterns::default(),
            metadata: HashMap::new(),
        }
    }

    /// Create character from archetype
    pub fn from_archetype(name: String, archetype: CharacterArchetype) -> Self {
        let mut character = Self::new(name, archetype.description());

        match archetype {
            CharacterArchetype::TechnicalAssistant => {
                character.personality.big_five.conscientiousness = 0.9;
                character.personality.big_five.openness = 0.8;
                character.personality.cognitive_style.analytical_thinking = 0.9;
                character.communication_style.technical_language = 0.8;
                character.capabilities = vec![
                    Capability::TextProcessing,
                    Capability::ToolUse,
                    Capability::CodeExecution,
                    Capability::Mathematics,
                ];
            }

            CharacterArchetype::CreativeCollaborator => {
                character.personality.big_five.openness = 0.95;
                character.personality.big_five.extraversion = 0.7;
                character.personality.cognitive_style.creativity = 0.9;
                character.communication_style.emotional_expression = 0.8;
                character.capabilities = vec![
                    Capability::TextProcessing,
                    Capability::CreativeWriting,
                    Capability::ImageGeneration,
                    Capability::EmotionalIntelligence,
                ];
            }

            CharacterArchetype::CustomerService => {
                character.personality.big_five.agreeableness = 0.95;
                character.personality.big_five.extraversion = 0.8;
                character.personality.emotional_tendencies.empathy = 0.9;
                character.behavioral_patterns.helpfulness = 0.95;
                character.capabilities = vec![
                    Capability::TextProcessing,
                    Capability::EmotionalIntelligence,
                    Capability::Memory,
                ];
            }

            CharacterArchetype::ResearchAssistant => {
                character.personality.big_five.openness = 0.95;
                character.personality.big_five.conscientiousness = 0.8;
                character.personality.cognitive_style.analytical_thinking = 0.85;
                character.behavioral_patterns.curiosity = 0.9;
                character.capabilities = vec![
                    Capability::TextProcessing,
                    Capability::Research,
                    Capability::WebAccess,
                    Capability::DataAnalysis,
                ];
            }

            _ => {
                // Default configuration for other archetypes
            }
        }

        character
    }

    /// Add a capability
    pub fn add_capability(&mut self, capability: Capability) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }

    /// Remove a capability
    pub fn remove_capability(&mut self, capability: &Capability) {
        self.capabilities.retain(|c| c != capability);
    }

    /// Check if character has a capability
    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }

    /// Add knowledge domain
    pub fn add_knowledge_domain(&mut self, domain: String) {
        if !self.knowledge_domains.contains(&domain) {
            self.knowledge_domains.push(domain);
        }
    }
}

impl CharacterArchetype {
    fn description(&self) -> String {
        match self {
            CharacterArchetype::TechnicalAssistant => {
                "A helpful technical assistant focused on problem-solving and accuracy".to_string()
            }
            CharacterArchetype::CreativeCollaborator => {
                "A creative and imaginative collaborator for artistic and innovative projects"
                    .to_string()
            }
            CharacterArchetype::CustomerService => {
                "A friendly and helpful customer service representative".to_string()
            }
            CharacterArchetype::ResearchAssistant => {
                "A thorough and methodical research assistant".to_string()
            }
            CharacterArchetype::EducationalTutor => {
                "A patient and knowledgeable educational tutor".to_string()
            }
            CharacterArchetype::PersonalAssistant => {
                "A reliable and organized personal assistant".to_string()
            }
            CharacterArchetype::DomainExpert(domain) => format!("An expert in {}", domain),
            CharacterArchetype::Custom(desc) => desc.clone(),
        }
    }
}

// Default implementations
impl Default for Personality {
    fn default() -> Self {
        Self {
            big_five: BigFiveTraits::default(),
            emotional_tendencies: EmotionalTendencies::default(),
            cognitive_style: CognitiveStyle::default(),
            social_orientation: SocialOrientation::default(),
        }
    }
}

impl Default for BigFiveTraits {
    fn default() -> Self {
        Self {
            openness: 0.5,
            conscientiousness: 0.5,
            extraversion: 0.5,
            agreeableness: 0.5,
            neuroticism: 0.5,
        }
    }
}

impl Default for EmotionalTendencies {
    fn default() -> Self {
        Self {
            volatility: 0.4,
            optimism: 0.6,
            resilience: 0.7,
            empathy: 0.6,
            expressiveness: 0.5,
        }
    }
}

impl Default for CognitiveStyle {
    fn default() -> Self {
        Self {
            analytical_thinking: 0.5,
            processing_speed: 0.5,
            attention_to_detail: 0.5,
            creativity: 0.5,
            risk_tolerance: 0.5,
        }
    }
}

impl Default for SocialOrientation {
    fn default() -> Self {
        Self {
            collaboration_preference: 0.7,
            leadership: 0.5,
            social_sensitivity: 0.6,
            assertiveness: 0.5,
            trust: 0.6,
        }
    }
}

impl Default for CommunicationStyle {
    fn default() -> Self {
        Self {
            formality: 0.5,
            verbosity: 0.5,
            directness: 0.6,
            emotional_expression: 0.5,
            humor: 0.3,
            technical_language: 0.4,
        }
    }
}

impl Default for BehavioralPatterns {
    fn default() -> Self {
        Self {
            goal_orientation: 0.7,
            persistence: 0.6,
            adaptability: 0.6,
            curiosity: 0.6,
            helpfulness: 0.7,
            initiative: 0.5,
        }
    }
}

/// Re-export for convenience
pub use CharacterSheet as Character;
