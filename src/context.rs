// lib/context.rs
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde_json::{Map, Value};
use tracing::error;

use crate::sd_protocol::SdClient;

// ======================
// Global Settings
// ======================

/// Thread-safe, push-on-write global settings cache.
/// All mutations push to Stream Deck automatically.
/// Only `hydrate_from_sd` writes without pushing (used when SD sends us a snapshot).
#[derive(Clone)]
pub struct GlobalSettings {
    sd: Arc<SdClient>,
    map: Arc<RwLock<Map<String, Value>>>,
}

impl GlobalSettings {
    pub(crate) fn new(sd: Arc<SdClient>) -> Self {
        Self {
            sd,
            map: Arc::new(RwLock::new(Map::new())),
        }
    }

    // ---- SD <-> cache sync (no push) -----------------------------------

    /// Replace the whole map from Stream Deck's snapshot (no push).
    /// Call from your `didReceiveGlobalSettings` handler.
    pub(crate) fn hydrate_from_sd(&self, from_sd: Map<String, Value>) {
        match self.map.write() {
            Ok(mut w) => {
                *w = from_sd;
            }
            Err(_) => error!(
                "GlobalSettings: write lock poisoned while hydrating from SD; keeping old cache"
            ),
        }
    }

    // ---- Reads ----------------------------------------------------------

    /// Clone of the entire map.
    pub fn snapshot(&self) -> Map<String, Value> {
        match self.map.read() {
            Ok(r) => r.clone(),
            Err(_) => {
                error!("GlobalSettings: read lock poisoned during snapshot; returning empty map");
                Map::new()
            }
        }
    }

    /// Get a single key.
    pub fn get(&self, key: &str) -> Option<Value> {
        match self.map.read() {
            Ok(r) => r.get(key).cloned(),
            Err(_) => {
                error!("GlobalSettings: read lock poisoned during get; returning None");
                None
            }
        }
    }

    /// Get multiple keys (present keys only).
    pub fn get_many(&self, keys: &[&str]) -> Map<String, Value> {
        let mut out = Map::new();
        match self.map.read() {
            Ok(r) => {
                for &k in keys {
                    if let Some(v) = r.get(k).cloned() {
                        out.insert(k.to_string(), v);
                    }
                }
            }
            Err(_) => error!("GlobalSettings: read lock poisoned during get_many; returning empty"),
        }
        out
    }

    // ---- Writes (auto-push) --------------------------------------------

    /// Replace all settings and push.
    pub fn replace(&self, new_map: Map<String, Value>) {
        if let Some(snapshot) = self.with_write_snapshot(|w| {
            *w = new_map;
        }) {
            self.sd.set_global_settings(snapshot);
        }
    }

    /// Set a single key and push.
    pub fn set(&self, key: impl Into<String>, value: Value) {
        if let Some(snapshot) = self.with_write_snapshot(|w| {
            w.insert(key.into(), value);
        }) {
            self.sd.set_global_settings(snapshot);
        }
    }

    /// Set multiple keys and push.
    pub fn set_many<I, K>(&self, entries: I)
    where
        I: IntoIterator<Item = (K, Value)>,
        K: Into<String>,
    {
        if let Some(snapshot) = self.with_write_snapshot(|w| {
            for (k, v) in entries {
                w.insert(k.into(), v);
            }
        }) {
            self.sd.set_global_settings(snapshot);
        }
    }

    /// Delete everything and push (leaves an empty object on SD).
    pub fn delete_all(&self) {
        if let Some(snapshot) = self.with_write_snapshot(|w| w.clear()) {
            self.sd.set_global_settings(snapshot);
        }
    }

    /// Delete a single key and push.
    pub fn delete(&self, key: &str) {
        if let Some(snapshot) = self.with_write_snapshot(|w| {
            w.remove(key);
        }) {
            self.sd.set_global_settings(snapshot);
        }
    }

