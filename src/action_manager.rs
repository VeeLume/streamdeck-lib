// action_manager.rs
use std::{collections::HashMap, sync::Arc};

use crate::{
    actions::{Action, ActionFactory, ActionId},
    context::Context,
    events::{ActionTarget, ErasedTopic},
    plugin::Plugin,
    sd_protocol::{StreamDeckEvent, views},
};

pub(crate) struct ActionManager {
    regs: HashMap<ActionId, ActionFactory>,
    instances: HashMap<(ActionId, String), Box<dyn Action>>,
    by_topic: HashMap<&'static str, Vec<(ActionId, String)>>, // topic -> [(action_id, ctx_id)]
}

impl ActionManager {
    pub(crate) fn new(regs: HashMap<ActionId, ActionFactory>) -> Self {
        Self {
            regs,
            instances: HashMap::new(),
            by_topic: HashMap::new(),
        }
    }

    #[inline]
    fn key(action_id: &str, ctx_id: &str) -> (ActionId, String) {
        (action_id.to_string(), ctx_id.to_string())
    }

    /// Ensure an instance exists and is **ready**:
    /// - constructs if missing
    /// - calls `init` exactly once
    /// - captures `topics()` and indexes for ActionTarget::Topic
    fn ensure_ready(
        &mut self,
        cx: &Context,
        action_id: &str,
        ctx_id: &str,
    ) -> Option<&mut Box<dyn Action>> {
        let key = Self::key(action_id, ctx_id);
        if !self.instances.contains_key(&key) {
            let reg = self.regs.get(action_id)?;
            let mut inst = (reg.build)();

            // capture topics before moving into the map
            let topics = inst.topics();

            // run init once
            inst.init(cx, ctx_id);

            // store the instance
            self.instances.insert(key.clone(), inst);

            // index topics for fan-out
            for &t in topics {
                self.by_topic.entry(t).or_default().push(key.clone());
            }
        }
        self.instances.get_mut(&key)
    }

    /// Create an instance **only so we can tear it down**.
    /// Does *not* call `init` and does *not* index topics.
    fn get_or_make_for_teardown(
        &mut self,
        action_id: &str,
        ctx_id: &str,
    ) -> Option<&mut Box<dyn Action>> {
        let key = Self::key(action_id, ctx_id);
        if !self.instances.contains_key(&key) {
            let reg = self.regs.get(action_id)?;
            self.instances.insert(key.clone(), (reg.build)());
        }
        self.instances.get_mut(&key)
    }

    /// Remove an instance (calling `teardown` first) and de-index its topics.
    fn remove(&mut self, cx: &Context, action_id: &str, ctx_id: &str) {
        let key = Self::key(action_id, ctx_id);
        if let Some(mut inst) = self.instances.remove(&key) {
            // de-index topics using the instance we just removed
            for &t in inst.topics() {
                if let Some(list) = self.by_topic.get_mut(t) {
                    list.retain(|k| k != &key);
                    if list.is_empty() {
                        self.by_topic.remove(t);
                    }
                }
            }
            inst.teardown(cx, ctx_id);
        }
    }

    pub(crate) fn notify_topic(&mut self, cx: &Context, topic_name: &str, event: Arc<ErasedTopic>) {
        if let Some(keys) = self.by_topic.get(topic_name) {
            for (aid, ctx) in keys.clone() {
                if let Some(a) = self.instances.get_mut(&(aid.clone(), ctx.clone())) {
                    a.on_notify(cx, &ctx, event.as_ref());
                }
            }
        }
    }

    /// Unified target-based notify (mirrors RuntimeMsg::ActionNotify).
    pub fn notify_target(&mut self, cx: &Context, target: ActionTarget, event: Arc<ErasedTopic>) {
        match target {
            ActionTarget::All => self.notify_all(cx, Arc::clone(&event)),
            ActionTarget::Context(ctx) => self.notify_context(cx, &ctx, Arc::clone(&event)),
            ActionTarget::Id(action_id) => {
                for ((aid, ctx), a) in self.instances.iter_mut() {
                    if aid == action_id {
                        a.on_notify(cx, ctx, event.as_ref());
                    }
                }
            }
        }
    }

    /// Broadcast a typed notify to all live instances.
    pub(crate) fn notify_all(&mut self, cx: &Context, event: Arc<ErasedTopic>) {
        for ((_, ctx_id), a) in self.instances.iter_mut() {
            a.on_notify(cx, ctx_id, event.as_ref());
        }
    }

    /// Notify a single context (if present).
    pub(crate) fn notify_context(&mut self, cx: &Context, ctx_id: &str, event: Arc<ErasedTopic>) {
        if let Some((_, a)) = self.instances.iter_mut().find(|((_, c), _)| c == ctx_id) {
            a.on_notify(cx, ctx_id, event.as_ref());
        }
    }
}

