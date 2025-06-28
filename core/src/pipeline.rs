//! Request processing pipeline implementation

use crate::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Pipeline stages
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineStage {
    PreOperation,
    Operation,
    PostOperation,
}

impl std::fmt::Display for PipelineStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineStage::PreOperation => write!(f, "pre-operation"),
            PipelineStage::Operation => write!(f, "operation"),
            PipelineStage::PostOperation => write!(f, "post-operation"),
        }
    }
}

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub stages: HashMap<PipelineStage, Vec<PluginConfig>>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            stages: HashMap::new(),
        }
    }
}

/// The main pipeline runner that executes plugins in stages
pub struct PipelineRunner {
    plugins: HashMap<PipelineStage, Vec<Arc<dyn PipelinePlugin>>>,
}

impl PipelineRunner {
    /// Create a new pipeline runner
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
    
    /// Register a plugin for a specific stage
    pub fn register_plugin(&mut self, stage: PipelineStage, plugin: Arc<dyn PipelinePlugin>) {
        self.plugins.entry(stage).or_insert_with(Vec::new).push(plugin);
    }
    
    /// Execute the pipeline for a request
    pub async fn execute(&self, mut ctx: RequestContext) -> Result<RequestContext, CoreError> {
        debug!("Starting pipeline execution for request {}", ctx.request_id);
        
        // Pre-operation stage
        ctx = self.execute_stage(PipelineStage::PreOperation, ctx).await?;
        if ctx.error.is_some() {
            return Ok(ctx);
        }
        
        // Operation stage (core business logic would be called here)
        ctx = self.execute_stage(PipelineStage::Operation, ctx).await?;
        if ctx.error.is_some() {
            return Ok(ctx);
        }
        
        // Post-operation stage
        ctx = self.execute_stage(PipelineStage::PostOperation, ctx).await?;
        
        info!(
            "Pipeline execution completed for request {} in {:?}",
            ctx.request_id,
            ctx.elapsed()
        );
        
        Ok(ctx)
    }
    
    /// Execute plugins for a specific stage
    async fn execute_stage(&self, stage: PipelineStage, mut ctx: RequestContext) -> Result<RequestContext, CoreError> {
        if let Some(plugins) = self.plugins.get(&stage) {
            debug!("Executing {} plugins for stage {}", plugins.len(), stage);
            
            for (index, plugin) in plugins.iter().enumerate() {
                debug!("Executing plugin {} ({}) for stage {}", plugin.name(), index + 1, stage);
                
                match plugin.call(&mut ctx).await {
                    PluginOutcome::Continue => {
                        debug!("Plugin {} returned Continue", plugin.name());
                        continue;
                    }
                    PluginOutcome::Halt => {
                        info!("Plugin {} halted pipeline execution", plugin.name());
                        break;
                    }
                    PluginOutcome::HaltWithError(e) => {
                        error!("Plugin {} halted with error: {}", plugin.name(), e);
                        ctx.error = Some(e.to_string());
                        break;
                    }
                }
            }
        }
        
        Ok(ctx)
    }
    
    /// Get the number of plugins registered for a stage
    pub fn plugin_count(&self, stage: &PipelineStage) -> usize {
        self.plugins.get(stage).map_or(0, |plugins| plugins.len())
    }
    
    /// Get all registered stages
    pub fn stages(&self) -> Vec<PipelineStage> {
        self.plugins.keys().cloned().collect()
    }
}

impl Default for PipelineRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in audit trail plugin
pub struct AuditTrailPlugin {
    name: &'static str,
}

impl AuditTrailPlugin {
    pub fn new() -> Self {
        Self {
            name: "AuditTrail",
        }
    }
}

impl Default for AuditTrailPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PipelinePlugin for AuditTrailPlugin {
    fn name(&self) -> &'static str {
        self.name
    }
    
    async fn init(&mut self, _config: PluginConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Initialized AuditTrail plugin");
        Ok(())
    }
    
    async fn call(&self, ctx: &mut RequestContext) -> PluginOutcome {
        // Log the request details
        info!(
            "Audit: {} {} by tenant {:?} (request_id: {})",
            ctx.method,
            ctx.path,
            ctx.tenant_id,
            ctx.request_id
        );
        
        // Store audit information
        ctx.set_attribute("audit_timestamp", serde_json::json!(chrono::Utc::now().to_rfc3339()));
        ctx.set_attribute("audit_logged", serde_json::json!(true));
        
        PluginOutcome::Continue
    }
}

/// Built-in request logging plugin
pub struct RequestLoggingPlugin {
    name: &'static str,
}

