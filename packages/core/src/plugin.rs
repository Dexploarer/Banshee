use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

use crate::{Action, Evaluator, Provider, Result};

/// Unique identifier for pods
pub type PodId = String;

/// Legacy alias for backward compatibility
#[deprecated(note = "Use PodId instead")]
pub type PluginId = String;

/// Semantic version for dependency resolution
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(version_str: &str) -> Result<Self> {
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", version_str).into());
        }

        let major = parts[0]
            .parse()
            .map_err(|_| format!("Invalid major version: {}", parts[0]))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| format!("Invalid minor version: {}", parts[1]))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| format!("Invalid patch version: {}", parts[2]))?;

        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    pub fn is_compatible(&self, required: &VersionConstraint) -> bool {
        match required {
            VersionConstraint::Exact(v) => self == v,
            VersionConstraint::AtLeast(v) => self >= v,
            VersionConstraint::Compatible(v) => {
                self.major == v.major
                    && (self.minor > v.minor || (self.minor == v.minor && self.patch >= v.patch))
            }
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }
}

/// Version constraint for dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionConstraint {
    Exact(Version),
    AtLeast(Version),
    Compatible(Version), // Same major, at least minor.patch
}

/// Pod dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodDependency {
    pub pod_id: PodId,
    pub version: VersionConstraint,
    pub optional: bool,
}

/// Legacy alias for backward compatibility
#[deprecated(note = "Use PodDependency instead")]
pub type PluginDependency = PodDependency;

/// Pod capability that can be provided to other pods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodCapability {
    pub name: String,
    pub version: Version,
    pub description: String,
}

/// Legacy alias for backward compatibility
#[deprecated(note = "Use PodCapability instead")]
pub type PluginCapability = PodCapability;

/// Pod configuration with dependency support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodConfig {
    pub id: PodId,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub dependencies: Vec<PodDependency>,
    pub provides: Vec<PodCapability>,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Legacy alias for backward compatibility
#[deprecated(note = "Use PodConfig instead")]
pub type PluginConfig = PodConfig;

/// Pod initialization context with dependencies
#[derive(Clone)]
pub struct PodContext {
    pub dependencies: HashMap<PodId, Arc<dyn Pod>>,
    pub settings: HashMap<String, serde_json::Value>,
}

impl std::fmt::Debug for PodContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodContext")
            .field(
                "dependencies",
                &self.dependencies.keys().collect::<Vec<_>>(),
            )
            .field("settings", &self.settings)
            .finish()
    }
}

/// Legacy alias for backward compatibility
#[deprecated(note = "Use PodContext instead")]
pub type PluginContext = PodContext;

/// Pod result type
pub type PodResult<T> = std::result::Result<T, String>;

/// Legacy alias for backward compatibility
#[deprecated(note = "Use PodResult instead")]
pub type PluginResult<T> = std::result::Result<T, String>;

/// Core pod trait that all pods must implement
#[async_trait]
pub trait Pod: Send + Sync {
    /// Get the pod name
    fn name(&self) -> &str;

    /// Get the pod version
    fn version(&self) -> &str;

    /// Get the pod dependencies
    fn dependencies(&self) -> Vec<PodDependency> {
        Vec::new()
    }

    /// Get the capabilities this pod provides
    fn capabilities(&self) -> Vec<PodCapability> {
        Vec::new()
    }

    /// Initialize the pod with runtime context and dependencies
    async fn initialize(&mut self) -> PodResult<()>;

    /// Shutdown the pod gracefully
    async fn shutdown(&mut self) -> PodResult<()>;

    /// Get actions provided by this pod
    fn actions(&self) -> Vec<Box<dyn Action>> {
        Vec::new()
    }

    /// Get providers offered by this pod
    fn providers(&self) -> Vec<Box<dyn Provider>> {
        Vec::new()
    }

    /// Get evaluators supplied by this pod
    fn evaluators(&self) -> Vec<Box<dyn Evaluator>> {
        Vec::new()
    }

