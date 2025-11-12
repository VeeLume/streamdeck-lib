// sd_protocol.rs
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::convert::TryFrom;
use tracing::trace;

// =========================
// Incoming: shared structs
// =========================

#[repr(u8)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    IntoPrimitive,
    TryFromPrimitive,
    Serialize_repr,
    Deserialize_repr,
)]
pub enum SdState {
    Primary = 0,
    Secondary = 1,
}

impl SdState {
    pub fn from_json(v: &serde_json::Value) -> Option<Self> {
        v.as_u64()
            .and_then(|n| u8::try_from(n).ok())
            .and_then(|b| SdState::try_from(b).ok())
    }
    pub fn as_u8(self) -> u8 {
        self.into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub columns: i64,
    pub rows: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct Coordinates {
    pub column: i64,
    pub row: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: i64,
    pub size: Size,
}

#[derive(Debug, Clone)]
pub struct TitleParameters {
    pub font_family: String,
    pub font_size: i64,
    pub font_style: String,
    pub font_underline: bool,
    pub show_title: bool,
    pub title_alignment: String,
    pub title_color: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TitleParametersWire {
    font_family: String,
    font_size: i64,
    font_style: String,
    font_underline: bool,
    show_title: bool,
    title_alignment: String,
    title_color: String,
}
impl From<TitleParametersWire> for TitleParameters {
    fn from(w: TitleParametersWire) -> Self {
        Self {
            font_family: w.font_family,
            font_size: w.font_size,
            font_style: w.font_style,
            font_underline: w.font_underline,
            show_title: w.show_title,
            title_alignment: w.title_alignment,
            title_color: w.title_color,
        }
    }
}

// =========================
// Incoming: event enum
// =========================

#[derive(Debug, Clone)]
pub enum StreamDeckEvent {
    ApplicationDidLaunch {
        application: String,
    },
    ApplicationDidTerminate {
        application: String,
    },
    DeviceDidChange {
        device: String,
        device_info: DeviceInfo,
    },
    DeviceDidConnect {
        device: String,
        device_info: DeviceInfo,
    },
    DeviceDidDisconnect {
        device: String,
    },
    DialDown {
        action: String,
        context: String,
        device: String,
        settings: Map<String, Value>,
        controller: String,
        coordinates: Coordinates,
    },
    DialRotate {
        action: String,
        context: String,
        device: String,
        settings: Map<String, Value>,
        controller: String,
        coordinates: Coordinates,
        pressed: bool,
        ticks: i64,
    },
    DialUp {
        action: String,
        context: String,
        device: String,
        settings: Map<String, Value>,
        controller: String,
        coordinates: Coordinates,
    },
    DidReceiveDeepLink {
        url: String,
    },
    DidReceiveGlobalSettings {
        settings: Map<String, Value>,
    },
    DidReceivePropertyInspectorMessage {
        action: String,
        context: String,
        payload: Map<String, Value>,
    },
    DidReceiveSettings {
        action: String,
        context: String,
        device: String,
        settings: Map<String, Value>,
        controller: String,
        is_in_multi_action: bool,
        state: Option<SdState>,
        coordinates: Option<Coordinates>,
    },
    KeyDown {
        action: String,
        context: String,
        device: String,
        settings: Map<String, Value>,
        controller: String,
        is_in_multi_action: bool,
        state: Option<SdState>,
        coordinates: Option<Coordinates>,
    },
    KeyUp {
        action: String,
        context: String,
        device: String,
        settings: Map<String, Value>,
        controller: String,
        is_in_multi_action: bool,
        state: Option<SdState>,
        coordinates: Option<Coordinates>,
    },
    PropertyInspectorDidAppear {
        action: String,
        context: String,
        device: String,
    },
    PropertyInspectorDidDisappear {
        action: String,
        context: String,
        device: String,
    },
    SystemDidWakeUp,
    TitleParametersDidChange {
        action: String,
        context: String,
        device: String,
        settings: Map<String, Value>,
        controller: String,
        coordinates: Coordinates,
        state: Option<SdState>,
        title: String,
        title_parameters: TitleParameters,
    },
    TouchTap {
        action: String,
        context: String,
        device: String,
        settings: Map<String, Value>,
        controller: String,
        coordinates: Coordinates,
        hold: bool,
        tap_pos: (i64, i64),
    },
    WillAppear {
        action: String,
        context: String,
        device: String,
        settings: Map<String, Value>,
        controller: String,
        is_in_multi_action: bool,
        state: Option<SdState>,
        coordinates: Option<Coordinates>,
    },
    WillDisappear {
        action: String,
        context: String,
        device: String,
        settings: Map<String, Value>,
        controller: String,
        is_in_multi_action: bool,
        state: Option<SdState>,
        coordinates: Option<Coordinates>,
    },
}

pub mod views {
    use super::{Coordinates, SdState, TitleParameters};
    use serde_json::{Map, Value};

    pub struct WillAppear<'a> {
        pub action: &'a str,
        pub context: &'a str,
        pub device: &'a str,
        pub settings: &'a Map<String, Value>,
        pub controller: &'a str,
        pub is_in_multi_action: &'a bool,
        pub state: &'a Option<SdState>,
        pub coordinates: &'a Option<Coordinates>,
    }

    pub struct WillDisappear<'a> {
        pub action: &'a str,
        pub context: &'a str,
        pub device: &'a str,
        pub settings: &'a Map<String, Value>,
        pub controller: &'a str,
        pub is_in_multi_action: &'a bool,
        pub state: &'a Option<SdState>,
        pub coordinates: &'a Option<Coordinates>,
    }

    pub struct KeyDown<'a> {
        pub action: &'a str,
        pub context: &'a str,
        pub device: &'a str,
        pub settings: &'a Map<String, Value>,
        pub controller: &'a str,
        pub is_in_multi_action: &'a bool,
        pub state: &'a Option<SdState>,
        pub coordinates: &'a Option<Coordinates>,
    }
    pub struct KeyUp<'a> {
        pub action: &'a str,
        pub context: &'a str,
        pub device: &'a str,
        pub settings: &'a Map<String, Value>,
        pub controller: &'a str,
        pub is_in_multi_action: &'a bool,
        pub state: &'a Option<SdState>,
        pub coordinates: &'a Option<Coordinates>,
    }

    pub struct DialDown<'a> {
        pub action: &'a str,
        pub context: &'a str,
        pub device: &'a str,
        pub settings: &'a Map<String, Value>,
        pub controller: &'a str,
        pub coordinates: &'a Coordinates,
    }
    pub struct DialUp<'a> {
        pub action: &'a str,
        pub context: &'a str,
        pub device: &'a str,
        pub settings: &'a Map<String, Value>,
        pub controller: &'a str,
        pub coordinates: &'a Coordinates,
    }
    pub struct DialRotate<'a> {
        pub action: &'a str,
        pub context: &'a str,
        pub device: &'a str,
        pub settings: &'a Map<String, Value>,
        pub controller: &'a str,
        pub coordinates: &'a Coordinates,
        pub pressed: &'a bool,
        pub ticks: &'a i64,
    }

    pub struct TouchTap<'a> {
        pub action: &'a str,
        pub context: &'a str,
        pub device: &'a str,
        pub settings: &'a Map<String, Value>,
        pub controller: &'a str,
        pub coordinates: &'a Coordinates,
        pub hold: &'a bool,
        pub tap_pos: &'a (i64, i64),
    }

    pub struct TitleParametersDidChange<'a> {
        pub action: &'a str,
        pub context: &'a str,
        pub device: &'a str,
        pub settings: &'a Map<String, Value>,
        pub controller: &'a str,
        pub coordinates: &'a Coordinates,
        pub state: &'a Option<SdState>,
        pub title: &'a str,
        pub title_parameters: &'a TitleParameters,
    }

    pub struct PropertyInspectorDidAppear<'a> {
        pub action: &'a str,
        pub context: &'a str,
        pub device: &'a str,
    }
    pub struct PropertyInspectorDidDisappear<'a> {
        pub action: &'a str,
        pub context: &'a str,
        pub device: &'a str,
    }

    pub struct DidReceiveSettings<'a> {
        pub action: &'a str,
        pub context: &'a str,
        pub device: &'a str,
        pub settings: &'a Map<String, Value>,
        pub controller: &'a str,
        pub is_in_multi_action: &'a bool,
        pub state: &'a Option<SdState>,
        pub coordinates: &'a Option<Coordinates>,
    }

    pub struct DidReceivePropertyInspectorMessage<'a> {
        pub action: &'a str,
        pub context: &'a str,
        pub payload: &'a Map<String, Value>,
    }
}

