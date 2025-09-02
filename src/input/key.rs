// src/input/key.rs
//! Typed key identifiers with a built-in Windows scancode map.
//! Keeps scancode math out of plugins. No game semantics here.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::types::{InputStep, Scan};

/// Typed keys. Add more as you need; `Custom` lets you provide raw scancodes.
/// Mapping is Windows-only right now (guarded with #[cfg(windows)]).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Key {
    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Number row
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,

    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // Modifiers
    LShift,
    RShift,
    LCtrl,
    RCtrl,
    LAlt,
    RAlt,
    LWin,
    RWin,

    // Symbols / misc
    Space,
    Tab,
    Enter,
    Escape,
    Backspace,
    Minus,
    Equal,
    LBracket,
    RBracket,
    Semicolon,
    Apostrophe,
    Comma,
    Period,
    Slash,
    Backslash,
    Grave, // `~ key
    CapsLock,
    Print, // Print Screen
    Pause, // Pause/Break

    // Navigation
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    // Numpad
    Np0,
    Np1,
    Np2,
    Np3,
    Np4,
    Np5,
    Np6,
    Np7,
    Np8,
    Np9,
    NpAdd,
    NpSubtract,
    NpMultiply,
    NpDivide,
    NpEnter,
    NpDecimal,
    NpLock,

    // Menu / context
    Menu,

    /// Raw scancode (Windows SetScanCode) + extended flag.
    /// Use to cover keys not yet in the enum.
    Custom {
        scan: u16,
        extended: bool,
    },
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::Custom { scan, extended } => {
                write!(f, "Custom(scan: {scan}, extended: {extended})")
            }
            _ => write!(f, "{self:?}"),
        }
    }
}

impl Key {
    /// Build a custom key without naming the fields.
    #[inline]
    pub const fn custom(scan: u16, extended: bool) -> Self {
        Key::Custom { scan, extended }
    }

    /// Quick check for common modifiers.
    #[inline]
    pub fn is_modifier(self) -> bool {
        matches!(
            self,
            Key::LShift
                | Key::RShift
                | Key::LCtrl
                | Key::RCtrl
                | Key::LAlt
                | Key::RAlt
                | Key::LWin
                | Key::RWin
        )
    }

