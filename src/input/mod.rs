pub mod types;
pub use types::{InputStep, MouseButton, Scan};

pub mod key;
pub use key::Key;

pub mod dsl;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::WinSynth;

use std::sync::Arc;

/// Platform-agnostic interface. Implemented by OS backends.
pub trait InputSynth: Send + Sync + 'static {
    fn send_step(&self, step: &InputStep) -> Result<(), String>;

    fn send_steps<I>(&self, steps: I) -> Result<(), String>
    where
        I: IntoIterator<Item = InputStep>,
    {
        for s in steps {
            self.send_step(&s)?;
        }
        Ok(())
    }
}

/// Optional worker to serialize steps.
pub struct Executor<S: InputSynth + ?Sized> {
    tx: crossbeam_channel::Sender<InputStep>,
    join: Option<std::thread::JoinHandle<()>>,
    synth: Arc<S>,
}

impl<S: InputSynth + ?Sized> Executor<S> {
    pub fn new(synth: Arc<S>) -> Self {
        Self::new_inner(synth, crossbeam_channel::unbounded())
    }

    /// Bounded queue with backpressure (optional).
    pub fn new_bounded(synth: Arc<S>, cap: usize) -> Self {
        Self::new_inner(synth, crossbeam_channel::bounded(cap))
    }

    fn new_inner(
        synth: Arc<S>,
        (tx, rx): (
            crossbeam_channel::Sender<InputStep>,
            crossbeam_channel::Receiver<InputStep>,
        ),
    ) -> Self {
        let s2 = Arc::clone(&synth);
        let join = std::thread::spawn(move || {
            for step in rx.iter() {
                let _ = s2.send_step(&step);
            }
        });
        Self {
            tx,
            join: Some(join),
            synth,
        }
    }

    /// Queue a single step (fire-and-forget).
    pub fn enqueue(&self, step: InputStep) {
        let _ = self.tx.send(step);
    }

    /// Queue a single step; surface send error.
    pub fn try_enqueue(
        &self,
        step: InputStep,
    ) -> Result<(), crossbeam_channel::SendError<InputStep>> {
        self.tx.send(step)
    }

    pub fn enqueue_all<I: IntoIterator<Item = InputStep>>(&self, steps: I) {
        for s in steps {
            let _ = self.tx.send(s);
        }
    }

    pub fn synth(&self) -> &Arc<S> {
        &self.synth
    }
}

impl<S: InputSynth + ?Sized> Drop for Executor<S> {
    fn drop(&mut self) {
        // Closing the sender ends the worker loop.
        // Take the join handle so we only join once.
        if let Some(j) = self.join.take() {
            drop(self.tx.clone()); // close
            let _ = j.join();
        }
    }
}