    /// Check if pod is healthy
    async fn health_check(&self) -> PodResult<bool> {
        Ok(true)
    }

    /// Called when a dependency becomes available
    async fn on_dependency_available(
        &mut self,
        _dependency_id: &str,
        _dependency: Arc<dyn Pod>,
    ) -> PodResult<()> {
        Ok(())
    }

    /// Called when a dependency becomes unavailable
    async fn on_dependency_unavailable(&mut self, _dependency_id: &str) -> PodResult<()> {
        Ok(())
    }
}

/// Legacy trait alias for backward compatibility
#[async_trait]
#[deprecated(note = "Use Pod trait instead")]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn dependencies(&self) -> Vec<PodDependency> {
        Vec::new()
    }
    fn capabilities(&self) -> Vec<PodCapability> {
        Vec::new()
    }
    async fn initialize(&mut self) -> PodResult<()>;
    async fn shutdown(&mut self) -> PodResult<()>;
    fn actions(&self) -> Vec<Box<dyn Action>> {
        Vec::new()
    }
    fn providers(&self) -> Vec<Box<dyn Provider>> {
        Vec::new()
    }
    fn evaluators(&self) -> Vec<Box<dyn Evaluator>> {
        Vec::new()
    }
    async fn health_check(&self) -> PodResult<bool> {
        Ok(true)
    }
    async fn on_dependency_available(
        &mut self,
        _dependency_id: &str,
        _dependency: Arc<dyn Pod>,
    ) -> PodResult<()> {
        Ok(())
    }
    async fn on_dependency_unavailable(&mut self, _dependency_id: &str) -> PodResult<()> {
        Ok(())
    }
}

/// Pod state tracking
#[derive(Debug, Clone, PartialEq)]
pub enum PodState {
    Unloaded,
    Loading,
    Loaded,
    Initializing,
    Running,
    Stopping,
    Stopped,
    Failed(String),
}

/// Legacy alias for backward compatibility
#[deprecated(note = "Use PodState instead")]
pub type PluginState = PodState;

/// Pod registry entry
struct PodEntry {
    pod: Arc<RwLock<Box<dyn Pod>>>,
    state: PodState,
    dependencies: Vec<PodId>,
    dependents: HashSet<PodId>,
}

impl std::fmt::Debug for PodEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodEntry")
            .field("state", &self.state)
            .field("dependencies", &self.dependencies)
            .field("dependents", &self.dependents)
            .finish()
    }
}

/// Registry for managing all available pods with dependency resolution
pub struct PodRegistry {
    pods: HashMap<PodId, PodEntry>,
    capabilities: HashMap<String, PodId>, // capability name -> pod providing it
    load_order: Vec<PodId>,               // Topologically sorted load order
    timeout_config: PodTimeoutConfig,     // Timeout configuration
}

/// Legacy alias for backward compatibility
#[deprecated(note = "Use PodRegistry instead")]
pub type PluginRegistry = PodRegistry;

impl PodRegistry {
    pub fn new() -> Self {
        Self {
            pods: HashMap::new(),
            capabilities: HashMap::new(),
            load_order: Vec::new(),
            timeout_config: PodTimeoutConfig::default(),
        }
    }

    /// Create a new registry with custom timeout configuration
    pub fn with_timeout_config(timeout_config: PodTimeoutConfig) -> Self {
        Self {
            pods: HashMap::new(),
            capabilities: HashMap::new(),
            load_order: Vec::new(),
            timeout_config,
        }
    }