    /// Best-effort name parser (case-insensitive). Keeps this in the lib so
    /// plugin PIs can pass strings safely without hand-rolled tables.
    ///
    /// Examples: "a", "F5", "lctrl", "np_1", "npDivide", "arrow_left", "left"
    pub fn parse(name: &str) -> Option<Self> {
        let s = name.trim().to_lowercase();

        // letters
        if s.len() == 1 {
            if let Some(c) = s.chars().next() {
                if c.is_ascii_lowercase() {
                    return Some(match c {
                        'a' => Key::A,
                        'b' => Key::B,
                        'c' => Key::C,
                        'd' => Key::D,
                        'e' => Key::E,
                        'f' => Key::F,
                        'g' => Key::G,
                        'h' => Key::H,
                        'i' => Key::I,
                        'j' => Key::J,
                        'k' => Key::K,
                        'l' => Key::L,
                        'm' => Key::M,
                        'n' => Key::N,
                        'o' => Key::O,
                        'p' => Key::P,
                        'q' => Key::Q,
                        'r' => Key::R,
                        's' => Key::S,
                        't' => Key::T,
                        'u' => Key::U,
                        'v' => Key::V,
                        'w' => Key::W,
                        'x' => Key::X,
                        'y' => Key::Y,
                        'z' => Key::Z,
                        _ => {
                            return None;
                        }
                    });
                }
                if c.is_ascii_digit() {
                    return Some(match c {
                        '0' => Key::D0,
                        '1' => Key::D1,
                        '2' => Key::D2,
                        '3' => Key::D3,
                        '4' => Key::D4,
                        '5' => Key::D5,
                        '6' => Key::D6,
                        '7' => Key::D7,
                        '8' => Key::D8,
                        '9' => Key::D9,
                        _ => {
                            return None;
                        }
                    });
                }
            }
        }

        // common aliases
        let s = s.replace('-', "_");
        let s = s.as_str();
        Some(match s {
            // function
            "f1" => Key::F1,
            "f2" => Key::F2,
            "f3" => Key::F3,
            "f4" => Key::F4,
            "f5" => Key::F5,
            "f6" => Key::F6,
            "f7" => Key::F7,
            "f8" => Key::F8,
            "f9" => Key::F9,
            "f10" => Key::F10,
            "f11" => Key::F11,
            "f12" => Key::F12,

            // modifiers
            "lshift" | "left_shift" => Key::LShift,
            "rshift" | "right_shift" => Key::RShift,
            "lctrl" | "left_ctrl" => Key::LCtrl,
            "rctrl" | "right_ctrl" => Key::RCtrl,
            "lalt" | "left_alt" => Key::LAlt,
            "ralt" | "right_alt" => Key::RAlt,
            "lwin" | "left_win" | "lmeta" | "super" | "meta" | "win" => Key::LWin,
            "rwin" | "right_win" | "rmeta" => Key::RWin,

            // misc
            "space" => Key::Space,
            "tab" => Key::Tab,
            "enter" | "return" => Key::Enter,
            "esc" | "escape" => Key::Escape,
            "backspace" => Key::Backspace,
            "minus" | "-" => Key::Minus,
            "equal" | "=" => Key::Equal,
            "[" | "lbracket" => Key::LBracket,
            "]" | "rbracket" => Key::RBracket,
            ";" | "semicolon" => Key::Semicolon,
            "'" | "apostrophe" | "quote" => Key::Apostrophe,
            "," | "comma" => Key::Comma,
            "." | "period" | "dot" => Key::Period,
            "/" | "slash" => Key::Slash,
            "\\" | "backslash" => Key::Backslash,
            "`" | "grave" | "tilde" => Key::Grave,

            // arrows / nav
            "up" | "arrow_up" => Key::ArrowUp,
            "down" | "arrow_down" => Key::ArrowDown,
            "left" | "arrow_left" => Key::ArrowLeft,
            "right" | "arrow_right" => Key::ArrowRight,
            "home" => Key::Home,
            "end" => Key::End,
            "insert" | "ins" => Key::Insert,
            "delete" | "del" => Key::Delete,
            "pgup" | "page_up" | "pageup" => Key::PageUp,
            "pgdn" | "page_down" | "pagedown" => Key::PageDown,

            // numpad
            "np_0" | "numpad0" => Key::Np0,
            "np_1" | "numpad1" => Key::Np1,
            "np_2" | "numpad2" => Key::Np2,
            "np_3" | "numpad3" => Key::Np3,
            "np_4" | "numpad4" => Key::Np4,
            "np_5" | "numpad5" => Key::Np5,
            "np_6" | "numpad6" => Key::Np6,
            "np_7" | "numpad7" => Key::Np7,
            "np_8" | "numpad8" => Key::Np8,
            "np_9" | "numpad9" => Key::Np9,
            "np_add" | "numpad_add" => Key::NpAdd,
            "np_subtract" | "numpad_subtract" => Key::NpSubtract,
            "np_multiply" | "numpad_multiply" => Key::NpMultiply,
            "np_divide" | "numpad_divide" => Key::NpDivide,
            "np_enter" | "numpad_enter" => Key::NpEnter,
            "np_period" | "numpad_decimal" | "np_decimal" => Key::NpDecimal,

            "menu" | "apps" | "context" => Key::Menu,
            "capslock" | "caps_lock" => Key::CapsLock,
            "print" | "prtsc" | "print_screen" => Key::Print,
            "pause" | "break" => Key::Pause,
            _ => {
                return None;
            }
        })
    }

