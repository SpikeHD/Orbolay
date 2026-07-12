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
