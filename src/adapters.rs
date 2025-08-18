use crossbeam_channel::Receiver;
use std::{sync::Arc, thread::JoinHandle};

use crate::{bus::Bus, context::Context, events::ErasedTopic};

/// How and when an adapter should be started/stopped.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartPolicy {
    /// Start immediately when runtime starts; stop when runtime ends.
    Eager,
    /// Start on ApplicationDidLaunch, stop on ApplicationDidTerminate.
    OnAppLaunch,
    /// Don't auto-start; expose this adapter behind a tag you can start/stop at runtime.
    /// The &'static str keeps this cheap and easy to compare.
    Manual,
}

/// Handle returned by `Adapter::start` so the runtime can shut it down.
pub struct AdapterHandle {
    join: Option<JoinHandle<()>>,
    shutdown: Box<dyn FnOnce() + Send + 'static>,
}

impl AdapterHandle {
    pub fn new(join: Option<JoinHandle<()>>, shutdown: impl FnOnce() + Send + 'static) -> Self {
        Self {
            join,
            shutdown: Box::new(shutdown),
        }
    }

    /// Run shutdown then join (best effort).
    pub fn shutdown(mut self) {
        (self.shutdown)();
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }

    /// Just join, if you already shut down out-of-band.
    pub fn join(mut self) {
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }

    /// Build a handle from a spawned thread and a shutdown fn.
    pub fn from_thread(join: JoinHandle<()>, shutdown: impl FnOnce() + Send + 'static) -> Self {
        Self::new(Some(join), shutdown)
    }

    /// Build a handle with no thread (pure async/signal adapter).
    pub fn from_shutdown(shutdown: impl FnOnce() + Send + 'static) -> Self {
        Self::new(None, shutdown)
    }

    pub fn from_crossbeam(
        join: JoinHandle<()>,
        shutdown_tx: crossbeam_channel::Sender<()>,
    ) -> Self {
        Self::new(Some(join), move || {
            let _ = shutdown_tx.send(());
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("initialization failed: {0}")]
    Init(String),
    #[error("runtime failed: {0}")]
    Runtime(String),
}
pub type AdapterResult = Result<AdapterHandle, AdapterError>;

pub trait AdapterStatic {
    const NAME: &'static str;
}

/// Implement this per “sidecar” (Mumble poller, file watcher, game API poller, …).
pub trait Adapter: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn policy(&self) -> StartPolicy {
        StartPolicy::Eager
    }
    fn topics(&self) -> &'static [&'static str] {
        &[]
    }

    fn start(
        &self,
        cx: &Context,
        bus: Arc<dyn Bus>,
        rx: Receiver<Arc<ErasedTopic>>,
    ) -> AdapterResult;
}