    /// Convert to a Windows scancode (SetScanCode) + extended flag.
    /// For non-Windows targets, this returns `None`.
    /// /// (e.g. `Print` and `Pause`) or when compiled for non-Windows targets.
    pub fn to_scan(self) -> Option<Scan> {
        #[cfg(windows)]
        {
            use Key::*;
            let (ext, sc) = match self {
                // letters
                A => (false, 0x1e),
                B => (false, 0x30),
                C => (false, 0x2e),
                D => (false, 0x20),
                E => (false, 0x12),
                F => (false, 0x21),
                G => (false, 0x22),
                H => (false, 0x23),
                I => (false, 0x17),
                J => (false, 0x24),
                K => (false, 0x25),
                L => (false, 0x26),
                M => (false, 0x32),
                N => (false, 0x31),
                O => (false, 0x18),
                P => (false, 0x19),
                Q => (false, 0x10),
                R => (false, 0x13),
                S => (false, 0x1f),
                T => (false, 0x14),
                U => (false, 0x16),
                V => (false, 0x2f),
                W => (false, 0x11),
                X => (false, 0x2d),
                Y => (false, 0x15),
                Z => (false, 0x2c),

                // number row
                D1 => (false, 0x02),
                D2 => (false, 0x03),
                D3 => (false, 0x04),
                D4 => (false, 0x05),
                D5 => (false, 0x06),
                D6 => (false, 0x07),
                D7 => (false, 0x08),
                D8 => (false, 0x09),
                D9 => (false, 0x0a),
                D0 => (false, 0x0b),

                // function
                F1 => (false, 0x3b),
                F2 => (false, 0x3c),
                F3 => (false, 0x3d),
                F4 => (false, 0x3e),
                F5 => (false, 0x3f),
                F6 => (false, 0x40),
                F7 => (false, 0x41),
                F8 => (false, 0x42),
                F9 => (false, 0x43),
                F10 => (false, 0x44),
                F11 => (false, 0x57),
                F12 => (false, 0x58),

                // modifiers
                LShift => (false, 0x2a),
                RShift => (false, 0x36),
                LCtrl => (false, 0x1d),
                RCtrl => (true, 0x1d),
                LAlt => (false, 0x38),
                RAlt => (true, 0x38),
                LWin => (true, 0x5b),
                RWin => (true, 0x5c),

                // misc
                Space => (false, 0x39),
                Tab => (false, 0x0f),
                Enter => (false, 0x1c),
                Escape => (false, 0x01),
                Backspace => (false, 0x0e),
                Minus => (false, 0x0c),
                Equal => (false, 0x0d),
                LBracket => (false, 0x1a),
                RBracket => (false, 0x1b),
                Semicolon => (false, 0x27),
                Apostrophe => (false, 0x28),
                Comma => (false, 0x33),
                Period => (false, 0x34),
                Slash => (false, 0x35),
                Backslash => (false, 0x2b),
                Grave => (false, 0x29),
                CapsLock => (false, 0x3a),
                // Print => E0 2A E0 37
                // Pause => E1 1D 45 E1 9D C5

                // nav
                Insert => (true, 0x52),
                Delete => (true, 0x53),
                Home => (true, 0x47),
                End => (true, 0x4f),
                PageUp => (true, 0x49),
                PageDown => (true, 0x51),
                ArrowUp => (true, 0x48),
                ArrowDown => (true, 0x50),
                ArrowLeft => (true, 0x4b),
                ArrowRight => (true, 0x4d),

                // numpad
                Np0 => (false, 0x52),
                Np1 => (false, 0x4f),
                Np2 => (false, 0x50),
                Np3 => (false, 0x51),
                Np4 => (false, 0x4b),
                Np5 => (false, 0x4c),
                Np6 => (false, 0x4d),
                Np7 => (false, 0x47),
                Np8 => (false, 0x48),
                Np9 => (false, 0x49),
                NpAdd => (false, 0x4e),
                NpSubtract => (false, 0x4a),
                NpMultiply => (false, 0x37),
                NpDivide => (true, 0x35),
                NpEnter => (true, 0x1c),
                NpDecimal => (false, 0x53),
                NpLock => (false, 0x45),

                Menu => (true, 0x5d),

                Custom { scan, extended } => {
                    return Some(Scan::new(scan, extended));
                }

                _ => {
                    return None;
                }
            };
            Some(Scan::new(sc, ext))
        }
        #[cfg(not(windows))]
        {
            // No mapping for non-Windows yet.
            match self {
                Key::Custom { scan, extended } => Some(Scan::new(scan, extended)),
                _ => None,
            }
        }
    }

    /// Convenience: turn a Key into a press or release step.
    pub fn to_step_down(self) -> Option<InputStep> {
        self.to_scan().map(InputStep::KeyDown)
    }
    pub fn to_step_up(self) -> Option<InputStep> {
        self.to_scan().map(InputStep::KeyUp)
    }