// Global

impl std::fmt::Display for StreamDeckEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use StreamDeckEvent::*;
        match self {
            ApplicationDidLaunch { .. } => write!(f, "ApplicationDidLaunch"),
            ApplicationDidTerminate { .. } => write!(f, "ApplicationDidTerminate"),
            DeviceDidChange { .. } => write!(f, "DeviceDidChange"),
            DeviceDidConnect { .. } => write!(f, "DeviceDidConnect"),
            DeviceDidDisconnect { .. } => write!(f, "DeviceDidDisconnect"),
            DialDown {
                action, context, ..
            } => write!(f, "DialDown(action={action}, context={context})"),
            DialRotate {
                action, context, ..
            } => write!(f, "DialRotate(action={action}, context={context})"),
            DialUp {
                action, context, ..
            } => write!(f, "DialUp(action={action}, context={context})"),
            DidReceiveDeepLink { .. } => write!(f, "DidReceiveDeepLink"),
            DidReceiveGlobalSettings { .. } => write!(f, "DidReceiveGlobalSettings"),
            DidReceivePropertyInspectorMessage {
                action, context, ..
            } => write!(
                f,
                "DidReceivePropertyInspectorMessage(action={action}, context={context})"
            ),
            DidReceiveSettings {
                action, context, ..
            } => write!(f, "DidReceiveSettings(action={action}, context={context})"),
            KeyDown {
                action, context, ..
            } => write!(f, "KeyDown(action={action}, context={context})"),
            KeyUp {
                action, context, ..
            } => write!(f, "KeyUp(action={action}, context={context})"),
            PropertyInspectorDidAppear {
                action, context, ..
            } => write!(
                f,
                "PropertyInspectorDidAppear(action={action}, context={context})"
            ),
            PropertyInspectorDidDisappear {
                action, context, ..
            } => write!(
                f,
                "PropertyInspectorDidDisappear(action={action}, context={context})"
            ),
            SystemDidWakeUp => write!(f, "SystemDidWakeUp"),
            TitleParametersDidChange {
                action, context, ..
            } => write!(
                f,
                "TitleParametersDidChange(action={action}, context={context})"
            ),
            TouchTap {
                action, context, ..
            } => write!(f, "TouchTap(action={action}, context={context})"),
            WillAppear {
                action, context, ..
            } => write!(f, "WillAppear(action={action}, context={context})"),
            WillDisappear {
                action, context, ..
            } => write!(f, "WillDisappear(action={action}, context={context})"),
        }
    }
}

