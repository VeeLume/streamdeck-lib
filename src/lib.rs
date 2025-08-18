// lib.rs

mod logger;
mod sd_protocol;
mod context;
mod actions;
mod action_manager;
mod runtime;
mod plugin_builder;
mod adapters;
mod hooks;
mod launch;
mod events;
mod adapters_manager;
mod bus;
pub mod input; // maybe this one stays public if it has submodules users need

// Public surface (root-level re-exports)
pub use crate::logger::{ ActionLog, ActionLogExt, FileLogger, Level, LoggerConfig };
pub use crate::sd_protocol::{
    SdState,
    Size,
    Coordinates,
    DeviceInfo,
    TitleParameters,
    StreamDeckEvent,
    Target,
    SetTitlePayload,
    SetImagePayload,
    TriggerPayload,
    SdClient,
};
pub use crate::hooks::{ AppHooks, HookEvent, HookFn };
pub use crate::events::{ AdapterTarget, ActionTarget, AdapterControl, ErasedTopic, TopicId };
pub use crate::context::{ Context, Extensions, GlobalSettings };
pub use crate::bus::{ Bus, BusTyped };
pub use crate::launch::{ LaunchArgs, RunConfig, LaunchArgError, parse_launch_args, parse_from };
pub use crate::adapters::{ Adapter, AdapterStatic, StartPolicy, AdapterHandle, AdapterError, AdapterResult };
pub use crate::actions::{ Action, ActionStatic, ActionId, ActionFactory };
pub use crate::plugin_builder::{ PluginBuilder, BuildError };
pub use crate::input::dsl::{ sleep_ms, sleep, tap, down, up, chord, hold, tap_with_delay, click, click_n };
pub use crate::input::key::Key;
pub use crate::input::{InputSynth, Executor};
pub use crate::input::types::{InputStep, MouseButton, Scan};
pub use crate::runtime::run;

// Prelude stays minimal and user-friendly
pub mod prelude {
    pub use crate::{ debug, info, warn, error, simple_action_factory };
    pub use crate::logger::{ ActionLog, FileLogger, Level, LoggerConfig };
    pub use crate::sd_protocol::{ SdClient, StreamDeckEvent, Target, SdState, views::* };
    pub use crate::hooks::{ AppHooks, HookEvent };
    pub use crate::events::{ TopicId, ErasedTopic };
    pub use crate::context::{ Context, Extensions, GlobalSettings };
    pub use crate::bus::{ Bus, BusTyped };
    pub use crate::launch::{ RunConfig, LaunchArgError, parse_launch_args };
    pub use crate::adapters::{ Adapter, AdapterStatic, StartPolicy, AdapterHandle, AdapterError, AdapterResult };
    pub use crate::actions::{ Action, ActionStatic, ActionFactory };
    pub use crate::plugin_builder::{ PluginBuilder, BuildError };
    pub use crate::input::dsl::{ sleep_ms, sleep, tap, down, up, chord, hold, tap_with_delay, click, click_n };
    pub use crate::input::key::Key;
    pub use crate::input::{InputSynth};
    pub use crate::input::types::{InputStep, MouseButton, Scan};
    pub use crate::runtime::run;
}