    /// Register a new pod (doesn't initialize it yet)
    pub async fn register(&mut self, pod: Box<dyn Pod>) -> PodResult<()> {
        let pod_id = pod.name().to_string();

        // Check for duplicate registrations
        if self.pods.contains_key(&pod_id) {
            return Err(format!("Pod '{}' is already registered", pod_id));
        }

        let dependencies = pod.dependencies();
        let capabilities = pod.capabilities();

        // Register capabilities
        for capability in capabilities {
            if let Some(existing_provider) = self.capabilities.get(&capability.name) {
                return Err(format!(
                    "Capability '{}' is already provided by pod '{}'",
                    capability.name, existing_provider
                ));
            }
            self.capabilities
                .insert(capability.name.clone(), pod_id.clone());
        }

        let entry = PodEntry {
            pod: Arc::new(RwLock::new(pod)),
            state: PodState::Loaded,
            dependencies: dependencies.iter().map(|d| d.pod_id.clone()).collect(),
            dependents: HashSet::new(),
        };

        self.pods.insert(pod_id, entry);
        Ok(())
    }

    /// Initialize all pods in dependency order
    pub async fn initialize_all(&mut self) -> PodResult<()> {
        // Resolve dependencies and compute load order
        self.resolve_dependencies()?;

        // Initialize pods in topological order
        for pod_id in self.load_order.clone() {
            self.initialize_pod(&pod_id).await?;
        }

        Ok(())
    }

    /// Initialize a specific pod and its dependencies
    fn initialize_pod<'a>(
        &'a mut self,
        pod_id: &'a PodId,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = PodResult<()>> + 'a>> {
        Box::pin(async move {
            // Check if already initializing or running
            if let Some(entry) = self.pods.get(pod_id) {
                match entry.state {
                    PodState::Running => return Ok(()),
                    PodState::Initializing => {
                        return Err(format!(
                            "Circular dependency detected with pod '{}'",
                            pod_id
                        ))
                    }
                    PodState::Failed(ref err) => {
                        return Err(format!("Pod '{}' is in failed state: {}", pod_id, err))
                    }
                    _ => {}
                }
            } else {
                return Err(format!("Pod '{}' not found", pod_id));
            }

            // Set state to initializing
            self.pods.get_mut(pod_id).unwrap().state = PodState::Initializing;

            // Initialize dependencies first
            let dependencies = self.pods[pod_id].dependencies.clone();
            for dep_id in dependencies {
                if let Some(dep_entry) = self.pods.get(&dep_id) {
                    if dep_entry.state != PodState::Running {
                        Box::pin(self.initialize_pod(&dep_id)).await?;
                    }
                }
            }

            // Initialize the pod with timeout
            let pod = self.pods[pod_id].pod.clone();
            let timeout_duration = Duration::from_secs(self.timeout_config.init_timeout_secs);
            
            match timeout(timeout_duration, async move {
                let mut pod_guard = pod.write().await;
                pod_guard.initialize().await
            }).await {
                Ok(Ok(())) => {
                    self.pods.get_mut(pod_id).unwrap().state = PodState::Running;
                    Ok(())
                }
                Ok(Err(err)) => {
                    self.pods.get_mut(pod_id).unwrap().state = PodState::Failed(err.clone());
                    Err(format!("Failed to initialize pod '{}': {}", pod_id, err))
                }
                Err(_) => {
                    let err = format!("Timeout after {} seconds", self.timeout_config.init_timeout_secs);
                    self.pods.get_mut(pod_id).unwrap().state = PodState::Failed(err.clone());
                    Err(format!("Pod '{}' initialization timed out: {}", pod_id, err))
                }
            }
        })
    }

