// launch.rs
use std::{env, ffi::OsString, fmt};

/// Values passed by Stream Deck on launch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchArgs {
    pub port: u16,
    pub plugin_uuid: String,
    pub register_event: String,
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

/// Build the websocket URL. Honors optional env overrides for tests/tools:
/// - SD_WS_SCHEME (default: "ws")
/// - SD_WS_HOST   (default: "127.0.0.1")
pub fn ws_url(port: u16) -> String {
    let scheme = env::var("SD_WS_SCHEME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "ws".into());
    let host = env::var("SD_WS_HOST")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "127.0.0.1".into());
    format!("{scheme}://{host}:{port}")
}

/// Batteries-included entrypoint for binaries:
/// - parses args
/// - calls the runtime with defaults (URL + log_ws from env)
///
/// Usage in your `main`:
/// ```no_run
/// let _guard = your_crate::telemetry::init("your_plugin_id");
/// your_crate::launch::run_plugin(build_plugin())?;
/// ```
pub fn run_plugin(plugin: crate::plugin::Plugin) -> anyhow::Result<()> {
    let args = parse_launch_args()?;
    crate::runtime::run_with_defaults(plugin, args)
}
