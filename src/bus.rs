use crossbeam_channel::Sender;
use std::{ sync::Arc };

use crate::{
    adapters::{ AdapterStatic, StartPolicy },
    events::{ ActionTarget, AdapterControl, AdapterTarget, ErasedTopic, RuntimeMsg, TopicId },
    logger::Level,
    sd_protocol::Outgoing,
};

/// Object-safe bus used everywhere.
pub trait Bus: Send + Sync {
    // Stream Deck out
    fn sd(&self, msg: Outgoing);

    // Logging
    fn log(&self, msg: &str, level: Level);

    // Unified notifies (erased payload + target)
    fn action_notify(&self, target: ActionTarget, event: Arc<ErasedTopic>);
    fn adapters_notify(&self, target: AdapterTarget, event: Arc<ErasedTopic>);

    // Adapter control
    fn adapter(&self, ctl: AdapterControl);
}

/// Thin, threadsafe bridge for threads to talk to the main loop.
#[derive(Clone)]
pub(crate) struct Emitter {
    tx: Sender<RuntimeMsg>,
}

impl Emitter {
    pub(crate) fn new(tx: Sender<RuntimeMsg>) -> Self {
        Self { tx }
    }
}

impl Bus for Emitter {
    fn sd(&self, msg: Outgoing) {
        let _ = self.tx.send(RuntimeMsg::Outgoing(msg));
    }

    fn log(&self, msg: &str, level: Level) {
        let _ = self.tx.send(RuntimeMsg::Log { msg: msg.to_string(), level });
    }

    fn action_notify(&self, target: ActionTarget, event: Arc<ErasedTopic>) {
        let _ = self.tx.send(RuntimeMsg::ActionNotify { target, event });
    }

    fn adapters_notify(&self, target: AdapterTarget, event: Arc<ErasedTopic>) {
        let _ = self.tx.send(RuntimeMsg::AdapterNotify { target, event });
    }

    fn adapter(&self, ctl: AdapterControl) {
        let _ = self.tx.send(RuntimeMsg::Adapter(ctl));
    }
}

/// Typed sugar on top of the object-safe Bus.
/// Kept in the same module so you don’t need a separate import.
pub trait BusTyped {
    // ----- actions -----
    fn action_notify_t<T: 'static + Send + Sync>(
        &self,
        target: ActionTarget,
        id: TopicId<T>,
        context: Option<String>,
        value: T
    );

    fn action_notify_all_t<T: 'static + Send + Sync>(
        &self,
        id: TopicId<T>,
        context: Option<String>,
        value: T
    ) {
        self.action_notify_t(ActionTarget::All, id, context, value);
    }

    fn action_notify_context_t<T: 'static + Send + Sync>(
        &self,
        ctx: impl Into<String>,
        id: TopicId<T>,
        context: Option<String>,
        value: T
    ) {
        self.action_notify_t(ActionTarget::Context(ctx.into()), id, context, value);
    }

    fn action_notify_id_t<T: 'static + Send + Sync>(
        &self,
        action_id: &'static str,
        id: TopicId<T>,
        context: Option<String>,
        value: T
    ) {
        self.action_notify_t(ActionTarget::Id(action_id), id, context, value);
    }

    fn action_notify_id_of<A: crate::actions::ActionStatic, T: 'static + Send + Sync>(
        &self,
        id: TopicId<T>,
        context: Option<String>,
        value: T
    ) {
        self.action_notify_t(ActionTarget::Id(A::ID), id, context, value);
    }

    fn action_notify_topic_t<T: 'static + Send + Sync>(
        &self,
        id: TopicId<T>,
        context: Option<String>,
        value: T
    ) {
        self.action_notify_t(ActionTarget::Topic(id.name), id, context, value);
    }

    // ----- adapters -----
    fn adapters_notify_t<T: 'static + Send + Sync>(
        &self,
        target: AdapterTarget,
        id: TopicId<T>,
        context: Option<String>,
        value: T
    );

    fn adapters_notify_all_t<T: 'static + Send + Sync>(
        &self,
        id: TopicId<T>,
        context: Option<String>,
        value: T
    ) {
        self.adapters_notify_t(AdapterTarget::All, id, context, value);
    }

    fn adapters_notify_policy_t<T: 'static + Send + Sync>(
        &self,
        policy: StartPolicy,
        id: TopicId<T>,
        context: Option<String>,
        value: T
    ) {
        self.adapters_notify_t(AdapterTarget::Policy(policy), id, context, value);
    }

    fn adapters_notify_name_t<T: 'static + Send + Sync>(
        &self,
        name: &'static str,
        id: TopicId<T>,
        context: Option<String>,
        value: T
    ) {
        self.adapters_notify_t(AdapterTarget::Name(name), id, context, value);
    }

    fn adapters_notify_topic_t<T: 'static + Send + Sync>(
        &self,
        id: TopicId<T>,
        context: Option<String>,
        value: T,
    ) {
        self.adapters_notify_t(AdapterTarget::Topic(id.name), id, context, value);
    }

    /// No-string helper: target a named adapter by its **type**.
    fn adapters_notify_name_of<A: AdapterStatic, T: 'static + Send + Sync>(
        &self,
        id: TopicId<T>,
        context: Option<String>,
        value: T
    ) {
        self.adapters_notify_t(AdapterTarget::Name(A::NAME), id, context, value);
    }

    // --- optional ergonomic adapter control ---
    #[allow(dead_code)]
    fn adapter_start(&self, name: &'static str) where Self: Sized {
        self.adapter(AdapterControl::Start(AdapterTarget::Name(name)));
    }

    #[allow(dead_code)]
    fn adapter_stop(&self, name: &'static str) where Self: Sized {
        self.adapter(AdapterControl::Stop(AdapterTarget::Name(name)));
    }

    #[allow(dead_code)]
    fn adapter_restart(&self, name: &'static str) where Self: Sized {
        self.adapter(AdapterControl::Restart(AdapterTarget::Name(name)));
    }

    /// Expose raw adapter control for the helpers above.
    fn adapter(&self, ctl: AdapterControl);
}

impl<B: Bus + ?Sized> BusTyped for B {
    #[inline]
    fn action_notify_t<T: 'static + Send + Sync>(
        &self,
        target: ActionTarget,
        id: TopicId<T>,
        context: Option<String>,
        value: T
    ) {
        let ev = Arc::new(ErasedTopic::new(id, context, value));
        self.action_notify(target, ev);
    }

    #[inline]
    fn adapters_notify_t<T: 'static + Send + Sync>(
        &self,
        target: AdapterTarget,
        id: TopicId<T>,
        context: Option<String>,
        value: T
    ) {
        let ev = Arc::new(ErasedTopic::new(id, context, value));
        self.adapters_notify(target, ev);
    }

    #[inline]
    fn adapter(&self, ctl: AdapterControl) {
        Bus::adapter(self, ctl);
    }
}
