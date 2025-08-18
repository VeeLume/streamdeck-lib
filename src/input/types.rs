use std::time::Duration;

use serde::{ Deserialize, Serialize };

/// Windows-style scancode and E0/E1 "extended" flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Scan {
    pub code: u16,
    pub extended: bool,
}
impl Scan {
    pub const fn new(code: u16, extended: bool) -> Self {
        Self { code, extended }
    }
    #[inline]
    pub const fn as_tuple(self) -> (u16, bool) { (self.code, self.extended) }
}
impl core::fmt::Display for Scan {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "scan=0x{:02X}, ext={}", self.code, self.extended)
    }
}

/// Mouse buttons we support portably.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    /// 1 = XButton1, 2 = XButton2
    X(u16),
}
impl MouseButton {
    pub const X1: MouseButton = MouseButton::X(1);
    pub const X2: MouseButton = MouseButton::X(2);

    #[inline]
    pub fn is_x(self) -> bool {
        matches!(self, MouseButton::X(_))
    }
}

/// Primitive, platform-agnostic input steps.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputStep {
    KeyDown(Scan),
    KeyUp(Scan),
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    Sleep(Duration),
}
