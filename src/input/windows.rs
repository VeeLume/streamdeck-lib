use std::mem::size_of;
use std::thread;

use windows::Win32::UI::Input::KeyboardAndMouse::*;

use super::InputSynth;
use super::types::{InputStep, MouseButton, Scan};

pub struct WinSynth;

impl WinSynth {
    pub fn new() -> Self {
        Self
    }

    /// Fast path: send many steps with as few `SendInput` calls as possible.
    /// `Sleep` acts as a flush boundary.
    pub fn send_batch<I>(&self, steps: I) -> Result<(), String>
    where
        I: IntoIterator<Item = InputStep>,
    {
        let mut buf: Vec<INPUT> = Vec::with_capacity(16);

        let flush = |buf: &mut Vec<INPUT>| -> Result<(), String> {
            if buf.is_empty() {
                return Ok(());
            }
            let sent = unsafe { SendInput(&buf[..], size_of::<INPUT>() as i32) };
            buf.clear();
            if sent == 0 {
                Err("SendInput failed".into())
            } else {
                Ok(())
            }
        };

        for step in steps {
            match step {
                InputStep::KeyDown(s) => buf.push(build_key(s, true)),
                InputStep::KeyUp(s) => buf.push(build_key(s, false)),
                InputStep::MouseDown(b) => buf.push(build_mouse(down_flag(b), mouse_data(b))),
                InputStep::MouseUp(b) => buf.push(build_mouse(up_flag(b), mouse_data(b))),
                InputStep::Sleep(dur) => {
                    flush(&mut buf)?;
                    thread::sleep(dur);
                }
            }
        }

        flush(&mut buf)
    }
}

impl InputSynth for WinSynth {
    /// Keep `send_step` simple but still use the slice-based binding.
    fn send_step(&self, step: &InputStep) -> Result<(), String> {
        match *step {
            InputStep::KeyDown(s) => send_one(build_key(s, true)),
            InputStep::KeyUp(s) => send_one(build_key(s, false)),
            InputStep::MouseDown(b) => send_one(build_mouse(down_flag(b), mouse_data(b))),
            InputStep::MouseUp(b) => send_one(build_mouse(up_flag(b), mouse_data(b))),
            InputStep::Sleep(d) => {
                thread::sleep(d);
                Ok(())
            }
        }
    }

    /// Override the default to batch efficiently.
    fn send_steps<I>(&self, steps: I) -> Result<(), String>
    where
        I: IntoIterator<Item = InputStep>,
    {
        self.send_batch(steps)
    }
}

#[inline]
fn down_flag(btn: MouseButton) -> MOUSE_EVENT_FLAGS {
    match btn {
        MouseButton::Left => MOUSEEVENTF_LEFTDOWN,
        MouseButton::Right => MOUSEEVENTF_RIGHTDOWN,
        MouseButton::Middle => MOUSEEVENTF_MIDDLEDOWN,
        MouseButton::X(_) => MOUSEEVENTF_XDOWN,
    }
}

#[inline]
fn up_flag(btn: MouseButton) -> MOUSE_EVENT_FLAGS {
    match btn {
        MouseButton::Left => MOUSEEVENTF_LEFTUP,
        MouseButton::Right => MOUSEEVENTF_RIGHTUP,
        MouseButton::Middle => MOUSEEVENTF_MIDDLEUP,
        MouseButton::X(_) => MOUSEEVENTF_XUP,
    }
}

#[inline]
fn mouse_data(btn: MouseButton) -> u32 {
    match btn {
        MouseButton::X(1) => 0x0001,
        MouseButton::X(2) => 0x0002,
        MouseButton::X(n) => n as u32, // undefined beyond 1/2; avoid if you can
        _ => 0,
    }
}

#[inline]
fn build_key(s: Scan, down: bool) -> INPUT {
    let mut flags = KEYEVENTF_SCANCODE;
    if s.extended {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }
    if !down {
        flags |= KEYEVENTF_KEYUP;
    }

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: s.code,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0, // ok to zero
            },
        },
    }
}

#[inline]
fn build_mouse(flags: MOUSE_EVENT_FLAGS, data: u32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

#[inline]
fn send_one(input: INPUT) -> Result<(), String> {
    let n = unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
    if n == 0 {
        Err("SendInput failed".into())
    } else {
        Ok(())
    }
}
