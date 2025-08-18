use std::{ any::Any, marker::PhantomData, sync::Arc };
use crate::{ adapters::StartPolicy, logger::Level, sd_protocol::{ Outgoing, StreamDeckEvent } };

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterTarget {
    All,
    Policy(StartPolicy),
    Topic(&'static str),
    Name(&'static str),
}

impl AdapterTarget {
    pub fn all() -> Self {
        Self::All
    }
    pub fn policy(p: StartPolicy) -> Self {
        Self::Policy(p)
    }
    pub fn topic(t: &'static str) -> Self {
        Self::Topic(t)
    }
    pub fn name(n: &'static str) -> Self {
        Self::Name(n)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionTarget {
    All,
    Context(String),
    Topic(&'static str),
    Id(&'static str),
}

impl ActionTarget {
    pub fn all() -> Self {
        Self::All
    }
    pub fn context(id: impl Into<String>) -> Self {
        Self::Context(id.into())
    }
    pub fn topic(t: &'static str) -> Self {
        Self::Topic(t)
    }
    pub fn id(n: &'static str) -> Self {
        Self::Id(n)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterControl {
    Start(AdapterTarget),
    Stop(AdapterTarget),
    Restart(AdapterTarget),
}

#[derive(Copy, Clone)]
pub struct TopicId<T: 'static> {
    pub name: &'static str,
    _pd: PhantomData<fn() -> T>,
}
impl<T: 'static> TopicId<T> {
    pub const fn new(name: &'static str) -> Self {
        Self { name, _pd: PhantomData }
    }
}

pub struct ErasedTopic {
    name: &'static str,
    context: Option<String>,
    payload: Box<dyn Any + Send + Sync>,
}

impl std::fmt::Debug for ErasedTopic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErasedTopic")
            .field("name", &self.name)
            .field("context", &self.context)
            .finish_non_exhaustive()
    }
}

impl ErasedTopic {
    pub fn new<T: 'static + Send + Sync>(
        id: TopicId<T>,
        context: Option<String>,
        value: T
    ) -> Self {
        Self { name: id.name, context, payload: Box::new(value) }
    }

    #[inline] pub fn name(&self) -> &'static str { self.name }
    #[inline] pub fn context(&self) -> Option<&str> { self.context.as_deref() }

    #[inline]
    pub fn is<T: 'static>(&self, id: TopicId<T>) -> bool {
        self.name == id.name && self.payload.is::<T>()
    }

    pub fn downcast<T: 'static>(&self, id: TopicId<T>) -> Option<&T> {
        if self.name != id.name {
            return None;
        }
        self.payload.downcast_ref::<T>()
    }
    pub fn downcast_mut<T: 'static>(&mut self, id: TopicId<T>) -> Option<&mut T> {
        if self.name != id.name {
            return None;
        }
        self.payload.downcast_mut::<T>()
    }
    pub fn into_downcast<T: 'static>(self, id: TopicId<T>) -> Option<T> {
        if self.name != id.name {
            return None;
        }
        self.payload
            .downcast::<T>()
            .ok()
            .map(|b| *b)
    }
}

pub(crate) enum RuntimeMsg {
    /// Outgoing message to send to the Stream Deck.
    Outgoing(Outgoing),
    /// Incoming message from the Stream Deck.
    Incoming(StreamDeckEvent),
    /// Log message from a worker thread.
    Log {
        msg: String,
        level: Level,
    },
    ActionNotify {
        target: ActionTarget,
        event: Arc<ErasedTopic>,
    },
    AdapterNotify {
        target: AdapterTarget,
        event: Arc<ErasedTopic>,
    },
    Adapter(AdapterControl),
    Exit,
}