// =========================
// Incoming: parse entrypoint
// =========================

#[inline]
pub fn parse_incoming_owned(mut m: Map<String, Value>) -> Result<StreamDeckEvent, String> {
    use StreamDeckEvent::*;

    // Small helper to get &str (still cheap clones for the strings themselves).
    #[inline]
    fn must_str<'a>(m: &'a Map<String, Value>, key: &str) -> Result<&'a str, String> {
        m.get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("missing {key}"))
    }

    // Pull top-level fields (strings are cheap to clone).
    let event = must_str(&m, "event")?.to_string();
    let action = m.get("action").and_then(Value::as_str).map(str::to_string);
    let context = m.get("context").and_then(Value::as_str).map(str::to_string);
    let device = m.get("device").and_then(Value::as_str).map(str::to_string);

    // Mutable access to payload so we can move things out without cloning.
    let mut payload = m.remove("payload"); // Option<Value>

    // Move out settings object (no clone).
    let settings: Map<String, Value> = match payload
        .as_mut()
        .and_then(Value::as_object_mut)
        .and_then(|p| p.remove("settings"))
    {
        Some(Value::Object(obj)) => obj,
        _ => Map::new(),
    };

    let controller_opt = payload
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|p| p.get("controller").and_then(Value::as_str))
        .map(str::to_string);

    let coordinates = payload
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|p| p.get("coordinates"))
        .and_then(|v| {
            let o = v.as_object()?;
            Some(crate::sd_protocol::Coordinates {
                column: o.get("column")?.as_i64()?,
                row: o.get("row")?.as_i64()?,
            })
        });

    let is_in_multi_action = payload
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|p| p.get("isInMultiAction").and_then(Value::as_bool))
        .unwrap_or(false);

    let state = payload
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|p| p.get("state"))
        .and_then(crate::sd_protocol::SdState::from_json);

    let title = payload
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|p| p.get("title").and_then(Value::as_str))
        .map(str::to_string);

    // Move out titleParameters (no clone); then deserialize.
    let title_parameters = payload
        .as_mut()
        .and_then(Value::as_object_mut)
        .and_then(|p| p.remove("titleParameters"))
        .map(serde_json::from_value::<crate::sd_protocol::TitleParametersWire>)
        .transpose()
        .map_err(|e| format!("bad titleParameters: {e}"))?
        .map(Into::into);

    match event.as_str() {
        "willAppear" => Ok(WillAppear {
            action: action.ok_or("missing action")?,
            context: context.ok_or("missing context")?,
            device: device.ok_or("missing device")?,
            settings,
            controller: controller_opt.ok_or("missing payload.controller")?,
            is_in_multi_action,
            state,
            coordinates,
        }),
        "didReceiveSettings" => Ok(DidReceiveSettings {
            action: action.ok_or("missing action")?,
            context: context.ok_or("missing context")?,
            device: device.ok_or("missing device")?,
            settings,
            controller: controller_opt.ok_or("missing payload.controller")?,
            is_in_multi_action,
            state,
            coordinates,
        }),
        "keyDown" => Ok(KeyDown {
            action: action.ok_or("missing action")?,
            context: context.ok_or("missing context")?,
            device: device.ok_or("missing device")?,
            settings,
            controller: controller_opt.unwrap_or_else(|| "Keypad".to_string()),
            is_in_multi_action,
            state,
            coordinates,
        }),
        "keyUp" => Ok(KeyUp {
            action: action.ok_or("missing action")?,
            context: context.ok_or("missing context")?,
            device: device.ok_or("missing device")?,
            settings,
            controller: controller_opt.unwrap_or_else(|| "Keypad".to_string()),
            is_in_multi_action,
            state,
            coordinates,
        }),
        "willDisappear" => Ok(WillDisappear {
            action: action.ok_or("missing action")?,
            context: context.ok_or("missing context")?,
            device: device.ok_or("missing device")?,
            settings,
            controller: controller_opt.ok_or("missing payload.controller")?,
            is_in_multi_action,
            state,
            coordinates,
        }),
        "propertyInspectorDidAppear" => Ok(PropertyInspectorDidAppear {
            action: action.ok_or("missing action")?,
            context: context.ok_or("missing context")?,
            device: device.ok_or("missing device")?,
        }),
        "propertyInspectorDidDisappear" => Ok(PropertyInspectorDidDisappear {
            action: action.ok_or("missing action")?,
            context: context.ok_or("missing context")?,
            device: device.ok_or("missing device")?,
        }),
        "titleParametersDidChange" => Ok(TitleParametersDidChange {
            action: action.ok_or("missing action")?,
            context: context.ok_or("missing context")?,
            device: device.ok_or("missing device")?,
            settings,
            controller: controller_opt.ok_or("missing payload.controller")?,
            coordinates: coordinates.ok_or("missing payload.coordinates")?,
            state,
            title: title.ok_or("missing payload.title")?,
            title_parameters: title_parameters.ok_or("missing payload.titleParameters")?,
        }),
        "touchTap" => {
            let (hold, x, y) = {
                let p = payload
                    .as_ref()
                    .and_then(Value::as_object)
                    .ok_or("missing payload")?;
                let hold = p
                    .get("hold")
                    .and_then(Value::as_bool)
                    .ok_or("missing payload.hold")?;
                let tap = p
                    .get("tapPos")
                    .and_then(Value::as_array)
                    .ok_or("missing payload.tapPos")?;
                if tap.len() != 2 {
                    return Err("payload.tapPos must be [x,y]".to_string());
                }
                let x = tap[0].as_i64().ok_or("payload.tapPos[0] not i64")?;
                let y = tap[1].as_i64().ok_or("payload.tapPos[1] not i64")?;
                (hold, x, y)
            };
            Ok(TouchTap {
                action: action.ok_or("missing action")?,
                context: context.ok_or("missing context")?,
                device: device.ok_or("missing device")?,
                settings,
                controller: controller_opt.ok_or("missing payload.controller")?,
                coordinates: coordinates.ok_or("missing payload.coordinates")?,
                hold,
                tap_pos: (x, y),
            })
        }
        "dialDown" => Ok(DialDown {
            action: action.ok_or("missing action")?,
            context: context.ok_or("missing context")?,
            device: device.ok_or("missing device")?,
            settings,
            controller: controller_opt.ok_or("missing payload.controller")?,
            coordinates: coordinates.ok_or("missing payload.coordinates")?,
        }),
        "dialRotate" => {
            let (pressed, ticks) = {
                let p = payload
                    .as_ref()
                    .and_then(Value::as_object)
                    .ok_or("missing payload")?;
                let pressed = p
                    .get("pressed")
                    .and_then(Value::as_bool)
                    .ok_or("missing payload.pressed")?;
                let ticks = p
                    .get("ticks")
                    .and_then(Value::as_i64)
                    .ok_or("missing payload.ticks")?;
                (pressed, ticks)
            };
            Ok(DialRotate {
                action: action.ok_or("missing action")?,
                context: context.ok_or("missing context")?,
                device: device.ok_or("missing device")?,
                settings,
                controller: controller_opt.ok_or("missing payload.controller")?,
                coordinates: coordinates.ok_or("missing payload.coordinates")?,
                pressed,
                ticks,
            })
        }
        "dialUp" => Ok(DialUp {
            action: action.ok_or("missing action")?,
            context: context.ok_or("missing context")?,
            device: device.ok_or("missing device")?,
            settings,
            controller: controller_opt.ok_or("missing payload.controller")?,
            coordinates: coordinates.ok_or("missing payload.coordinates")?,
        }),
        "applicationDidLaunch" => Ok(ApplicationDidLaunch {
            application: payload
                .as_ref()
                .and_then(Value::as_object)
                .and_then(|p| p.get("application").and_then(Value::as_str))
                .ok_or("missing payload.application")?
                .to_string(),
        }),
        "applicationDidTerminate" => Ok(ApplicationDidTerminate {
            application: payload
                .as_ref()
                .and_then(Value::as_object)
                .and_then(|p| p.get("application").and_then(Value::as_str))
                .ok_or("missing payload.application")?
                .to_string(),
        }),
        "deviceDidChange" => Ok(DeviceDidChange {
            device: device.ok_or("missing device")?,
            device_info: serde_json::from_value(
                m.remove("deviceInfo").ok_or("missing deviceInfo")?,
            )
            .map_err(|e| format!("bad deviceInfo: {e}"))?,
        }),
        "deviceDidConnect" => Ok(DeviceDidConnect {
            device: device.ok_or("missing device")?,
            device_info: serde_json::from_value(
                m.remove("deviceInfo").ok_or("missing deviceInfo")?,
            )
            .map_err(|e| format!("bad deviceInfo: {e}"))?,
        }),
        "deviceDidDisconnect" => Ok(DeviceDidDisconnect {
            device: device.ok_or("missing device")?,
        }),
        "didReceiveDeepLink" => Ok(DidReceiveDeepLink {
            url: payload
                .as_ref()
                .and_then(Value::as_object)
                .and_then(|p| p.get("url").and_then(Value::as_str))
                .ok_or("missing payload.url")?
                .to_string(),
        }),
        "didReceiveGlobalSettings" => Ok(DidReceiveGlobalSettings { settings }),
        "sendToPlugin" => Ok(DidReceivePropertyInspectorMessage {
            action: action.ok_or("missing action")?,
            context: context.ok_or("missing context")?,
            payload: match payload {
                Some(Value::Object(obj)) => obj,
                _ => return Err("missing payload".to_string()),
            },
        }),
        "systemDidWakeUp" => Ok(SystemDidWakeUp),
        other => Err(format!("unknown StreamDeck event: {other}")),
    }
}

