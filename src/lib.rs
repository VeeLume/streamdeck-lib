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
mod plugin_builder;
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
pub use crate::launch::{LaunchArgError, LaunchArgs, RunConfig, parse_from, parse_launch_args};
pub use crate::logger::{ActionLog, ActionLogExt, FileLogger, Level, LoggerConfig};
pub use crate::plugin_builder::{BuildError, PluginBuilder};
pub use crate::runtime::run;
pub use crate::sd_protocol::{
    Coordinates, DeviceInfo, SdClient, SdState, SetImagePayload, SetTitlePayload, Size,
    StreamDeckEvent, Target, TitleParameters, TriggerPayload,
};

// Prelude stays minimal and user-friendly
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
    pub use crate::launch::{LaunchArgError, RunConfig, parse_launch_args};
    pub use crate::logger::{ActionLog, FileLogger, Level, LoggerConfig};
    pub use crate::plugin_builder::{BuildError, PluginBuilder};
    pub use crate::runtime::run;
    pub use crate::sd_protocol::{SdClient, SdState, StreamDeckEvent, Target, views::*};
    pub use crate::{debug, error, info, simple_action_factory, warn};
}
