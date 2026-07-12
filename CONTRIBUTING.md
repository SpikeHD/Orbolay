# Contributing

Thank you for considering making a change! It is much appreciated :)

# General Guidelines

1. Make an issue first! Unless it's trivial (ie. code quality), please open an issue first so your effort isn't wasted in the case of a direction change.
2. If you are looking to add a new control or interaction between Orbolay and the client, ensure it is an available [RPC command or event](https://docs.discord.food/topics/rpc#rpc-commands).
Even if you know you can make it work using the 3rd party plugins, to keep feature parity, I prefer project features stick to what's available to both sides, official and not.
I am open to exceptions, but please make an issue to discuss it!
4. Please ensure any new settings options are `#[cfg]`-gated, if needed. For example, the `XWayland` setting is gated to only show on Linux.
5. Always run `cargo fmt` and `cargo clippy` before commits, the CI catches it too but I will likely ask you to fix them before merge.
6. If using LLM assistance, PLEASE review the output thoroughly. The more slop PRs I get, the more trigger happy to close them I will become.

# Project Setup

The project is extremely simple to set up for development, the only requirements are:

1. The [libraries required for Freya](https://docs.rs/freya/0.4.0-rc.24/freya/_docs/development_setup/)
2. [Rust and Cargo](https://rust-lang.org/tools/install/)

Running the project is as simple as:

```sh
git clone https://github.com/SpikeHD/Orbolay.git
cd Orbolay
cargo run
```

# High-level Architecture

Orbolay is a 2-part system. There is the UI "layer" and the transport "layer". A global `AppState` is used to bridge these two concepts together.

## The UI Layer

Unlike other overlay programs like the official overlay or mangohud (which inject into the rendering pipeline of a running application), Orbolay works by running a fullscreen,
transparent window on top of everything else. The key benefit here being that Orbolay does not depend on supporting a bunch of different graphics APIs, and can even run
without any other program running at all! On Windows and MacOS this kinda just works, but on Linux it requires running via `xwayland` if using Wayland.
This will likely change when `winit` supports `layer-shell`.

The UI, located in `crates/orbolay-ui`, is nothing special. Freya is used to create some custom elements that we display in the window. Like something akin to React, when the
global `AppState` changes, the UI will update along with it. Reading the code for these is probably the easiest way to get aquainted with them.

## The Transport Layer

Orbolay contains two transport methods, IPC and websocket. Both can be found in `crates/orbolay-transport`.

### IPC

IPC is used to interface with official clients, as Discord exposes a socket on the filesystem for us to query and send commands to. Orbolay pretends to be StreamKit in order to get access to most
commands and events.

IPC works by subscribing to events and sending commands. For example, we subscribe to `VOICE_STATE_UPDATE` to see the changes in a user in the voice channel (such as, if they mute themselves),
or we can send the command `SET_VOICE_SETTINGS` with `{ mute: true }` to mute ourselves.

All available commands and events can be found on [docs.discord.food](https://docs.discord.food/topics/rpc).

### Websocket

Websocket is used to communicate with 3rd party clients running Orbolay bridge plugins, like the [shelter plugin](https://github.com/SpikeHD/shelter-plugins#orbolay-bridge)
or the [Vencord](https://github.com/SpikeHD/vc-orbolay-bridge)/[Equicord](https://github.com/equicord/equicord) plugins.

Because we have full control over these, there is no need to subscribe to events, the plugins are just written to send them by listening to Flux events on the client. We also have access to
more client data than RPC, since we have full control over the client via the bridge plugins.
