// hooks.rs
use crate::{
    context::Context,
    events::{AdapterControl, AdapterTarget, ErasedTopic},
    logger::Level,
    sd_protocol::{DeviceInfo, Outgoing, StreamDeckEvent},
};
use std::sync::Arc;

/// Everything that can be observed.
#[non_exhaustive]
#[derive(Debug)]
pub enum HookEvent<'a> {
    // SD lifecycle/raw
    Incoming(&'a StreamDeckEvent),
    ApplicationDidLaunch(&'a str),
    ApplicationDidTerminate(&'a str),
    DeviceDidConnect(&'a str, &'a DeviceInfo),
    DeviceDidDisconnect(&'a str),
    DeviceDidChange(&'a str, &'a DeviceInfo),
    DidReceiveDeepLink(&'a str),
    DidReceiveGlobalSettings(&'a serde_json::Map<String, serde_json::Value>),

    // Runtime mirrors
    Outgoing(&'a Outgoing),
    Log(Level, &'a str),
    ActionNotify(&'a ErasedTopic),
    AdapterNotify(&'a AdapterTarget, &'a ErasedTopic),
    AdapterControl(&'a AdapterControl),

    // Lifecycle
    Init,
    Exit,
    Tick,
}

pub type HookFn = dyn for<'a> Fn(&'a Context, &'a HookEvent<'a>) + Send + Sync;

/// A tiny bus of closures.
#[derive(Clone, Default)]
pub struct AppHooks {
    listeners: Vec<Arc<HookFn>>,
}

impl AppHooks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append<F>(mut self, f: F) -> Self
    where
        F: for<'a> Fn(&'a Context, &'a HookEvent<'a>) + Send + Sync + 'static,
    {
        self.listeners.push(Arc::new(f));
        self
    }

    pub fn push<F>(&mut self, f: F)
    where
        F: for<'a> Fn(&'a Context, &'a HookEvent<'a>) + Send + Sync + 'static,
    {
        self.listeners.push(Arc::new(f));
    }

    #[inline]
    pub fn fire(&self, cx: &Context, ev: &HookEvent) {
        for l in &self.listeners {
            l(cx, ev);
        }
    }

    // Optional: small sugar if you like names
    #[inline]
    pub fn fire_incoming(&self, cx: &Context, e: &StreamDeckEvent) {
        self.fire(cx, &HookEvent::Incoming(e));
    }
    #[inline]
    pub fn fire_outgoing(&self, cx: &Context, m: &Outgoing) {
        self.fire(cx, &HookEvent::Outgoing(m));
    }
    #[inline]
    pub fn fire_log(&self, cx: &Context, lvl: Level, msg: &str) {
        self.fire(cx, &HookEvent::Log(lvl, msg));
    }
    #[inline]
    pub fn fire_action_notify(&self, cx: &Context, ev: &ErasedTopic) {
        self.fire(cx, &HookEvent::ActionNotify(ev));
    }
    #[inline]
    pub fn fire_adapter_notify(&self, cx: &Context, t: &AdapterTarget, ev: &ErasedTopic) {
        self.fire(cx, &HookEvent::AdapterNotify(t, ev));
    }
    #[inline]
    pub fn fire_adapter_control(&self, cx: &Context, ctl: &AdapterControl) {
        self.fire(cx, &HookEvent::AdapterControl(ctl));
    }
    #[inline]
    pub fn fire_init(&self, cx: &Context) {
        self.fire(cx, &HookEvent::Init);
    }
    #[inline]
    pub fn fire_exit(&self, cx: &Context) {
        self.fire(cx, &HookEvent::Exit);
    }
    #[inline]
    pub fn fire_tick(&self, cx: &Context) {
        self.fire(cx, &HookEvent::Tick);
    }

    // If you already emit these elsewhere in the main loop, keep the sugar:
    #[inline]
    pub fn fire_application_did_launch(&self, cx: &Context, app: &str) {
        self.fire(cx, &HookEvent::ApplicationDidLaunch(app));
    }
    #[inline]
    pub fn fire_application_did_terminate(&self, cx: &Context, app: &str) {
        self.fire(cx, &HookEvent::ApplicationDidTerminate(app));
    }
    #[inline]
    pub fn fire_device_did_connect(&self, cx: &Context, dev: &str, info: &DeviceInfo) {
        self.fire(cx, &HookEvent::DeviceDidConnect(dev, info));
    }
    #[inline]
    pub fn fire_device_did_disconnect(&self, cx: &Context, dev: &str) {
        self.fire(cx, &HookEvent::DeviceDidDisconnect(dev));
    }
    #[inline]
    pub fn fire_device_did_change(&self, cx: &Context, dev: &str, info: &DeviceInfo) {
        self.fire(cx, &HookEvent::DeviceDidChange(dev, info));
    }
    #[inline]
    pub fn fire_did_receive_deep_link(&self, cx: &Context, url: &str) {
        self.fire(cx, &HookEvent::DidReceiveDeepLink(url));
    }
    #[inline]
    pub fn fire_did_receive_global_settings(
        &self,
        cx: &Context,
        gs: &serde_json::Map<String, serde_json::Value>,
    ) {
        self.fire(cx, &HookEvent::DidReceiveGlobalSettings(gs));
    }
}
