<!-- Improved compatibility of back to top link -->

<a id="readme-top"></a>

<!-- PROJECT SHIELDS -->

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![project\_license][license-shield]][license-url]

<!-- PROJECT LOGO -->

<!-- <br /> -->
<div align="center">
  <!-- <a href="https://github.com/veelume/streamdeck-lib">
    <img src="images/logo.png" alt="Logo" width="80" height="80">
  </a> -->

<h3 align="center">streamdeck-lib</h3>

  <p align="center">
    A Rust library for building Elgato Stream Deck plugins
    <br />
    <a href="https://github.com/veelume/streamdeck-lib"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/veelume/streamdeck-lib">View Demo</a>
    &middot;
    <a href="https://github.com/veelume/streamdeck-lib/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    &middot;
    <a href="https://github.com/veelume/streamdeck-lib/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>

<!-- TABLE OF CONTENTS -->

<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <!-- <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul> -->
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>

<!-- ABOUT THE PROJECT -->

## About The Project

<!-- [![Product Screenshot][product-screenshot]](https://example.com) -->

**streamdeck-lib** is a Rust library to simplify writing Elgato Stream Deck plugins.
It provides a structured runtime, typed events, and a cross-platform input system (with Windows `SendInput` support).

Key features:

* Typed Stream Deck event handling (`serde`-powered)
* Plugin lifecycle management (actions, adapters, hooks)
* Keyboard and mouse input synthesis
* Global + per-action hooks
* Custom background code with adapters

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ### Built With

* [![Rust][Rust-shield]][Rust-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p> -->

<!-- GETTING STARTED -->

## Getting Started

### Prerequisites

* [Rust](https://www.rust-lang.org/tools/install) (edition 2024+)

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
streamdeck-lib = { git = "https://github.com/veelume/streamdeck-lib" }
```

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- USAGE EXAMPLES -->

## Usage

Minimal plugin (single file example)
```rust
use std::sync::Arc;
use streamdeck_lib::prelude::*;

const PLUGIN_ID: &str = "com.example.hello";

// --- a simple action ------------------------------------------------------
#[derive(Default)]
struct HelloAction;

impl ActionStatic for HelloAction { const ID: &'static str = "com.example.hello.action"; }
impl Action for HelloAction {
    fn id(&self) -> &str { Self::ID }
    fn key_down(&mut self, cx: &Context, ev: &views::KeyDown) {
        info!(cx.log(), "Hello from {} in {}!", Self::ID, ev.context);
        cx.sd().set_title_simple(ev.context, "👋");
        cx.sd().show_ok(ev.context);
    }
}

fn main() -> anyhow::Result<()> {
    // ----- logging -----
    let logger: Arc<dyn ActionLog> = Arc::new(FileLogger::from_appdata(PLUGIN_ID)?);

    // ----- parse launch args from Stream Deck -----
    let args = parse_launch_args().map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // ----- optional hooks -----
    let hooks = AppHooks::default().add(|cx, ev| {
        if let HookEvent::Init = ev { info!(cx.log(), "plugin init: {}", PLUGIN_ID); }
    });

    // ----- build plugin -----
    let plugin = PluginBuilder::new()
        .set_hooks(hooks)
        .add_action(ActionFactory::default_of::<HelloAction>())
        .build()?;

    // ----- run runtime -----
    let cfg = RunConfig::default().set_log_websocket(false);
    run(plugin, args, logger, cfg)?;
    Ok(())
}
```

Actions — minimal + reacting to events
```rust
use streamdeck_lib::prelude::*;

const PLUGIN_ID: &str = "com.example.demo";

#[derive(Default)]
struct HelloAction;

impl ActionStatic for HelloAction { const ID: &'static str = "com.example.demo.hello"; }
impl Action for HelloAction {
    fn id(&self) -> &str { Self::ID }

    // show a title + toast on press
    fn key_down(&mut self, cx: &Context, ev: &views::KeyDown) {
        cx.sd().set_title_simple(ev.context, "👋");
        cx.sd().show_ok(ev.context);
        info!(cx.log(), "HelloAction on {}", ev.context);
    }

    // react to a global SD event (e.g., device changes)
    fn on_global_event(&mut self, cx: &Context, ev: &StreamDeckEvent) {
        debug!(cx.log(), "global event for {}: {}", Self::ID, ev);
    }
}
```

Hooks — observe runtime & SD lifecycle
```rust
use streamdeck_lib::prelude::*;

let hooks = AppHooks::default()
    .add(|cx, ev| match ev {
        HookEvent::Init => info!(cx.log(), "init"),
        HookEvent::Exit => info!(cx.log(), "exit"),
        HookEvent::ApplicationDidLaunch(app) => info!(cx.log(), "app launched: {app}"),
        HookEvent::Outgoing(msg) => debug!(cx.log(), "→ SD: {msg:?}"),
        HookEvent::Log(level, msg) => debug!(cx.log(), "[{level}] {msg}"),
        _ => {}
    });
```

Notifications — define topics + send + receive (typed)
```rust
use streamdeck_lib::prelude::*;

// 1) Define topics
pub const DEMO_PING: TopicId<Ping> = TopicId::new("demo.ping");
#[derive(Debug, Clone)]
pub struct Ping { pub msg: String }

// 2) Send from anywhere with a Context
fn send_ping(cx: &Context) {
    // to all adapters with topic
    cx.bus().adapters_notify_topic_t(DEMO_PING, None, Ping { msg: "hello".into() });
    // to all actions with topic
    cx.bus().action_notify_topic_t(DEMO_PING, None, Ping { msg: "hello".into() })
}

// 3) Receive in an Action (subscribe via topics())
#[derive(Default)]
struct NotifiedAction;
impl ActionStatic for NotifiedAction { const ID: &'static str = "com.example.NotifiedAction"; }
impl Action for NotifiedAction {
    fn id(&self) -> &str { Self::ID }
    fn topics(&self) -> &'static [&'static str] { &[DEMO_PING.name] }
    fn on_notify(&mut self, cx: &Context, ctx: &str, ev: &ErasedTopic) {
        if let Some(p) = ev.downcast(DEMO_PING) {
            info!(cx.log(), "Action got Ping in ctx={ctx}: {}", p.msg);
        }
    }
}
```

Adapters — small worker that listens for a topic
```rust
use std::sync::Arc;
use streamdeck_lib::prelude::*;

#[derive(Default)]
struct DemoAdapter;
impl AdapterStatic for DemoAdapter { const NAME: &'static str = "demo.adapter"; }

impl Adapter for DemoAdapter {
    fn name(&self) -> &'static str { Self::NAME }
    fn policy(&self) -> StartPolicy { StartPolicy::Eager }
    fn topics(&self) -> &'static [&'static str] { &[DEMO_PING.name] }

    fn start(
        &self,
        cx: &Context,
        bus: Arc<dyn Bus>,
        inbox: crossbeam_channel::Receiver<Arc<ErasedTopic>>,
    ) -> Result<AdapterHandle, String> {
        let log = cx.log().clone();
        let (tx_stop, rx_stop) = crossbeam_channel::bounded::<()>(1);

        let join = std::thread::spawn(move || {
            info!(log, "DemoAdapter started");
            loop {
                crossbeam_channel::select! {
                    recv(inbox) -> msg => match msg {
                        Ok(ev) => {
                            if let Some(p) = ev.downcast(DEMO_PING) {
                                info!(log, "Adapter got Ping: {}", p.msg);
                                // echo to all actions listening on this topic:
                                bus.action_notify_topic_t(DEMO_PING, ev.context.clone(), p.clone());
                            }
                        }
                        Err(_) => break,
                    },
                    recv(rx_stop) -> _ => break,
                }
            }
            info!(log, "DemoAdapter stopped");
        });

        Ok(AdapterHandle::from_crossbeam(join, tx_stop))
    }
}
```

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ROADMAP -->

## Roadmap

See the [open issues](https://github.com/veelume/streamdeck-lib/issues) for the full roadmap.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- CONTRIBUTING -->

## Contributing

Contributions make the open source community amazing!
Fork the repo, create a feature branch, and open a PR 🚀

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit (`git commit -m 'Add some AmazingFeature'`)
4. Push (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- LICENSE -->

## License

Distributed under the MIT OR Apache-2.0 License.
See [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- CONTACT -->

## Contact

Project Link: [https://github.com/veelume/streamdeck-lib](https://github.com/veelume/streamdeck-lib)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ACKNOWLEDGMENTS -->

## Acknowledgments

* [Elgato Stream Deck SDK](https://docs.elgato.com/streamdeck/sdk/references/websocket/plugin)
* [Rust language](https://www.rust-lang.org/)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- MARKDOWN LINKS & IMAGES -->

[contributors-shield]: https://img.shields.io/github/contributors/veelume/streamdeck-lib.svg?style=for-the-badge
[contributors-url]: https://github.com/veelume/streamdeck-lib/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/veelume/streamdeck-lib.svg?style=for-the-badge
[forks-url]: https://github.com/veelume/streamdeck-lib/network/members
[stars-shield]: https://img.shields.io/github/stars/veelume/streamdeck-lib.svg?style=for-the-badge
[stars-url]: https://github.com/veelume/streamdeck-lib/stargazers
[issues-shield]: https://img.shields.io/github/issues/veelume/streamdeck-lib.svg?style=for-the-badge
[issues-url]: https://github.com/veelume/streamdeck-lib/issues
[license-shield]: https://img.shields.io/github/license/veelume/streamdeck-lib.svg?style=for-the-badge
[license-url]: https://github.com/veelume/streamdeck-lib/blob/main/LICENSE-MIT
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/yourlinkedin
[product-screenshot]: images/screenshot.png
[Rust-shield]: https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white
[Rust-url]: https://www.rust-lang.org/
