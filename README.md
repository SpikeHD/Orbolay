<div align="center">
  <h1>Orbolay</h1>
  <p>Quick, small, native Discord overlay alternative</p>
</div>

<div align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/SpikeHD/Orbolay/build.yml" />
  <img src="https://img.shields.io/github/commit-activity/m/SpikeHD/Orbolay" />
  <img src="https://img.shields.io/github/release-date/SpikeHD/Orbolay" />
</div>

<div align="center">
  <a href="https://discord.gg/agQ9mRdHMZ"><img src="https://img.shields.io/discord/1128909403672821811?logo=discord&color=%20%235865F2" /></a>
  <img src="https://img.shields.io/github/stars/SpikeHD/Orbolay" />
</div>

<div align="center">
  <img src="https://github.com/user-attachments/assets/d7adf8d3-96e5-4159-ac1d-7afb131e1fc1" />
</div>

# Features

* Voice channel member list and status (who is speaking/muted/deafened/etc)
* Custom notifications
* Mute/deafen/disconnect controls
* Customizable visuals and layout
* Works with any modded client (including web!)

# Compatibility

* **Windows** - 10 and 11 both work, Windows 7 might work with kernel extensions
* **MacOS** - works, but cannot watch for keybinds (which means no voice controls)
* **Linux**
  * **X11** - should work fine, you may need to add your user to the `input` group
  * **Wayland** - technically works, you will want to use XWayland though (via `WAYLAND_DISPLAY="" orbolay`) and you may need to add your user to the `input` group

# Installation

## With Package Managers

> [!NOTE]
> Maintaining an `orbolay` package somewhere else? Let me know in an issue and I will add it here!

### Arch
```sh
yay -S orbolay-bin
```

### `cargo`
```sh
cargo install --locked --git https://github.com/SpikeHD/Orbolay.git
```

## Manual Installation

1. Download a [release](https://github.com/SpikeHD/Orbolay/releases) or the [latest actions build](https://github.com/SpikeHD/Orbolay/actions/workflows/build.yml).
2. Run the executable!

# How to Use

1. Run the executable
2. Install a compatible bridge plugin ([Shelter](https://github.com/SpikeHD/shelter-plugins?tab=readme-ov-file#orbolay-bridge) / [Vencord](https://github.com/SpikeHD/vc-orbolay-bridge), also available on [Equicord](https://github.com/Equicord/Equicord))
3. If you join a VC, it should show members in the corner
4. If you get a notification, it should show in the other corner
5. Press <kbd>Ctrl</kbd> + <kbd>`</kbd> to open the overlay and interact with voice controls

# Building

## Requirements

* Rust and Cargo

## Steps

1. Clone the repository
2. `cargo build --release`
3. Binaries will be in `target/release/`

# Special Thanks

* [Freya](https://github.com/marc2332/freya) - the main GUI library (that I may have fallen in love with)
* [Dioxus](https://dioxuslabs.com/) - framework for things like signals and `rsx!`
* Everyone else who contributes positively to the Rust ecosystem :)

# Contributing

PRs (especially for compatibility), polite issues, etc. are all welcome!