    /// Shutdown a pod and its dependents
    pub async fn shutdown(&mut self, pod_id: &PodId) -> PodResult<()> {
        // Get dependents first to avoid borrow conflicts
        let dependents = if let Some(entry) = self.pods.get(pod_id) {
            entry.dependents.clone()
        } else {
            return Ok(());
        };

        // Shutdown dependents first
        for dependent_id in dependents {
            Box::pin(self.shutdown(&dependent_id)).await?;
        }

        // Shutdown the pod with timeout
        if let Some(entry) = self.pods.get(pod_id) {
            let pod = entry.pod.clone();
            let timeout_duration = Duration::from_secs(self.timeout_config.shutdown_timeout_secs);
            let pod_id_clone = pod_id.clone();
            
            match timeout(timeout_duration, async move {
                let mut pod_guard = pod.write().await;
                pod_guard.shutdown().await
            }).await {
                Ok(Ok(())) => {
                    self.pods.get_mut(pod_id).unwrap().state = PodState::Stopped;
                    Ok(())
                }
                Ok(Err(err)) => {
                    // Still mark as stopped even if shutdown failed
                    self.pods.get_mut(pod_id).unwrap().state = PodState::Stopped;
                    Err(format!("Failed to shutdown pod '{}': {}", pod_id_clone, err))
                }
                Err(_) => {
                    // Timeout - force stop
                    self.pods.get_mut(pod_id).unwrap().state = PodState::Stopped;
                    Err(format!("Pod '{}' shutdown timed out after {} seconds", pod_id_clone, self.timeout_config.shutdown_timeout_secs))
                }
            }
        } else {
            Ok(())
        }
    }

    /// Shutdown all pods
    pub async fn shutdown_all(&mut self) -> PodResult<()> {
        // Shutdown in reverse load order
        let shutdown_order: Vec<_> = self.load_order.iter().rev().cloned().collect();

        for pod_id in shutdown_order {
            if let Some(entry) = self.pods.get(&pod_id) {
                if entry.state == PodState::Running {
                    let pod = entry.pod.clone();
                    let timeout_duration = Duration::from_secs(self.timeout_config.shutdown_timeout_secs);
                    
                    // Try to shutdown with timeout, but don't fail the overall process
                    let _ = timeout(timeout_duration, async move {
                        let mut pod_guard = pod.write().await;
                        pod_guard.shutdown().await
                    }).await;
                    
                    self.pods.get_mut(&pod_id).unwrap().state = PodState::Stopped;
                }
            }
        }

        Ok(())
    }

    /// Get a pod by ID
    pub fn get(&self, pod_id: &PodId) -> Option<Arc<RwLock<Box<dyn Pod>>>> {
        self.pods.get(pod_id).map(|entry| entry.pod.clone())
    }

    /// Get pod state
    pub fn get_state(&self, pod_id: &PodId) -> Option<PodState> {
        self.pods.get(pod_id).map(|entry| entry.state.clone())
    }

    /// Get all running pods
    pub fn running_pods(&self) -> Vec<PodId> {
        self.pods
            .iter()
            .filter(|(_, entry)| entry.state == PodState::Running)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Resolve pod dependencies using topological sort
    fn resolve_dependencies(&mut self) -> PodResult<()> {
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        let mut load_order = Vec::new();

        // Build dependency graph - collect dependencies first to avoid mutable borrow conflict
        let dependencies: Vec<(PodId, Vec<PodId>)> = self
            .pods
            .iter()
            .map(|(pod_id, entry)| (pod_id.clone(), entry.dependencies.clone()))
            .collect();

        for (pod_id, deps) in dependencies {
            for dep_id in deps {
                if let Some(dep_entry) = self.pods.get_mut(&dep_id) {
                    dep_entry.dependents.insert(pod_id.clone());
                } else {
                    return Err(format!(
                        "Pod '{}' depends on unknown pod '{}'",
                        pod_id, dep_id
                    ));
                }
            }
        }

        // Topological sort using DFS
        let pod_ids: Vec<_> = self.pods.keys().cloned().collect();
        for pod_id in pod_ids {
            if !visited.contains(&pod_id) {
                self.topological_sort(&pod_id, &mut visited, &mut visiting, &mut load_order)?;
            }
        }

        self.load_order = load_order;
        Ok(())
    }

    fn topological_sort(
        &self,
        pod_id: &PodId,
        visited: &mut HashSet<PodId>,
        visiting: &mut HashSet<PodId>,
        load_order: &mut Vec<PodId>,
    ) -> PodResult<()> {
        if visiting.contains(pod_id) {
            return Err(format!(
                "Circular dependency detected involving pod '{}'",
                pod_id
            ));
        }

        if visited.contains(pod_id) {
            return Ok(());
        }

        visiting.insert(pod_id.clone());

        if let Some(entry) = self.pods.get(pod_id) {
            for dep_id in &entry.dependencies {
                self.topological_sort(dep_id, visited, visiting, load_order)?;
            }
        }

        visiting.remove(pod_id);
        visited.insert(pod_id.clone());
        load_order.push(pod_id.clone());

        Ok(())
    }
}

/// Pod initialization and shutdown timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodTimeoutConfig {
    /// Timeout for pod initialization in seconds
    pub init_timeout_secs: u64,
    /// Timeout for pod shutdown in seconds  
    pub shutdown_timeout_secs: u64,
}

