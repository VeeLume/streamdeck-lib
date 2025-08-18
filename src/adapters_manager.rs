// adapters_manager.rs
use crate::{
    adapters::{Adapter, AdapterHandle, StartPolicy},
    bus::Bus,
    context::Context,
    debug, error,
    events::{AdapterTarget, ErasedTopic},
    logger::ActionLog,
};
use crossbeam_channel::{Sender, unbounded};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

struct RunningAdapter {
    name: &'static str,
    policy: StartPolicy,
    topics: &'static [&'static str],
    tx: Sender<Arc<ErasedTopic>>,
    handle: AdapterHandle,
}

pub(crate) struct AdapterManager {
    registry: Vec<Arc<dyn Adapter + Send + Sync + 'static>>,
    running: Vec<RunningAdapter>,
    by_name: HashMap<&'static str, Vec<usize>>,
    by_topic: HashMap<&'static str, Vec<usize>>,

    // lifecycle counters
    apps_up: usize,

    // debounce for stopping OnAppLaunch adapters after the *last* app quits
    app_stop_due: Option<Instant>, // <— when to stop OnAppLaunch adapters
    app_debounce: Duration,        // <— debounce delay

    // infra
    bus: Arc<dyn Bus>,
    logger: Arc<dyn ActionLog>,
}

