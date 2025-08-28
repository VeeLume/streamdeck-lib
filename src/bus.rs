use crossbeam_channel::Sender;
use std::sync::Arc;

use crate::{
    adapters::StartPolicy,
    events::{ActionTarget, AdapterControl, AdapterTarget, ErasedTopic, RuntimeMsg, TopicId},
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
    fn publish(&self, event: Arc<ErasedTopic>);

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
        let _ = self.tx.send(RuntimeMsg::Log {
            msg: msg.to_string(),
            level,
        });
    }

    fn action_notify(&self, target: ActionTarget, event: Arc<ErasedTopic>) {
        let _ = self.tx.send(RuntimeMsg::ActionNotify { target, event });
    }

    fn adapters_notify(&self, target: AdapterTarget, event: Arc<ErasedTopic>) {
        let _ = self.tx.send(RuntimeMsg::AdapterNotify { target, event });
    }
    fn publish(&self, event: Arc<ErasedTopic>) {
        let _ = self.tx.send(RuntimeMsg::Publish(event));
    }

    fn adapter(&self, ctl: AdapterControl) {
        let _ = self.tx.send(RuntimeMsg::Adapter(ctl));
    }
}

/// Typed sugar on top of the object-safe Bus.
/// Kept in the same module so you don’t need a separate import.
pub trait BusTyped {
    fn publish_t<T: 'static + Send + Sync>(&self, id: TopicId<T>, value: T);

    fn action_notify_t<T: 'static + Send + Sync>(
        &self,
        target: ActionTarget,
        id: TopicId<T>,
        value: T,
    );
    fn action_notify_all_t<T: 'static + Send + Sync>(&self, id: TopicId<T>, value: T) {
        self.action_notify_t(ActionTarget::All, id, value);
    }
    fn action_notify_context_t<T: 'static + Send + Sync>(
        &self,
        ctx: impl Into<String>,
        id: TopicId<T>,
        value: T,
    ) {
        self.action_notify_t(ActionTarget::Context(ctx.into()), id, value);
    }
    fn action_notify_id_t<T: 'static + Send + Sync>(
        &self,
        action_id: &'static str,
        id: TopicId<T>,
        value: T,
    ) {
        self.action_notify_t(ActionTarget::Id(action_id), id, value);
    }
    fn action_notify_id_of<A: crate::actions::ActionStatic, T: 'static + Send + Sync>(
        &self,
        id: TopicId<T>,
        value: T,
    ) {
        self.action_notify_t(ActionTarget::Id(A::ID), id, value);
    }

    fn adapters_notify_t<T: 'static + Send + Sync>(
        &self,
        target: AdapterTarget,
        id: TopicId<T>,
        value: T,
    );
    fn adapters_notify_all_t<T: 'static + Send + Sync>(&self, id: TopicId<T>, value: T) {
        self.adapters_notify_t(AdapterTarget::All, id, value);
    }
    fn adapters_notify_policy_t<T: 'static + Send + Sync>(
        &self,
        policy: StartPolicy,
        id: TopicId<T>,
        value: T,
    ) {
        self.adapters_notify_t(AdapterTarget::Policy(policy), id, value);
    }
    fn adapters_notify_name_t<T: 'static + Send + Sync>(
        &self,
        name: &'static str,
        id: TopicId<T>,
        value: T,
    ) {
        self.adapters_notify_t(AdapterTarget::Name(name), id, value);
    }
    fn adapters_notify_name_of<A: crate::adapters::AdapterStatic, T: 'static + Send + Sync>(
        &self,
        id: TopicId<T>,
        value: T,
    ) {
        self.adapters_notify_t(AdapterTarget::Name(A::NAME), id, value);
    }

    #[deprecated(note = "Use publish_t(...) instead")]
    fn action_notify_topic_t<T: 'static + Send + Sync>(&self, id: TopicId<T>, value: T) {
        self.publish_t(id, value);
    }

    #[deprecated(note = "Use publish_t(...) instead")]
    fn adapters_notify_topic_t<T: 'static + Send + Sync>(&self, id: TopicId<T>, value: T) {
        self.publish_t(id, value);
    }

    fn adapter(&self, ctl: AdapterControl);
}

impl<B: Bus + ?Sized> BusTyped for B {
    #[inline]
    fn publish_t<T: 'static + Send + Sync>(&self, id: TopicId<T>, value: T) {
        self.publish(Arc::new(ErasedTopic::new(id, value)));
    }

    #[inline]
    fn action_notify_t<T: 'static + Send + Sync>(
        &self,
        target: ActionTarget,
        id: TopicId<T>,
        value: T,
    ) {
        self.action_notify(target, Arc::new(ErasedTopic::new(id, value)));
    }

    #[inline]
    fn adapters_notify_t<T: 'static + Send + Sync>(
        &self,
        target: AdapterTarget,
        id: TopicId<T>,
        value: T,
    ) {
        self.adapters_notify(target, Arc::new(ErasedTopic::new(id, value)));
    }

    #[inline]
    fn adapter(&self, ctl: AdapterControl) {
        Bus::adapter(self, ctl)
    }
}