impl Default for PodTimeoutConfig {
    fn default() -> Self {
        Self {
            init_timeout_secs: 30,
            shutdown_timeout_secs: 10,
        }
    }
}

/// Pod manager that coordinates pod lifecycle with dependency resolution
pub struct PodManager {
    registry: PodRegistry,
    timeout_config: PodTimeoutConfig,
}

impl PodManager {
    pub fn new() -> Self {
        Self {
            registry: PodRegistry::new(),
            timeout_config: PodTimeoutConfig::default(),
        }
    }

    /// Create a new pod manager with custom timeout configuration
    pub fn with_timeout_config(timeout_config: PodTimeoutConfig) -> Self {
        Self {
            registry: PodRegistry::with_timeout_config(timeout_config.clone()),
            timeout_config,
        }
    }

    /// Get the timeout configuration
    pub fn timeout_config(&self) -> &PodTimeoutConfig {
        &self.timeout_config
    }

    /// Register a pod
    pub async fn register(&mut self, pod: Box<dyn Pod>) -> PodResult<()> {
        self.registry.register(pod).await
    }

    /// Initialize all registered pods in dependency order
    pub async fn initialize_all(&mut self) -> PodResult<()> {
        self.registry.initialize_all().await
    }

    /// Shutdown all pods
    pub async fn shutdown_all(&mut self) -> PodResult<()> {
        self.registry.shutdown_all().await
    }

    /// Get a pod by ID
    pub fn get_pod(&self, pod_id: &str) -> Option<Arc<RwLock<Box<dyn Pod>>>> {
        self.registry.get(&pod_id.to_string())
    }

    /// Get pod state
    pub fn get_pod_state(&self, pod_id: &str) -> Option<PodState> {
        self.registry.get_state(&pod_id.to_string())
    }

    /// Get all running pods
    pub fn running_pods(&self) -> Vec<PodId> {
        self.registry.running_pods()
    }

    /// Get all actions from running pods
    pub async fn get_all_actions(&self) -> Vec<Box<dyn Action>> {
        let mut actions = Vec::new();
        for pod_id in self.running_pods() {
            if let Some(pod_arc) = self.get_pod(&pod_id) {
                let pod = pod_arc.read().await;
                actions.extend(pod.actions());
            }
        }
        actions
    }

    /// Get all providers from running pods
    pub async fn get_all_providers(&self) -> Vec<Box<dyn Provider>> {
        let mut providers = Vec::new();
        for pod_id in self.running_pods() {
            if let Some(pod_arc) = self.get_pod(&pod_id) {
                let pod = pod_arc.read().await;
                providers.extend(pod.providers());
            }
        }
        providers
    }

    /// Get all evaluators from running pods
    pub async fn get_all_evaluators(&self) -> Vec<Box<dyn Evaluator>> {
        let mut evaluators = Vec::new();
        for pod_id in self.running_pods() {
            if let Some(pod_arc) = self.get_pod(&pod_id) {
                let pod = pod_arc.read().await;
                evaluators.extend(pod.evaluators());
            }
        }
        evaluators
    }

