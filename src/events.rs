use crate::{
    adapters::StartPolicy,
    sd_protocol::{Outgoing, StreamDeckEvent},
};
use std::{any::Any, marker::PhantomData, sync::Arc};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterTarget {
    All,
    Policy(StartPolicy),
    Name(&'static str),
    Label(&'static str),
}

impl AdapterTarget {
    pub fn all() -> Self {
        Self::All
    }
    pub fn policy(p: StartPolicy) -> Self {
        Self::Policy(p)
    }
    pub fn name(n: &'static str) -> Self {
        Self::Name(n)
    }
    pub fn label(l: &'static str) -> Self {
        Self::Label(l)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionTarget {
    All,
    Context(String),
    Id(&'static str),
}

impl ActionTarget {
    pub fn all() -> Self {
        Self::All
    }
    pub fn context(id: impl Into<String>) -> Self {
        Self::Context(id.into())
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
        Self {
            name,
            _pd: PhantomData,
        }
    }
}

pub struct ErasedTopic {
    name: &'static str,
    payload: Box<dyn Any + Send + Sync>,
}

impl ErasedTopic {
    pub fn new<T: 'static + Send + Sync>(id: TopicId<T>, value: T) -> Self {
        Self {
            name: id.name,
            payload: Box::new(value),
        }
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }
    #[inline]
    pub fn is<T: 'static>(&self, id: TopicId<T>) -> bool {
        self.name == id.name && self.payload.is::<T>()
    }
    pub fn downcast<T: 'static>(&self, id: TopicId<T>) -> Option<&T> {
        (self.name == id.name).then_some(())?;
        self.payload.downcast_ref::<T>()
    }
    pub fn downcast_mut<T: 'static>(&mut self, id: TopicId<T>) -> Option<&mut T> {
        (self.name == id.name).then_some(())?;
        self.payload.downcast_mut::<T>()
    }
    pub fn into_downcast<T: 'static>(self, id: TopicId<T>) -> Option<T> {
        (self.name == id.name).then_some(())?;
        self.payload.downcast::<T>().ok().map(|b| *b)
    }
}

// Debug impl no longer prints context
impl std::fmt::Debug for ErasedTopic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErasedTopic")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

pub(crate) enum RuntimeMsg {
    Outgoing(Outgoing),
    Incoming(StreamDeckEvent),
    Publish(Arc<ErasedTopic>),
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