    /// Single source of truth: all known variants except `Custom`.
    pub const ALL: &'static [Key] = &[
        // Letters
        Key::A,
        Key::B,
        Key::C,
        Key::D,
        Key::E,
        Key::F,
        Key::G,
        Key::H,
        Key::I,
        Key::J,
        Key::K,
        Key::L,
        Key::M,
        Key::N,
        Key::O,
        Key::P,
        Key::Q,
        Key::R,
        Key::S,
        Key::T,
        Key::U,
        Key::V,
        Key::W,
        Key::X,
        Key::Y,
        Key::Z,
        // Number row
        Key::D0,
        Key::D1,
        Key::D2,
        Key::D3,
        Key::D4,
        Key::D5,
        Key::D6,
        Key::D7,
        Key::D8,
        Key::D9,
        // Function keys
        Key::F1,
        Key::F2,
        Key::F3,
        Key::F4,
        Key::F5,
        Key::F6,
        Key::F7,
        Key::F8,
        Key::F9,
        Key::F10,
        Key::F11,
        Key::F12,
        // Modifiers
        Key::LShift,
        Key::RShift,
        Key::LCtrl,
        Key::RCtrl,
        Key::LAlt,
        Key::RAlt,
        Key::LWin,
        Key::RWin,
        // Symbols / misc
        Key::Space,
        Key::Tab,
        Key::Enter,
        Key::Escape,
        Key::Backspace,
        Key::Minus,
        Key::Equal,
        Key::LBracket,
        Key::RBracket,
        Key::Semicolon,
        Key::Apostrophe,
        Key::Comma,
        Key::Period,
        Key::Slash,
        Key::Backslash,
        Key::Grave,
        Key::CapsLock,
        Key::Print,
        Key::Pause,
        // Navigation
        Key::Insert,
        Key::Delete,
        Key::Home,
        Key::End,
        Key::PageUp,
        Key::PageDown,
        Key::ArrowUp,
        Key::ArrowDown,
        Key::ArrowLeft,
        Key::ArrowRight,
        // Numpad
        Key::Np0,
        Key::Np1,
        Key::Np2,
        Key::Np3,
        Key::Np4,
        Key::Np5,
        Key::Np6,
        Key::Np7,
        Key::Np8,
        Key::Np9,
        Key::NpAdd,
        Key::NpSubtract,
        Key::NpMultiply,
        Key::NpDivide,
        Key::NpEnter,
        Key::NpDecimal,
        Key::NpLock,
        // Menu
        Key::Menu,
    ];

    /// Iterate all keys (excludes `Custom`).
    #[inline]
    pub fn iter() -> impl Iterator<Item = Key> + 'static {
        Self::ALL.iter().copied()
    }

    #[inline]
    pub fn to_token(self) -> &'static str {
        use Key::*;
        match self {
            // letters
            A => "a",
            B => "b",
            C => "c",
            D => "d",
            E => "e",
            F => "f",
            G => "g",
            H => "h",
            I => "i",
            J => "j",
            K => "k",
            L => "l",
            M => "m",
            N => "n",
            O => "o",
            P => "p",
            Q => "q",
            R => "r",
            S => "s",
            T => "t",
            U => "u",
            V => "v",
            W => "w",
            X => "x",
            Y => "y",
            Z => "z",

            // number row
            D0 => "0",
            D1 => "1",
            D2 => "2",
            D3 => "3",
            D4 => "4",
            D5 => "5",
            D6 => "6",
            D7 => "7",
            D8 => "8",
            D9 => "9",

            // function keys
            F1 => "f1",
            F2 => "f2",
            F3 => "f3",
            F4 => "f4",
            F5 => "f5",
            F6 => "f6",
            F7 => "f7",
            F8 => "f8",
            F9 => "f9",
            F10 => "f10",
            F11 => "f11",
            F12 => "f12",

            // modifiers
            LShift => "lshift",
            RShift => "rshift",
            LCtrl => "lctrl",
            RCtrl => "rctrl",
            LAlt => "lalt",
            RAlt => "ralt",
            LWin => "lwin",
            RWin => "rwin",

            // symbols / misc
            Space => "space",
            Tab => "tab",
            Enter => "enter",
            Escape => "escape",
            Backspace => "backspace",
            Minus => "minus",
            Equal => "equals",
            LBracket => "lbracket",
            RBracket => "rbracket",
            Semicolon => "semicolon",
            Apostrophe => "apostrophe",
            Comma => "comma",
            Period => "period",
            Slash => "slash",
            Backslash => "backslash",
            Grave => "grave",
            CapsLock => "capslock",
            Print => "print",
            Pause => "pause",

            // navigation
            Insert => "insert",
            Delete => "delete",
            Home => "home",
            End => "end",
            PageUp => "pgup",
            PageDown => "pgdn",
            ArrowUp => "up",
            ArrowDown => "down",
            ArrowLeft => "left",
            ArrowRight => "right",

            // numpad
            Np0 => "np_0",
            Np1 => "np_1",
            Np2 => "np_2",
            Np3 => "np_3",
            Np4 => "np_4",
            Np5 => "np_5",
            Np6 => "np_6",
            Np7 => "np_7",
            Np8 => "np_8",
            Np9 => "np_9",
            NpAdd => "np_add",
            NpSubtract => "np_subtract",
            NpMultiply => "np_multiply",
            NpDivide => "np_divide",
            NpEnter => "np_enter",
            NpDecimal => "np_period",
            NpLock => "np_lock",

            Menu => "menu",

            // You generally shouldn't emit a token for `Custom`
            Custom { .. } => "custom",
        }
    }

    /// Convenience if you want the raw tokens directly.
    pub fn iter_tokens() -> impl Iterator<Item = &'static str> {
        Self::iter().map(|k| k.to_token())
    }
}