    /// Delete multiple keys and push.
    pub fn delete_many(&self, keys: &[&str]) {
        if let Some(snapshot) = self.with_write_snapshot(|w| {
            for &k in keys {
                w.remove(k);
            }
        }) {
            self.sd.set_global_settings(snapshot);
        }
    }

    /// Batch-edit the settings and push once to Stream Deck.
    ///
    /// The closure receives a mutable view of the cached map. After it returns,
    /// the fresh snapshot is pushed via `set_global_settings`.
    /// Returns the closure's value on success, or `None` if the write lock was poisoned.
    pub fn with_mut<R, F>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut Map<String, Value>) -> R,
    {
        match self.map.write() {
            Ok(mut w) => {
                let ret = f(&mut w);
                let snapshot = w.clone(); // single push with the final state
                drop(w);
                self.sd.set_global_settings(snapshot);
                Some(ret)
            }
            Err(_) => {
                error!(
                    "GlobalSettings: write lock poisoned during with_mut; skipping mutation & push"
                );
                None
            }
        }
    }

    // ---- Internals ------------------------------------------------------

    /// Helper: run a write op and return the fresh snapshot, logging on lock errors.
    fn with_write_snapshot<F>(&self, f: F) -> Option<Map<String, Value>>
    where
        F: FnOnce(&mut Map<String, Value>),
    {
        match self.map.write() {
            Ok(mut w) => {
                f(&mut w);
                Some(w.clone())
            }
            Err(_) => {
                error!(
                    "GlobalSettings: write lock poisoned during mutation; skipping mutation & push"
                );
                None
            }
        }
    }
}

// ======================================
// Type-safe extension registry (plugins)
// ======================================

/// A type-indexed store for plugin-specific shared state.
/// Insert once; fetch anywhere by concrete type.
#[derive(Clone, Default)]
pub struct Extensions(pub Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>);

impl Extensions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Provide a typed extension (e.g., `TemplateStore`, `ActiveChar`, `BindingsStore`).
    pub fn provide<T>(&self, value: Arc<T>) -> &Self
    where
        T: Send + Sync + 'static,
    {
        if let Ok(mut w) = self.0.write() {
            w.insert(TypeId::of::<T>(), value);
        }
        self
    }

    /// Fetch a typed extension. Returns `None` if not registered.
    pub fn get<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.0
            .read()
            .ok()
            .and_then(|m| m.get(&TypeId::of::<T>()).cloned())
            .and_then(|arc_any| arc_any.downcast::<T>().ok())
    }

    pub fn require<T>(&self) -> Arc<T>
    where
        T: Send + Sync + 'static,
    {
        self.get::<T>().unwrap_or_else(|| {
            panic!(
                "Extensions: missing required extension {}",
                std::any::type_name::<T>()
            )
        })
    }
}

// ======================
// Context handed around
// ======================

#[derive(Clone)]
pub struct Context {
    sd: Arc<SdClient>,
    plugin_uuid: String,
    globals: GlobalSettings,
    exts: Extensions,
    bus: Arc<dyn crate::bus::Bus>,
}

impl Context {
    pub fn new(
        sd: Arc<SdClient>,
        plugin_uuid: String,
        exts: Extensions,
        bus: Arc<dyn crate::bus::Bus>,
    ) -> Self {
        let globals = GlobalSettings::new(Arc::clone(&sd));
        Self {
            sd,
            plugin_uuid,
            globals,
            exts,
            bus,
        }
    }

    pub fn sd(&self) -> &SdClient {
        &self.sd
    }
    pub fn bus(&self) -> Arc<dyn crate::bus::Bus> {
        Arc::clone(&self.bus)
    }
    pub fn uuid(&self) -> &str {
        &self.plugin_uuid
    }
    pub fn globals(&self) -> GlobalSettings {
        self.globals.clone()
    }
    pub fn exts(&self) -> Extensions {
        self.exts.clone()
    }

    pub fn try_ext<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.exts.get::<T>()
    }
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("plugin_uuid", &self.plugin_uuid)
            .finish()
    }
}