// =========================
// Outgoing: typed payloads
// =========================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    Both,
    Hardware,
    Software,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetTitlePayload {
    /// Title to display; None resets to the user-configured title.
    pub title: Option<String>,
    /// Optional state for multi-state actions.
    pub state: Option<SdState>,
    /// Which aspects to update.
    pub target: Option<Target>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetImagePayload {
    /// Path or base64 with data URI.
    pub image: Option<String>,
    /// Optional state for multi-state actions.
    pub state: Option<SdState>,
    /// Which aspects to update.
    pub target: Option<Target>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerPayload {
    pub long_touch: Option<String>,
    pub push: Option<String>,
    pub rotate: Option<String>,
    pub touch: Option<String>,
}

// =========================
// Outgoing: public enum
// =========================

#[derive(Debug, Clone)]
pub enum Outgoing {
    GetGlobalSettings {
        context: String,
    },
    GetSettings {
        context: String,
    },
    LogMessage {
        message: String,
    },
    OpenUrl {
        url: String,
    },
    SendToPropertyInspector {
        context: String,
        payload: Value,
    },
    SetFeedback {
        context: String,
        payload: Value,
    },
    SetFeedbackLayout {
        context: String,
        layout: String,
    },
    SetGlobalSettings {
        context: String,
        payload: Map<String, Value>,
    },
    SetImage {
        context: String,
        payload: SetImagePayload,
    },
    SetSettings {
        context: String,
        payload: Map<String, Value>,
    },
    SetState {
        context: String,
        state: SdState,
    },
    SetTitle {
        context: String,
        payload: SetTitlePayload,
    },
    SetTriggerDescription {
        context: String,
        payload: TriggerPayload,
    },
    ShowAlert {
        context: String,
    },
    ShowOk {
        context: String,
    },
}

// Internal: serializable shape
#[derive(Serialize)]
#[serde(tag = "event")]
enum WireOutgoing<'a> {
    #[serde(rename = "getGlobalSettings")]
    GetGlobalSettings { context: &'a str },

    #[serde(rename = "getSettings")]
    GetSettings { context: &'a str },

    #[serde(rename = "logMessage")]
    LogMessage { payload: WireLogMessage<'a> },

    #[serde(rename = "openUrl")]
    OpenUrl { payload: WireOpenUrl<'a> },

    #[serde(rename = "sendToPropertyInspector")]
    SendToPropertyInspector {
        context: &'a str,
        payload: &'a Value,
    },

    #[serde(rename = "setFeedback")]
    SetFeedback {
        context: &'a str,
        payload: &'a Value,
    },

    #[serde(rename = "setFeedbackLayout")]
    SetFeedbackLayout {
        context: &'a str,
        payload: WireLayout<'a>,
    },

    #[serde(rename = "setGlobalSettings")]
    SetGlobalSettings {
        context: &'a str,
        payload: &'a Map<String, Value>,
    },

    #[serde(rename = "setImage")]
    SetImage {
        context: &'a str,
        payload: &'a SetImagePayload,
    },

    #[serde(rename = "setSettings")]
    SetSettings {
        context: &'a str,
        payload: &'a Map<String, Value>,
    },

    #[serde(rename = "setState")]
    SetState {
        context: &'a str,
        payload: WireState,
    },

    #[serde(rename = "setTitle")]
    SetTitle {
        context: &'a str,
        payload: &'a SetTitlePayload,
    },

    #[serde(rename = "setTriggerDescription")]
    SetTriggerDescription {
        context: &'a str,
        payload: &'a TriggerPayload,
    },

    #[serde(rename = "showAlert")]
    ShowAlert { context: &'a str },

    #[serde(rename = "showOk")]
    ShowOk { context: &'a str },
}

#[derive(Serialize)]
struct WireLogMessage<'a> {
    message: &'a str,
}
#[derive(Serialize)]
struct WireOpenUrl<'a> {
    url: &'a str,
}
#[derive(Serialize)]
struct WireLayout<'a> {
    layout: &'a str,
}
#[derive(Serialize)]
struct WireState {
    state: SdState,
}

impl<'a> From<&'a Outgoing> for WireOutgoing<'a> {
    fn from(o: &'a Outgoing) -> Self {
        use Outgoing::*;
        match o {
            GetGlobalSettings { context } => WireOutgoing::GetGlobalSettings { context },
            GetSettings { context } => WireOutgoing::GetSettings { context },
            LogMessage { message } => WireOutgoing::LogMessage {
                payload: WireLogMessage { message },
            },
            OpenUrl { url } => WireOutgoing::OpenUrl {
                payload: WireOpenUrl { url },
            },
            SendToPropertyInspector { context, payload } => {
                WireOutgoing::SendToPropertyInspector { context, payload }
            }
            SetFeedback { context, payload } => WireOutgoing::SetFeedback { context, payload },
            SetFeedbackLayout { context, layout } => WireOutgoing::SetFeedbackLayout {
                context,
                payload: WireLayout { layout },
            },
            SetGlobalSettings { context, payload } => {
                WireOutgoing::SetGlobalSettings { context, payload }
            }
            SetImage { context, payload } => WireOutgoing::SetImage { context, payload },
            SetSettings { context, payload } => WireOutgoing::SetSettings { context, payload },
            SetState { context, state } => WireOutgoing::SetState {
                context,
                payload: WireState { state: *state },
            },
            SetTitle { context, payload } => WireOutgoing::SetTitle { context, payload },
            SetTriggerDescription { context, payload } => {
                WireOutgoing::SetTriggerDescription { context, payload }
            }
            ShowAlert { context } => WireOutgoing::ShowAlert { context },
            ShowOk { context } => WireOutgoing::ShowOk { context },
        }
    }
}

pub fn serialize_outgoing(msg: &Outgoing) -> serde_json::Result<String> {
    let w: WireOutgoing = msg.into();
    serde_json::to_string(&w)
}

// =========================
// Thin, typed client
// =========================

use crossbeam_channel::Sender;

use crate::events::RuntimeMsg;

#[derive(Clone)]
pub struct SdClient {
    tx: Sender<RuntimeMsg>,
    plugin_uuid: String,
}

impl SdClient {
    pub(crate) fn new(tx: Sender<RuntimeMsg>, plugin_uuid: impl Into<String>) -> Self {
        Self {
            tx,
            plugin_uuid: plugin_uuid.into(),
        }
    }

    #[inline]
    fn send(&self, o: Outgoing) {
        trace!("📤 WebSocket outgoing: {:#?}", o);
        let _ = self.tx.send(RuntimeMsg::Outgoing(o));
    }

    pub fn get_global_settings(&self) {
        self.send(Outgoing::GetGlobalSettings {
            context: self.plugin_uuid.clone(),
        });
    }
    pub fn get_settings(&self, context: impl Into<String>) {
        self.send(Outgoing::GetSettings {
            context: context.into(),
        });
    }
    pub fn log_message(&self, message: impl Into<String>) {
        self.send(Outgoing::LogMessage {
            message: message.into(),
        });
    }
    pub fn open_url(&self, url: impl Into<String>) {
        self.send(Outgoing::OpenUrl { url: url.into() });
    }
    pub fn send_to_property_inspector(&self, context: impl Into<String>, payload: Value) {
        self.send(Outgoing::SendToPropertyInspector {
            context: context.into(),
            payload,
        });
    }
    pub fn set_feedback(&self, context: impl Into<String>, payload: Value) {
        self.send(Outgoing::SetFeedback {
            context: context.into(),
            payload,
        });
    }
    pub fn set_feedback_layout(&self, context: impl Into<String>, layout: impl Into<String>) {
        self.send(Outgoing::SetFeedbackLayout {
            context: context.into(),
            layout: layout.into(),
        });
    }
    pub fn set_global_settings(&self, settings: Map<String, Value>) {
        self.send(Outgoing::SetGlobalSettings {
            context: self.plugin_uuid.clone(),
            payload: settings,
        });
    }
    pub fn set_image(
        &self,
        context: impl Into<String>,
        image_base64: Option<String>,
        state: Option<SdState>,
        target: Option<Target>,
    ) {
        self.send(Outgoing::SetImage {
            context: context.into(),
            payload: SetImagePayload {
                image: image_base64,
                state,
                target,
            },
        });
    }
    pub fn set_settings(&self, context: impl Into<String>, settings: Map<String, Value>) {
        self.send(Outgoing::SetSettings {
            context: context.into(),
            payload: settings,
        });
    }
    pub fn set_state(&self, context: impl Into<String>, state: SdState) {
        self.send(Outgoing::SetState {
            context: context.into(),
            state,
        });
    }
    pub fn set_title(
        &self,
        context: impl Into<String>,
        title: Option<String>,
        state: Option<SdState>,
        target: Option<Target>,
    ) {
        self.send(Outgoing::SetTitle {
            context: context.into(),
            payload: SetTitlePayload {
                title,
                state,
                target,
            },
        });
    }

    // ergonomic helpers
    pub fn set_title_simple(&self, ctx: impl Into<String>, title: impl Into<String>) {
        self.set_title(ctx, Some(title.into()), None, None);
    }
    pub fn clear_title(&self, ctx: impl Into<String>) {
        self.set_title(ctx, None, None, None);
    }
    pub fn set_image_b64(&self, ctx: impl Into<String>, b64: impl Into<String>) {
        self.set_image(ctx, Some(b64.into()), None, None);
    }

    pub fn set_trigger_description(
        &self,
        context: impl Into<String>,
        long_touch: Option<String>,
        push: Option<String>,
        rotate: Option<String>,
        touch: Option<String>,
    ) {
        self.send(Outgoing::SetTriggerDescription {
            context: context.into(),
            payload: TriggerPayload {
                long_touch,
                push,
                rotate,
                touch,
            },
        });
    }
    pub fn show_alert(&self, context: impl Into<String>) {
        self.send(Outgoing::ShowAlert {
            context: context.into(),
        });
    }
    pub fn show_ok(&self, context: impl Into<String>) {
        self.send(Outgoing::ShowOk {
            context: context.into(),
        });
    }
}