impl RequestLoggingPlugin {
    pub fn new() -> Self {
        Self {
            name: "RequestLogging",
        }
    }
}

impl Default for RequestLoggingPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PipelinePlugin for RequestLoggingPlugin {
    fn name(&self) -> &'static str {
        self.name
    }
    
    async fn init(&mut self, _config: PluginConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Initialized RequestLogging plugin");
        Ok(())
    }
    
    async fn call(&self, ctx: &mut RequestContext) -> PluginOutcome {
        debug!(
            "Request: {} {} (ID: {}, Elapsed: {:?})",
            ctx.method,
            ctx.path,
            ctx.request_id,
            ctx.elapsed()
        );
        
        PluginOutcome::Continue
    }
}

/// Built-in tenant validation plugin
pub struct TenantValidationPlugin {
    name: &'static str,
}

impl TenantValidationPlugin {
    pub fn new() -> Self {
        Self {
            name: "TenantValidation",
        }
    }
}

impl Default for TenantValidationPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PipelinePlugin for TenantValidationPlugin {
    fn name(&self) -> &'static str {
        self.name
    }
    
    async fn init(&mut self, _config: PluginConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Initialized TenantValidation plugin");
        Ok(())
    }
    
    async fn call(&self, ctx: &mut RequestContext) -> PluginOutcome {
        // Check if tenant is required for this endpoint
        if ctx.path.contains("/graph/") || ctx.path.contains("/llm/") {
            if ctx.tenant_id.is_none() {
                warn!("Request to {} requires tenant but none provided", ctx.path);
                ctx.error = Some("Tenant ID is required for this operation".to_string());
                return PluginOutcome::Halt;
            }
        }
        
        PluginOutcome::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestPlugin {
        name: &'static str,
        call_count: Arc<AtomicUsize>,
    }

    impl TestPlugin {
        fn new(name: &'static str) -> Self {
            Self {
                name,
                call_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn call_count(&self) -> usize {
            self.call_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl PipelinePlugin for TestPlugin {
        fn name(&self) -> &'static str {
            self.name
        }

        async fn init(&mut self, _config: PluginConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }

        async fn call(&self, _ctx: &mut RequestContext) -> PluginOutcome {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            PluginOutcome::Continue
        }
    }

    #[tokio::test]
    async fn test_pipeline_execution() {
        let mut runner = PipelineRunner::new();
        
        let plugin1 = Arc::new(TestPlugin::new("TestPlugin1"));
        let plugin2 = Arc::new(TestPlugin::new("TestPlugin2"));
        
        runner.register_plugin(PipelineStage::PreOperation, plugin1.clone());
        runner.register_plugin(PipelineStage::PreOperation, plugin2.clone());
        
        let ctx = RequestContext::new("GET".to_string(), "/test".to_string());
        let result = runner.execute(ctx).await;
        
        assert!(result.is_ok());
        assert_eq!(plugin1.call_count(), 1);
        assert_eq!(plugin2.call_count(), 1);
    }

    #[tokio::test]
    async fn test_plugin_halt() {
        struct HaltPlugin;

        #[async_trait]
        impl PipelinePlugin for HaltPlugin {
            fn name(&self) -> &'static str {
                "HaltPlugin"
            }

            async fn init(&mut self, _config: PluginConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                Ok(())
            }

            async fn call(&self, _ctx: &mut RequestContext) -> PluginOutcome {
                PluginOutcome::Halt
            }
        }

        let mut runner = PipelineRunner::new();
        let halt_plugin = Arc::new(HaltPlugin);
        let test_plugin = Arc::new(TestPlugin::new("AfterHalt"));
        
        runner.register_plugin(PipelineStage::PreOperation, halt_plugin);
        runner.register_plugin(PipelineStage::PreOperation, test_plugin.clone());
        
        let ctx = RequestContext::new("GET".to_string(), "/test".to_string());
        let result = runner.execute(ctx).await;
        
        assert!(result.is_ok());
        // Plugin after halt should not be called
        assert_eq!(test_plugin.call_count(), 0);
    }

    #[test]
    fn test_request_context() {
        let mut ctx = RequestContext::new("POST".to_string(), "/api/test".to_string());
        
        assert_eq!(ctx.method, "POST");
        assert_eq!(ctx.path, "/api/test");
        assert!(ctx.tenant_id.is_none());
        
        ctx.set_attribute("test_key", serde_json::json!("test_value"));
        assert_eq!(ctx.get_attribute("test_key"), Some(&serde_json::json!("test_value")));
    }
}