impl AdapterManager {
    pub fn new(
        adapters: &[Arc<dyn Adapter + Send + Sync + 'static>],
        bus: Arc<dyn Bus>,
        logger: Arc<dyn ActionLog>,
    ) -> Self {
        Self {
            registry: adapters.iter().cloned().collect(),
            running: Vec::new(),
            by_name: HashMap::new(),
            by_topic: HashMap::new(),
            apps_up: 0,
            app_stop_due: None,
            app_debounce: Duration::from_millis(250),
            bus,
            logger,
        }
    }

    // ---- small helpers --------------------------------------------------

    fn is_running_name(&self, name: &str) -> bool {
        self.by_name.get(name).map_or(false, |v| !v.is_empty())
    }

    fn start_adapter(&mut self, a: &Arc<dyn Adapter + Send + Sync + 'static>, cx: &Context) {
        let (tx, rx) = unbounded::<Arc<ErasedTopic>>();
        match a.start(cx, Arc::clone(&self.bus), rx) {
            Ok(handle) => {
                let idx = self.running.len();
                let name = a.name();
                let policy = a.policy();
                let topics = a.topics();
                self.running.push(RunningAdapter {
                    name,
                    policy,
                    topics,
                    tx,
                    handle,
                });

                self.by_name.entry(name).or_default().push(idx);
                for &t in topics {
                    self.by_topic.entry(t).or_default().push(idx);
                }

                debug!(self.logger, "▶ started adapter: {}", name);
            }
            Err(e) => error!(self.logger, "Failed to start adapter {}: {}", a.name(), e),
        }
    }

    fn start_where(
        &mut self,
        cx: &Context,
        mut pred: impl FnMut(&Arc<dyn Adapter + Send + Sync + 'static>) -> bool,
    ) {
        // clone Arcs first to avoid borrowing self across start calls
        let to_start: Vec<_> = self
            .registry
            .iter()
            .cloned()
            .filter(|a| pred(a))
            // avoid duplicate instances by name
            .filter(|a| !self.is_running_name(a.name()))
            .collect();

        for a in to_start {
            self.start_adapter(&a, cx);
        }
    }

    fn stop_where(&mut self, mut keep: impl FnMut(&RunningAdapter) -> bool) {
        // (unchanged) — rebuild indexes for kept entries
        let mut old = std::mem::take(&mut self.running);
        self.by_name.clear();
        self.by_topic.clear();

        let mut new_running: Vec<RunningAdapter> = Vec::with_capacity(old.len());
        for r in old.drain(..) {
            if keep(&r) {
                let idx = new_running.len();
                self.by_name.entry(r.name).or_default().push(idx);
                for &t in r.topics {
                    self.by_topic.entry(t).or_default().push(idx);
                }
                new_running.push(r);
            } else {
                r.handle.shutdown();
                debug!(self.logger, "■ stopped adapter: {}", r.name);
            }
        }
        self.running = new_running;
    }

    // ---- public control API --------------------------------------------

    pub(crate) fn start_all(&mut self, cx: &Context) {
        self.start_where(cx, |_| true);
    }

    pub(crate) fn stop_all(&mut self) {
        self.stop_where(|_| false);
    }

    pub(crate) fn restart_all(&mut self, cx: &Context) {
        self.stop_all();
        self.start_all(cx);
    }

    pub(crate) fn start_by_policy(&mut self, cx: &Context, policy: StartPolicy) {
        self.start_where(cx, |a| a.policy() == policy);
    }

    pub(crate) fn stop_by_policy(&mut self, policy: StartPolicy) {
        self.stop_where(|r| r.policy != policy);
    }

    pub(crate) fn restart_by_policy(&mut self, cx: &Context, policy: StartPolicy) {
        self.stop_by_policy(policy);
        self.start_by_policy(cx, policy);
    }

    pub(crate) fn start_by_name(&mut self, cx: &Context, name: &str) {
        self.start_where(cx, |a| a.name() == name);
    }

    pub(crate) fn stop_by_name(&mut self, name: &str) {
        self.stop_where(|r| r.name != name);
    }

    pub(crate) fn restart_by_name(&mut self, cx: &Context, name: &str) {
        self.stop_by_name(name);
        self.start_by_name(cx, name);
    }

    #[inline]
    fn has_topic(topics: &'static [&'static str], t: &str) -> bool {
        topics.iter().any(|&x| x == t)
    }

    pub(crate) fn start_by_topic(&mut self, cx: &Context, topic: &str) {
        self.start_where(cx, |a| a.topics().iter().any(|&t| t == topic));
    }

    pub(crate) fn stop_by_topic(&mut self, topic: &str) {
        self.stop_where(|r| !Self::has_topic(r.topics, topic));
    }

    pub(crate) fn restart_by_topic(&mut self, cx: &Context, topic: &str) {
        self.stop_by_topic(topic);
        self.start_by_topic(cx, topic);
    }

    // ---- notifications (you already had these) -------------------------

    pub(crate) fn notify_target(&self, target: AdapterTarget, note: Arc<ErasedTopic>) {
        match target {
            AdapterTarget::All => self.notify_all(note),
            AdapterTarget::Policy(p) => self.notify_policy(p, note),
            AdapterTarget::Topic(t) => self.notify_topic_name(t, note),
            AdapterTarget::Name(n) => self.notify_name(&n, note),
        }
    }

    pub(crate) fn notify_all(&self, note: Arc<ErasedTopic>) {
        for r in &self.running {
            let _ = r.tx.send(Arc::clone(&note));
        }
    }

    pub(crate) fn notify_policy(&self, policy: StartPolicy, note: Arc<ErasedTopic>) {
        for r in &self.running {
            if r.policy == policy {
                let _ = r.tx.send(Arc::clone(&note));
            }
        }
    }

    pub(crate) fn notify_name(&self, name: &str, note: Arc<ErasedTopic>) {
        if let Some(ixs) = self.by_name.get(name) {
            for &i in ixs {
                let _ = self.running[i].tx.send(Arc::clone(&note));
            }
        }
    }

    pub(crate) fn notify_topic_name(&self, topic_name: &str, note: Arc<ErasedTopic>) {
        if let Some(ixs) = self.by_topic.get(topic_name) {
            for &i in ixs {
                let _ = self.running[i].tx.send(Arc::clone(&note));
            }
        }
    }

    // ---- lifecycle with debounce ----

    /// Call when *any* target app launches.
    pub(crate) fn on_application_did_launch(&mut self, cx: &Context) {
        let was_zero = self.apps_up == 0;
        self.apps_up += 1;
        // if we had a pending stop, cancel it
        self.app_stop_due = None;
        if was_zero {
            self.start_where(cx, |a| matches!(a.policy(), StartPolicy::OnAppLaunch));
        }
    }

    /// Call when a target app terminates.
    pub(crate) fn on_application_did_terminate(&mut self) {
        if self.apps_up > 0 {
            self.apps_up -= 1;
        }
        if self.apps_up == 0 {
            // schedule a deferred stop
            self.app_stop_due = Some(Instant::now() + self.app_debounce);
            debug!(
                self.logger,
                "⏳ scheduling stop of OnAppLaunch adapters in {:?}", self.app_debounce
            );
        }
    }

    /// Drive deferred work; call this regularly from the runtime loop.
    pub(crate) fn tick(&mut self) {
        if let Some(due) = self.app_stop_due {
            if Instant::now() >= due && self.apps_up == 0 {
                self.stop_by_policy(StartPolicy::OnAppLaunch);
                self.app_stop_due = None;
                debug!(
                    self.logger,
                    "🛑 OnAppLaunch adapters stopped (no apps, debounced)"
                );
            }
        }
    }

    pub(crate) fn shutdown(mut self) {
        // Stop everything
        for r in self.running.drain(..) {
            r.handle.shutdown();
        }
        self.by_name.clear();
        self.by_topic.clear();
    }
}
