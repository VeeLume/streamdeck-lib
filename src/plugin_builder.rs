// plugin/builder.rs
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

use crate::actions::{ ActionFactory, ActionId };
use crate::context::{ Context, Extensions };
use crate::hooks::AppHooks;
use crate::logger::ActionLog;
use crate::sd_protocol::SdClient;
use crate::adapters::{ Adapter };

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Missing required extension: {0}")] MissingExtension(&'static str),
}

#[derive(Default)]
pub struct PluginBuilder {
    actions: HashMap<ActionId, ActionFactory>,
    exts: Extensions,
    hooks: AppHooks,
    adapters: Vec<Arc<dyn Adapter + Send + Sync>>,
}

impl PluginBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_action(mut self, reg: ActionFactory) -> Self {
        self.actions.insert(reg.id.clone(), reg);
        self
    }

    /// Register a typed extension (TemplateStore, ActiveChar, BindingsStore, …)
    pub fn add_extension<T>(self, value: Arc<T>) -> Self where T: Send + Sync + 'static {
        self.exts.provide::<T>(value);
        self
    }

    /// Provide your configured AppHooks (closure chains and/or listeners).
    pub fn set_hooks(mut self, hooks: AppHooks) -> Self {
        self.hooks = hooks;
        self
    }

    pub fn add_adapter<A: Adapter + Send + Sync + 'static>(mut self, a: A) -> Self {
        self.adapters.push(Arc::new(a));
        self
    }

    pub fn build(self) -> Result<Plugin, BuildError> {
        Ok(Plugin {
            actions: self.actions,
            exts: self.exts,
            hooks: self.hooks,
            adapters: self.adapters,
        })
    }
}

pub struct Plugin {
    pub(crate) actions: HashMap<String, ActionFactory>,
    pub(crate) exts: Extensions,
    pub(crate) hooks: AppHooks,
    pub(crate) adapters: Vec<Arc<dyn Adapter + Send + Sync>>,
}

impl Plugin {
    pub(crate) fn make_context(
        &self,
        sd: Arc<SdClient>,
        log: Arc<dyn ActionLog>,
        plugin_uuid: String,
        exts: Extensions,
        bus: Arc<dyn crate::bus::Bus>
    ) -> Context {
        Context::new(sd, log, plugin_uuid, exts, bus)
    }

    // clean exposure for runtime
    pub fn actions(&self) -> &HashMap<ActionId, ActionFactory> {
        &self.actions
    }

    /// Owned copy (useful if your ActionManager wants ownership)
    pub fn actions_cloned(&self) -> HashMap<ActionId, ActionFactory> {
        self.actions.clone()
    }

    pub fn hooks(&self) -> &AppHooks {
        &self.hooks
    }
    pub fn adapters(&self) -> &[Arc<dyn Adapter + Send + Sync>] {
        &self.adapters
    }
    pub fn exts(&self) -> Extensions {
        self.exts.clone()
    }
}