pub(crate) fn dispatch(
    mgr: &mut ActionManager,
    cx: &Context,
    _plugin: &Plugin,
    ev: StreamDeckEvent,
) {
    use StreamDeckEvent::*;

    match &ev {
        WillAppear {
            action,
            context,
            device,
            settings,
            controller,
            is_in_multi_action,
            state,
            coordinates,
        } => {
            let v = views::WillAppear {
                action,
                context,
                device,
                settings,
                controller,
                is_in_multi_action,
                state,
                coordinates,
            };
            if let Some(a) = mgr.ensure_ready(cx, action, context) {
                a.will_appear(cx, &v);
            }
        }

        WillDisappear {
            action,
            context,
            device,
            settings,
            controller,
            is_in_multi_action,
            state,
            coordinates,
        } => {
            let v = views::WillDisappear {
                action,
                context,
                device,
                settings,
                controller,
                is_in_multi_action,
                state,
                coordinates,
            };
            if let Some(a) = mgr.get_or_make_for_teardown(action, context) {
                a.will_disappear(cx, &v);
            }
            mgr.remove(cx, action, context);
        }

        KeyDown {
            action,
            context,
            device,
            settings,
            controller,
            is_in_multi_action,
            state,
            coordinates,
        } => {
            let v = views::KeyDown {
                action,
                context,
                device,
                settings,
                controller,
                is_in_multi_action,
                state,
                coordinates,
            };
            if let Some(a) = mgr.ensure_ready(cx, action, context) {
                a.key_down(cx, &v);
            }
        }

        KeyUp {
            action,
            context,
            device,
            settings,
            controller,
            is_in_multi_action,
            state,
            coordinates,
        } => {
            let v = views::KeyUp {
                action,
                context,
                device,
                settings,
                controller,
                is_in_multi_action,
                state,
                coordinates,
            };
            if let Some(a) = mgr.ensure_ready(cx, action, context) {
                a.key_up(cx, &v);
            }
        }

        DialDown {
            action,
            context,
            device,
            settings,
            controller,
            coordinates,
        } => {
            let v = views::DialDown {
                action,
                context,
                device,
                settings,
                controller,
                coordinates,
            };
            if let Some(a) = mgr.ensure_ready(cx, action, context) {
                a.dial_down(cx, &v);
            }
        }

        DialUp {
            action,
            context,
            device,
            settings,
            controller,
            coordinates,
        } => {
            let v = views::DialUp {
                action,
                context,
                device,
                settings,
                controller,
                coordinates,
            };
            if let Some(a) = mgr.ensure_ready(cx, action, context) {
                a.dial_up(cx, &v);
            }
        }

        DialRotate {
            action,
            context,
            device,
            settings,
            controller,
            coordinates,
            pressed,
            ticks,
        } => {
            let v = views::DialRotate {
                action,
                context,
                device,
                settings,
                controller,
                coordinates,
                pressed,
                ticks,
            };
            if let Some(a) = mgr.ensure_ready(cx, action, context) {
                a.dial_rotate(cx, &v);
            }
        }

        TouchTap {
            action,
            context,
            device,
            settings,
            controller,
            coordinates,
            hold,
            tap_pos,
        } => {
            let v = views::TouchTap {
                action,
                context,
                device,
                settings,
                controller,
                coordinates,
                hold,
                tap_pos,
            };
            if let Some(a) = mgr.ensure_ready(cx, action, context) {
                a.touch_tap(cx, &v);
            }
        }

        TitleParametersDidChange {
            action,
            context,
            device,
            settings,
            controller,
            coordinates,
            state,
            title,
            title_parameters,
        } => {
            let v = views::TitleParametersDidChange {
                action,
                context,
                device,
                settings,
                controller,
                coordinates,
                state,
                title,
                title_parameters,
            };
            if let Some(a) = mgr.ensure_ready(cx, action, context) {
                a.title_parameters_did_change(cx, &v);
            }
        }

        PropertyInspectorDidAppear {
            action,
            context,
            device,
        } => {
            let v = views::PropertyInspectorDidAppear {
                action,
                context,
                device,
            };
            if let Some(a) = mgr.ensure_ready(cx, action, context) {
                a.property_inspector_did_appear(cx, &v);
            }
        }

        PropertyInspectorDidDisappear {
            action,
            context,
            device,
        } => {
            let v = views::PropertyInspectorDidDisappear {
                action,
                context,
                device,
            };
            if let Some(a) = mgr.ensure_ready(cx, action, context) {
                a.property_inspector_did_disappear(cx, &v);
            }
        }

        DidReceiveSettings {
            action,
            context,
            device,
            settings,
            controller,
            is_in_multi_action,
            state,
            coordinates,
        } => {
            let v = views::DidReceiveSettings {
                action,
                context,
                device,
                settings,
                controller,
                is_in_multi_action,
                state,
                coordinates,
            };
            if let Some(a) = mgr.ensure_ready(cx, action, context) {
                a.did_receive_settings(cx, &v);
            }
        }

        DidReceivePropertyInspectorMessage {
            action,
            context,
            payload,
        } => {
            let v = views::DidReceivePropertyInspectorMessage {
                action,
                context,
                payload,
            };
            if let Some(a) = mgr.ensure_ready(cx, action, context) {
                a.did_receive_property_inspector_message(cx, &v);
            }
        }

        _ => {
            for (_, a) in mgr.instances.iter_mut() {
                a.on_global_event(cx, &ev);
            }
        }
    }
}
