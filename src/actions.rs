// actions.rs
use std::sync::Arc;

use crate::{
    context::Context,
    events::ErasedTopic,
    sd_protocol::{StreamDeckEvent, views::*},
};

pub type ActionId = String;

/// Object-safe trait used by the runtime.
pub trait Action: Send + 'static {
    /// Return your action id (usually a string literal).
    fn id(&self) -> &str;

    /// Static subscriptions for ActionTarget::Topic fan-out (optional).
    fn topics(&self) -> &'static [&'static str] {
        &[]
    }

    fn init(&mut self, _cx: &Context, _ctx_id: &str) {}
    fn teardown(&mut self, _cx: &Context, _ctx_id: &str) {}

    // Lifecycle / keypad
    fn will_appear(&mut self, _cx: &Context, _ev: &WillAppear) {}
    fn will_disappear(&mut self, _cx: &Context, _ev: &WillDisappear) {}
    fn key_down(&mut self, _cx: &Context, _ev: &KeyDown) {}
    fn key_up(&mut self, _cx: &Context, _ev: &KeyUp) {}

    // Encoders / touch
    fn dial_down(&mut self, _cx: &Context, _ev: &DialDown) {}
    fn dial_up(&mut self, _cx: &Context, _ev: &DialUp) {}
    fn dial_rotate(&mut self, _cx: &Context, _ev: &DialRotate) {}
    fn touch_tap(&mut self, _cx: &Context, _ev: &TouchTap) {}

    // Title/PI/settings
    fn title_parameters_did_change(&mut self, _cx: &Context, _ev: &TitleParametersDidChange) {}
    fn property_inspector_did_appear(&mut self, _cx: &Context, _ev: &PropertyInspectorDidAppear) {}
    fn property_inspector_did_disappear(
        &mut self,
        _cx: &Context,
        _ev: &PropertyInspectorDidDisappear,
    ) {
    }
    fn did_receive_settings(&mut self, _cx: &Context, _ev: &DidReceiveSettings) {}
    fn did_receive_property_inspector_message(
        &mut self,
        _cx: &Context,
        _ev: &DidReceivePropertyInspectorMessage,
    ) {
    }

    fn on_global_event(&mut self, _cx: &Context, _ev: &StreamDeckEvent) {}

    /// Typed broadcasts from your runtime.
    fn on_notify(&mut self, _cx: &Context, _ctx_id: &str, _event: &ErasedTopic) {}
}

/// Compile-time helper (NOT a supertrait) for type-safe targeting and factories.
pub trait ActionStatic {
    const ID: &'static str;
}

/// Factory for action instances.
#[derive(Clone)]
pub struct ActionFactory {
    pub id: ActionId,
    pub build: Arc<dyn (Fn() -> Box<dyn Action>) + Send + Sync>,
}

impl std::fmt::Debug for ActionFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionFactory")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl ActionFactory {
    /// Explicit id.
    pub fn new<F, A>(id: impl Into<String>, factory: F) -> Self
    where
        F: Fn() -> A + Send + Sync + 'static,
        A: Action + 'static,
    {
        Self {
            id: id.into(),
            build: Arc::new(move || Box::new(factory())),
        }
    }

    /// Use the action type's compile-time id.
    pub fn from_static<A, F>(factory: F) -> Self
    where
        A: Action + ActionStatic + 'static,
        F: Fn() -> A + Send + Sync + 'static,
    {
        Self::new(A::ID, factory)
    }

    /// Convenience: default-constructible actions.
    pub fn default_of<A>() -> Self
    where
        A: Action + ActionStatic + Default + 'static,
    {
        Self::from_static::<A, _>(|| A::default())
    }
}

/// Tiny helper so you can register with less ceremony.
#[macro_export]
macro_rules! simple_action_factory {
    ($ty:ty) => {{ $crate::ActionFactory::default_of::<$ty>() }};
    ($id:expr, $ty:ty) => {{ $crate::ActionFactory::new($id, || <$ty>::default()) }};
}
