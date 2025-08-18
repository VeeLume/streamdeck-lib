use std::time::Duration;
use super::{InputStep, Key, MouseButton};

#[inline] pub fn sleep_ms(ms: u64) -> InputStep { InputStep::Sleep(Duration::from_millis(ms)) }

#[inline]
pub fn sleep(d: Duration) -> InputStep { InputStep::Sleep(d) }

#[inline]
pub fn tap(k: Key) -> Vec<InputStep> {
    let mut v = Vec::new();
    if let Some(s) = k.to_step_down() { v.push(s); }
    if let Some(s) = k.to_step_up()   { v.push(s); }
    v
}

#[inline] pub fn down(k: Key) -> Option<InputStep> { k.to_step_down() }
#[inline] pub fn up(k: Key)   -> Option<InputStep> { k.to_step_up()   }

/// Press and release `main` with modifiers held.
#[inline]
pub fn chord(mods: &[Key], main: Key) -> Vec<InputStep> {
    let mut v = Vec::new();
    for &m in mods { if let Some(s)=down(m){ v.push(s) } }
    v.extend(tap(main));
    for &m in mods.iter().rev() { if let Some(s)=up(m){ v.push(s) } }
    v
}

/// Hold `main` for `ms` with modifiers held.
#[inline]
pub fn hold(mods: &[Key], main: Key, ms: u64) -> Vec<InputStep> {
    let mut v = Vec::new();
    for &m in mods { if let Some(s)=down(m){ v.push(s) } }
    if let Some(s)=down(main){ v.push(s) }
    v.push(sleep_ms(ms));
    if let Some(s)=up(main){ v.push(s) }
    for &m in mods.iter().rev() { if let Some(s)=up(m){ v.push(s) } }
    v
}

/// Tap `k`, then wait `ms`.
#[inline]
pub fn tap_with_delay(k: Key, ms: u64) -> Vec<InputStep> {
    let mut v = tap(k);
    v.push(sleep_ms(ms));
    v
}

#[inline]
pub fn click(btn: MouseButton) -> Vec<InputStep> {
    vec![ InputStep::MouseDown(btn), InputStep::MouseUp(btn) ]
}

/// Click `n` times with an optional delay between clicks.
#[inline]
pub fn click_n(btn: MouseButton, n: usize, between: Option<Duration>) -> Vec<InputStep> {
    let mut v = Vec::with_capacity(n * 3);
    for i in 0..n {
        v.push(InputStep::MouseDown(btn));
        v.push(InputStep::MouseUp(btn));
        if i + 1 != n {
            if let Some(d) = between { v.push(InputStep::Sleep(d)); }
        }
    }
    v
}
