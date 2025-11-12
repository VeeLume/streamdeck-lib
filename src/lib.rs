// lib.rs

mod action_manager;
mod actions;
mod adapters;
mod adapters_manager;
mod bus;
mod context;
mod events;
mod hooks;
pub mod input;
mod launch;
mod logger;
mod plugin;
mod runtime;
mod sd_protocol; // maybe this one stays public if it has submodules users need

// Public surface (root-level re-exports)
pub use crate::actions::{Action, ActionFactory, ActionId, ActionStatic};
pub use crate::adapters::{
    Adapter, AdapterError, AdapterHandle, AdapterResult, AdapterStatic, StartPolicy,
};
pub use crate::bus::{Bus, BusTyped};
pub use crate::context::{Context, Extensions, GlobalSettings};
pub use crate::events::{ActionTarget, AdapterControl, AdapterTarget, ErasedTopic, TopicId};
pub use crate::hooks::{AppHooks, HookEvent, HookFn};
pub use crate::input::dsl::{
    chord, click, click_n, down, hold, sleep, sleep_ms, tap, tap_with_delay, up,
};
pub use crate::input::key::Key;
pub use crate::input::types::{InputStep, MouseButton, Scan};
pub use crate::input::{Executor, InputSynth};
pub use crate::launch::run_plugin;
pub use crate::launch::{LaunchArgError, LaunchArgs, parse_from, parse_launch_args};
pub use crate::logger::{init, init_with};
pub use crate::plugin::Plugin;
pub use crate::runtime::run_with_defaults;
pub use crate::sd_protocol::{
    Coordinates, DeviceInfo, SdClient, SdState, SetImagePayload, SetTitlePayload, Size,
    StreamDeckEvent, Target, TitleParameters, TriggerPayload,
};

pub mod prelude {
    pub use crate::actions::{Action, ActionFactory, ActionStatic};
    pub use crate::adapters::{
        Adapter, AdapterError, AdapterHandle, AdapterResult, AdapterStatic, StartPolicy,
    };
    pub use crate::bus::{Bus, BusTyped};
    pub use crate::context::{Context, Extensions, GlobalSettings};
    pub use crate::events::{ErasedTopic, TopicId};
    pub use crate::hooks::{AppHooks, HookEvent};
    pub use crate::input::InputSynth;
    pub use crate::input::dsl::{
        chord, click, click_n, down, hold, sleep, sleep_ms, tap, tap_with_delay, up,
    };
    pub use crate::input::key::Key;
    pub use crate::input::types::{InputStep, MouseButton, Scan};
    pub use crate::launch::run_plugin;
    pub use crate::launch::{LaunchArgError, parse_launch_args};
    pub use crate::logger::{init, init_with};
    pub use crate::plugin::Plugin;
    pub use crate::runtime::run_with_defaults;
    pub use crate::sd_protocol::{SdClient, SdState, StreamDeckEvent, Target, views::*};
    pub use crate::simple_action_factory;
}