    /// Health check for all running pods
    pub async fn health_check_all(&self) -> HashMap<PodId, bool> {
        let mut results = HashMap::new();

        for pod_id in self.running_pods() {
            if let Some(pod_arc) = self.get_pod(&pod_id) {
                let pod = pod_arc.read().await;
                let healthy = pod.health_check().await.unwrap_or(false);
                results.insert(pod_id, healthy);
            }
        }
        results
    }
}

/// Legacy alias for backward compatibility
#[deprecated(note = "Use PodManager instead")]
pub type PluginManager = PodManager;

/// Macro to help create pod dependencies
#[macro_export]
macro_rules! pod_dependency {
    ($pod_id:expr, $version:expr) => {
        PodDependency {
            pod_id: $pod_id.to_string(),
            version: VersionConstraint::Compatible(Version::parse($version).unwrap()),
            optional: false,
        }
    };

    ($pod_id:expr, $version:expr, optional) => {
        PodDependency {
            pod_id: $pod_id.to_string(),
            version: VersionConstraint::Compatible(Version::parse($version).unwrap()),
            optional: true,
        }
    };
}

/// Legacy macro alias for backward compatibility
#[macro_export]
#[deprecated(note = "Use pod_dependency! instead")]
macro_rules! plugin_dependency {
    ($plugin_id:expr, $version:expr) => {
        PodDependency {
            pod_id: $plugin_id.to_string(),
            version: VersionConstraint::Compatible(Version::parse($version).unwrap()),
            optional: false,
        }
    };

    ($plugin_id:expr, $version:expr, optional) => {
        PodDependency {
            pod_id: $plugin_id.to_string(),
            version: VersionConstraint::Compatible(Version::parse($version).unwrap()),
            optional: true,
        }
    };
}

/// Macro to help create pod capabilities
#[macro_export]
macro_rules! pod_capability {
    ($name:expr, $version:expr, $description:expr) => {
        PodCapability {
            name: $name.to_string(),
            version: Version::parse($version).unwrap(),
            description: $description.to_string(),
        }
    };
}

/// Legacy macro alias for backward compatibility
#[macro_export]
#[deprecated(note = "Use pod_capability! instead")]
macro_rules! plugin_capability {
    ($name:expr, $version:expr, $description:expr) => {
        PodCapability {
            name: $name.to_string(),
            version: Version::parse($version).unwrap(),
            description: $description.to_string(),
        }
    };
}

/// Helper trait for pods to implement simplified dependency management
pub trait PodExt: Pod {
    /// Get pod ID based on name
    fn id(&self) -> String {
        self.name().to_string()
    }

    /// Create a basic pod config
    fn config(&self) -> PodConfig {
        PodConfig {
            id: self.id(),
            name: self.name().to_string(),
            version: Version::parse(self.version()).unwrap_or(Version::new(0, 1, 0)),
            description: format!("{} pod", self.name()),
            dependencies: self.dependencies(),
            provides: self.capabilities(),
            settings: HashMap::new(),
        }
    }
}

// Blanket implementation for all pods
impl<T: Pod> PodExt for T {}

/// Legacy helper trait for backward compatibility
#[deprecated(note = "Use PodExt instead")]
pub trait PluginExt: Pod {
    fn id(&self) -> String {
        self.name().to_string()
    }

    fn config(&self) -> PodConfig {
        PodConfig {
            id: self.id(),
            name: self.name().to_string(),
            version: Version::parse(self.version()).unwrap_or(Version::new(0, 1, 0)),
            description: format!("{} pod", self.name()),
            dependencies: self.dependencies(),
            provides: self.capabilities(),
            settings: HashMap::new(),
        }
    }
}

