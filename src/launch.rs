// launch.rs
use std::{env, ffi::OsString, fmt};

/// Values passed by Stream Deck on launch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchArgs {
    pub port: u16,
    pub plugin_uuid: String,
    pub register_event: String,
}

/// Plugin runtime configuration knobs.
#[derive(Clone)]
pub struct RunConfig {
    /// Optional custom websocket URL builder (useful for tests).
    pub url_fn: std::sync::Arc<dyn (Fn(u16) -> String) + Send + Sync>,
    pub log_websocket: bool,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            url_fn: std::sync::Arc::new(|port| format!("ws://127.0.0.1:{port}")),
            log_websocket: false,
        }
    }
}

impl fmt::Debug for RunConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RunConfig")
            .field("log_websocket", &self.log_websocket)
            .finish_non_exhaustive()
    }
}

impl RunConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom websocket URL builder function.
    pub fn with_url_fn<F>(mut self, url_fn: F) -> Self
    where
        F: Fn(u16) -> String + Send + Sync + 'static,
    {
        self.url_fn = std::sync::Arc::new(url_fn);
        self
    }

    /// Set a custom URL builder function.
    pub fn ws_url(&self, port: u16) -> String {
        (self.url_fn)(port)
    }

    /// Enable or disable logging of incoming websocket messages.
    pub fn set_log_websocket(mut self, enable: bool) -> Self {
        self.log_websocket = enable;
        self
    }
}

/// Errors when parsing Stream Deck launch flags.
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub enum LaunchArgError {
    MissingPort,
    MissingPluginUUID,
    MissingRegisterEvent,
    InvalidPort(String),
}

impl fmt::Display for LaunchArgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LaunchArgError::*;
        match self {
            MissingPort => write!(f, "missing -port"),
            MissingPluginUUID => write!(f, "missing -pluginUUID"),
            MissingRegisterEvent => write!(f, "missing -registerEvent"),
            InvalidPort(v) => write!(f, "invalid port '{v}'"),
        }
    }
}

impl std::error::Error for LaunchArgError {}

/// Parse from the current process argv (skips argv[0]).
pub fn parse_launch_args() -> Result<LaunchArgs, LaunchArgError> {
    parse_from(env::args_os().skip(1))
}

/// Parse from any iterator of OsString (nice for unit tests).
pub fn parse_from<I>(it: I) -> Result<LaunchArgs, LaunchArgError>
where
    I: Iterator<Item = OsString>,
{
    // Collect into Strings once (Stream Deck doesn’t pass weird encodings).
    let args: Vec<String> = it.map(|s| s.to_string_lossy().into_owned()).collect();

    let port_str = value_after(&args, "-port").ok_or(LaunchArgError::MissingPort)?;
    let plugin_uuid = value_after(&args, "-pluginUUID").ok_or(LaunchArgError::MissingPluginUUID)?;
    let register_event =
        value_after(&args, "-registerEvent").ok_or(LaunchArgError::MissingRegisterEvent)?;

    let port = port_str
        .parse::<u16>()
        .map_err(|_| LaunchArgError::InvalidPort(port_str.to_string()))?;

    Ok(LaunchArgs {
        port,
        plugin_uuid: plugin_uuid.to_string(),
        register_event: register_event.to_string(),
    })
}

fn value_after<'a>(argv: &'a [String], flag: &str) -> Option<&'a str> {
    argv.iter()
        .position(|a| a == flag)
        .and_then(|i| argv.get(i + 1))
        .map(|s| s.as_str())
}
