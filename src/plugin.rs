// plugin/builder.rs
use std::collections::HashMap;
use std::sync::Arc;

use crate::actions::{ActionFactory, ActionId};
use crate::adapters::Adapter;
use crate::context::{Context, Extensions};
use crate::hooks::AppHooks;
use crate::sd_protocol::SdClient;

/// The assembled plugin: actions, adapters, hooks, and extensions.
pub struct Plugin {
    actions: HashMap<ActionId, ActionFactory>,
    exts: Extensions,
    hooks: AppHooks,
    adapters: Vec<Arc<dyn Adapter + Send + Sync>>,
}

impl Default for Plugin {
    fn default() -> Self {
        Self {
            actions: HashMap::new(),
            exts: Extensions::default(),
            hooks: AppHooks::default(),
            adapters: Vec::new(),
        }
    }
}

impl Plugin {
    /// Start from empty plugin.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct from parts (handy for tests or struct-literal style).
    pub fn from_parts(
        actions: HashMap<ActionId, ActionFactory>,
        exts: Extensions,
        hooks: AppHooks,
        adapters: Vec<Arc<dyn Adapter + Send + Sync>>,
    ) -> Self {
        Self {
            actions,
            exts,
            hooks,
            adapters,
        }
    }

    /// Add a single action factory (chainable).
    pub fn add_action(mut self, reg: ActionFactory) -> Self {
        self.actions.insert(reg.id.clone(), reg);
        self
    }

    /// Add multiple actions (chainable).
    pub fn add_actions<I>(mut self, regs: I) -> Self
    where
        I: IntoIterator<Item = ActionFactory>,
    {
        for reg in regs {
            self.actions.insert(reg.id.clone(), reg);
        }
        self
    }

    /// Register a typed extension (chainable).
    pub fn add_extension<T>(mut self, value: Arc<T>) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.exts.provide::<T>(value);
        self
    }

    /// Replace the entire Extensions set (chainable).
    pub fn set_extensions(mut self, exts: Extensions) -> Self {
        self.exts = exts;
        self
    }

    /// Provide configured hooks (chainable).
    pub fn set_hooks(mut self, hooks: AppHooks) -> Self {
        self.hooks = hooks;
        self
    }

    /// Add an adapter by value (chainable).
    pub fn add_adapter<A>(mut self, a: A) -> Self
    where
        A: Adapter + Send + Sync + 'static,
    {
        self.adapters.push(Arc::new(a));
        self
    }

    /// Add an adapter you already have in an Arc (chainable).
    pub fn add_adapter_arc(mut self, a: Arc<dyn Adapter + Send + Sync>) -> Self {
        self.adapters.push(a);
        self
    }

    /// Build a Context using this plugin’s Extensions.
    pub(crate) fn make_context(
        &self,
        sd: Arc<SdClient>,
        plugin_uuid: String,
        bus: Arc<dyn crate::bus::Bus>,
    ) -> Context {
        Context::new(sd, plugin_uuid, self.exts.clone(), bus)
    }

    // ----- accessors kept for runtime -----

    pub fn actions(&self) -> &HashMap<ActionId, ActionFactory> {
        &self.actions
    }

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