// Blanket implementation for legacy trait
#[allow(deprecated)]
impl<T: Pod> PluginExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct TestPod {
        name: String,
        version: String,
        dependencies: Vec<PodDependency>,
    }

    impl TestPod {
        fn new(name: &str, version: &str) -> Self {
            Self {
                name: name.to_string(),
                version: version.to_string(),
                dependencies: Vec::new(),
            }
        }

        fn with_dependency(mut self, dep_name: &str, dep_version: &str) -> Self {
            self.dependencies
                .push(pod_dependency!(dep_name, dep_version));
            self
        }
    }

    #[async_trait]
    impl Pod for TestPod {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            &self.version
        }

        fn dependencies(&self) -> Vec<PodDependency> {
            self.dependencies.clone()
        }

        async fn initialize(&mut self) -> PodResult<()> {
            Ok(())
        }

        async fn shutdown(&mut self) -> PodResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_version_parsing() {
        let version = Version::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[tokio::test]
    async fn test_version_compatibility() {
        let v1_2_3 = Version::new(1, 2, 3);
        let v1_2_4 = Version::new(1, 2, 4);
        let v1_3_0 = Version::new(1, 3, 0);
        let v2_0_0 = Version::new(2, 0, 0);

        let compatible_constraint = VersionConstraint::Compatible(Version::new(1, 2, 0));

        assert!(v1_2_3.is_compatible(&compatible_constraint));
        assert!(v1_2_4.is_compatible(&compatible_constraint));
        assert!(v1_3_0.is_compatible(&compatible_constraint));
        assert!(!v2_0_0.is_compatible(&compatible_constraint));
    }

    #[tokio::test]
    async fn test_pod_registration() {
        let mut registry = PodRegistry::new();

        let pod = Box::new(TestPod::new("test", "1.0.0"));
        registry.register(pod).await.unwrap();

        assert!(registry.get(&"test".to_string()).is_some());
        assert_eq!(
            registry.get_state(&"test".to_string()),
            Some(PodState::Loaded)
        );
    }

    #[tokio::test]
    async fn test_dependency_resolution() {
        let mut registry = PodRegistry::new();

        // Register pods with dependencies: c -> b -> a
        let pod_a = Box::new(TestPod::new("a", "1.0.0"));
        let pod_b = Box::new(TestPod::new("b", "1.0.0").with_dependency("a", "1.0.0"));
        let pod_c = Box::new(TestPod::new("c", "1.0.0").with_dependency("b", "1.0.0"));

        registry.register(pod_c).await.unwrap();
        registry.register(pod_b).await.unwrap();
        registry.register(pod_a).await.unwrap();

        registry.initialize_all().await.unwrap();

        // All pods should be running
        assert_eq!(
            registry.get_state(&"a".to_string()),
            Some(PodState::Running)
        );
        assert_eq!(
            registry.get_state(&"b".to_string()),
            Some(PodState::Running)
        );
        assert_eq!(
            registry.get_state(&"c".to_string()),
            Some(PodState::Running)
        );

        // Load order should be a, b, c
        assert_eq!(registry.load_order, vec!["a", "b", "c"]);
    }

    #[tokio::test]
    async fn test_circular_dependency_detection() {
        let mut registry = PodRegistry::new();

        // Create circular dependency: a -> b -> a
        let pod_a = Box::new(TestPod::new("a", "1.0.0").with_dependency("b", "1.0.0"));
        let pod_b = Box::new(TestPod::new("b", "1.0.0").with_dependency("a", "1.0.0"));

        registry.register(pod_a).await.unwrap();
        registry.register(pod_b).await.unwrap();

        let result = registry.initialize_all().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular dependency"));
    }

    #[tokio::test]
    async fn test_pod_manager() {
        let mut manager = PodManager::new();

        let pod = Box::new(TestPod::new("test", "1.0.0"));
        manager.register(pod).await.unwrap();
        manager.initialize_all().await.unwrap();

        assert_eq!(manager.running_pods(), vec!["test"]);
        assert!(manager.get_pod("test").is_some());

        manager.shutdown_all().await.unwrap();
    }
}
