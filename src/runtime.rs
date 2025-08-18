// runtime.rs
use std::{
    collections::VecDeque,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crossbeam_channel::{select, unbounded};
use websocket::{ClientBuilder, OwnedMessage};

use crate::{
    action_manager::{ActionManager, dispatch},
    adapters_manager::AdapterManager,
    bus::Emitter,
    debug, error,
    events::{AdapterControl, AdapterTarget, RuntimeMsg},
    hooks::AppHooks,
    info,
    launch::{LaunchArgs, RunConfig},
    logger::{ActionLog, Level},
    plugin_builder::Plugin,
    sd_protocol::{self, SdClient, StreamDeckEvent, parse_incoming_owned, serialize_outgoing},
    warn,
};

fn drain_outgoing(
    outq: &mut VecDeque<sd_protocol::Outgoing>,
    writer: &Arc<Mutex<websocket::sender::Writer<std::net::TcpStream>>>,
    logger: &Arc<dyn ActionLog>,
) {
    const DRAIN_PER_TICK: usize = 8;
    for _ in 0..DRAIN_PER_TICK {
        let Some(msg) = outq.pop_front() else {
            break;
        };
        match serialize_outgoing(&msg) {
            Ok(text) => {
                if let Ok(mut w) = writer.lock() {
                    if let Err(e) = w.send_message(&OwnedMessage::Text(text)) {
                        error!(logger, "❌ websocket send: {:?}", e);
                        outq.push_front(msg);
                        break;
                    }
                } else {
                    error!(logger, "❌ writer mutex poisoned");
                    outq.push_front(msg);
                    break;
                }
            }
            Err(e) => error!(logger, "❌ serialize outgoing: {:?}", e),
        }
    }
}

/// Run the plugin runtime (non-generic; notifications handled dynamically if you need them).
pub fn run(
    plugin: Plugin,
    args: LaunchArgs,
    logger: Arc<dyn ActionLog>,
    cfg: RunConfig,
) -> anyhow::Result<()> {
    // ---------- connect ----------
    let url = (cfg.url_fn)(args.port);
    info!(logger, "🔗 connecting websocket: {}", url);

    let client = ClientBuilder::new(&url)?.connect_insecure()?;
    let (mut reader, writer_raw) = client.split()?;
    let writer = Arc::new(Mutex::new(writer_raw));

    // ---------- single bus for everything ----------
    let (rt_tx, rt_rx) = unbounded::<RuntimeMsg>();

    // ---------- sd client + context ----------
    // SdClient should internally do: tx.send(RuntimeMsg::Outgoing(...))
    let sd = Arc::new(SdClient::new(
        rt_tx.clone(),
        args.plugin_uuid.clone(),
        logger.clone(),
        cfg.log_websocket,
    ));

    let emitter = Emitter::new(rt_tx.clone());
    let bus = Arc::new(emitter);

    // Now build the Context with enriched Extensions
    let cx = plugin.make_context(
        Arc::clone(&sd),
        Arc::clone(&logger),
        args.plugin_uuid.clone(),
        plugin.exts(),
        bus,
    );

    // ---------- register with Stream Deck ----------
    {
        let register_msg = serde_json::json!({
            "event": args.register_event,
            "uuid": args.plugin_uuid
        });
        writer
            .lock()
            .map_err(|_| anyhow::anyhow!("writer mutex poisoned"))?
            .send_message(&OwnedMessage::Text(register_msg.to_string()))?;
    }
    info!(logger, "✅ registered: {}", args.plugin_uuid);

    // ---------- fire init hooks ----------
    plugin.hooks().fire_init(&cx);

    cx.sd().get_global_settings();

    // ---------- reader thread (websocket -> RuntimeMsg::Incoming) ----------
    {
        let logger = Arc::clone(&logger);
        let tx = rt_tx.clone();
        let writer_for_reader = Arc::clone(&writer);

        // Helper to avoid log spam with huge frames
        #[inline]
        fn truncate_for_log(s: &str, max: usize) -> &str {
            if s.len() <= max {
                s
            } else {
                s.get(..max).unwrap_or(s)
            }
        }

        thread::spawn(move || {
            if let Err(p) = catch_unwind(AssertUnwindSafe(|| {
                for incoming in reader.incoming_messages() {
                    match incoming {
                        Ok(OwnedMessage::Text(text)) => {
                            // Parse ONCE, move out of the Map without cloning.
                            let parsed = serde_json::from_str::<
                                serde_json::Map<String, serde_json::Value>,
                            >(&text)
                            .map_err(|e| format!("json parse error: {e}"))
                            .and_then(parse_incoming_owned);

                            match parsed {
                                Ok(ev) => {
                                    if cfg.log_websocket {
                                        debug!(logger, "📥 WebSocket incoming: {:#?}", ev);
                                        // No re-parse: log the raw string (truncated)
                                        debug!(
                                            logger,
                                            "📥 WebSocket raw: {}",
                                            truncate_for_log(&text, 4096)
                                        );
                                    }
                                    let _ = tx.send(RuntimeMsg::Incoming(ev));
                                }
                                Err(err) => {
                                    // Keep raw on failures (truncated)
                                    warn!(
                                        logger,
                                        "⚠️ unrecognized SD event: {} | raw = {}",
                                        err,
                                        truncate_for_log(&text, 4096)
                                    );
                                }
                            }
                        }
                        Ok(OwnedMessage::Close(_)) => {
                            debug!(logger, "🔌 websocket close received");
                            let _ = tx.send(RuntimeMsg::Exit);
                            break;
                        }
                        Ok(OwnedMessage::Ping(payload)) => {
                            if let Ok(mut w) = writer_for_reader.lock() {
                                let _ = w.send_message(&OwnedMessage::Pong(payload));
                            }
                            debug!(logger, "🔄 websocket ping received");
                        }
                        Ok(OwnedMessage::Pong(_)) => {
                            debug!(logger, "🔄 websocket pong received");
                        }
                        Ok(OwnedMessage::Binary(_)) => {
                            // If you want, handle Binary similarly (see commented code above)
                            warn!(logger, "⚠️ unrecognized binary message");
                        }
                        Err(e) => {
                            error!(logger, "❌ websocket read: {:?}", e);
                            break;
                        }
                    }
                }
            })) {
                error!(logger, "❌ reader thread panicked: {:?}", p);
            }
        });
    }

    // ---------- adapters ----------
    let mut adapter_mgr = AdapterManager::new(plugin.adapters(), cx.bus(), Arc::clone(&logger));

    // Start adapters with Eager policy right away
    adapter_mgr.start_by_policy(&cx, crate::adapters::StartPolicy::Eager);
    // ---------- hooks + action manager ----------
    let hooks: AppHooks = plugin.hooks().clone();
    let mut mgr: ActionManager = ActionManager::new(plugin.actions().clone());

    // ---------- tiny burst buffer for outgoing ----------
    let mut outq: VecDeque<sd_protocol::Outgoing> = VecDeque::new();

    // ---------- main loop ----------
    use RuntimeMsg::*;
    loop {
        select! {
            recv(rt_rx) -> msg => {
                match msg {
                    // ---------- incoming SD events ----------
                    Ok(Incoming(ev)) => {
                        hooks.fire_incoming(&cx, &ev);

                        // fire hooks and adapters
                        match &ev {
                            StreamDeckEvent::ApplicationDidLaunch { application } => {
                                adapter_mgr.on_application_did_launch(&cx);
                                hooks.fire_application_did_launch(&cx, application);
                            }
                            StreamDeckEvent::ApplicationDidTerminate { application } => {
                                adapter_mgr.on_application_did_terminate();
                                hooks.fire_application_did_terminate(&cx, application);
                            }
                            StreamDeckEvent::DeviceDidConnect { device, device_info } => {
                                hooks.fire_device_did_connect(&cx, device, device_info);
                            }
                            StreamDeckEvent::DeviceDidDisconnect { device } => {
                                hooks.fire_device_did_disconnect(&cx, device);
                            }
                            StreamDeckEvent::DeviceDidChange { device, device_info } => {
                                hooks.fire_device_did_change(&cx, device, device_info);
                            }
                            StreamDeckEvent::DidReceiveDeepLink { url } => {
                                hooks.fire_did_receive_deep_link(&cx, url);
                            }
                            StreamDeckEvent::DidReceiveGlobalSettings { settings } => {
                                cx.globals().hydrate_from_sd(settings.clone());
                                hooks.fire_did_receive_global_settings(&cx, settings);
                            }
                            _ => {}
                        }

                        // dispatch to actions
                        dispatch(&mut mgr, &cx, &plugin, ev);
                    }

                    // ---------- outgoing SD messages ----------
                    Ok(Outgoing(msg)) => {
                        hooks.fire_outgoing(&cx, &msg);
                        let was_empty = outq.is_empty();
                        outq.push_back(msg);
                        if was_empty {
                            // quick flush, more than 8 messages per tick is unlikely
                            drain_outgoing(&mut outq, &writer, &logger);
                        }
                    }

                    // ---------- logs ----------
                    Ok(Log { msg, level }) => {
                        hooks.fire_log(&cx, level, &msg);
                        match level {
                            Level::Debug => debug!(logger, "{}", msg),
                            Level::Info  => info!(logger,  "{}", msg),
                            Level::Warn  => warn!(logger,  "{}", msg),
                            Level::Error => error!(logger, "{}", msg),
                        }
                    }

                    // ---------- typed action notify ----------
                    Ok(ActionNotify { target, event }) => {
                        // let hooks see target + topic
                        hooks.fire_action_notify(&cx, &event);
                        // fan-out by target (All / Context / Id / Topic)
                        mgr.notify_target(&cx, target, event);
                    }

                    // ---------- typed adapter notify ----------
                    Ok(AdapterNotify { target, event }) => {
                        hooks.fire_adapter_notify(&cx, &target, event.as_ref());
                        // fan-out by target (All / Policy / Name / Topic)
                        adapter_mgr.notify_target(target, event);
                    }

                    // ---------- adapter control ----------
                    Ok(RuntimeMsg::Adapter(ctl)) => {
                        hooks.fire_adapter_control(&cx, &ctl);
                        match ctl {
                            AdapterControl::Start(target) => match target {
                                AdapterTarget::All         => adapter_mgr.start_all(&cx),
                                AdapterTarget::Policy(p)   => adapter_mgr.start_by_policy(&cx, p),
                                AdapterTarget::Name(n)     => adapter_mgr.start_by_name(&cx, n),
                                AdapterTarget::Topic(t)    => adapter_mgr.start_by_topic(&cx, t),
                            },
                            AdapterControl::Stop(target) => match target {
                                AdapterTarget::All         => adapter_mgr.stop_all(),
                                AdapterTarget::Policy(p)   => adapter_mgr.stop_by_policy(p),
                                AdapterTarget::Name(n)     => adapter_mgr.stop_by_name(n),
                                AdapterTarget::Topic(t)    => adapter_mgr.stop_by_topic(t),
                            },
                            AdapterControl::Restart(target) => match target {
                                AdapterTarget::All         => adapter_mgr.restart_all(&cx),
                                AdapterTarget::Policy(p)   => adapter_mgr.restart_by_policy(&cx, p),
                                AdapterTarget::Name(n)     => adapter_mgr.restart_by_name(&cx, n),
                                AdapterTarget::Topic(t)    => adapter_mgr.restart_by_topic(&cx, t),
                            },
                        }
                    }

                    // ---------- exit ----------
                    Ok(Exit) => {
                        hooks.fire_exit(&cx);
                        info!(logger, "🔚 runtime exit requested");
                        break;
                    }

                    Err(err) => {
                        error!(logger, "❌ runtime channel error: {:?}", err);
                        break;
                    }, // bus closed
                }
            }

            default(Duration::from_millis(100)) => {
                drain_outgoing(&mut outq, &writer, &logger);
                hooks.fire_tick(&cx);
                adapter_mgr.tick();
            }
        }
    }

    // ---------- shutdown ----------
    adapter_mgr.shutdown();

    info!(logger, "🔚 runtime shutdown complete");

    Ok(())
}
