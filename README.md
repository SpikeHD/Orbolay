<div align="center">
  <h1>Orbolay</h1>
  <p>Quick, small, native Discord overlay alternative</p>
</div>

<div align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/SpikeHD/Orbolay/build.yml" />
  <img src="https://img.shields.io/github/repo-size/SpikeHD/Orbolay" />
  <img src="https://img.shields.io/github/commit-activity/m/SpikeHD/Orbolay" />
</div>

<div align="center">
  <img src="https://img.shields.io/github/release-date/SpikeHD/Orbolay" />
  <img src="https://img.shields.io/github/stars/SpikeHD/Orbolay" />
</div>

<div align="center">
  <img src="https://github.com/user-attachments/assets/d7adf8d3-96e5-4159-ac1d-7afb131e1fc1" />
</div>

# Features

* Voice channel status (whose in the call, whose speaking/not speaking/muted/etc.)
* Custom notifications
* ~~Mute/deafen/disconnect controls~~

# Compatibility

* **Windows**: 10 and 11 should work fine
* **MacOS**: Works, but no interactivity (no VC controls through overlay)
* **Linux**: X11 should be fine, Wayland works but cannot use always-on-top

# Installation

> [!NOTE]
> This will change. Orbolay is still in early stages!

1. Download the [latest actions build](https://github.com/SpikeHD/Orbolay/actions/workflows/build.yml).
2. Ensure you are using a compatible bridge plugin ([Shelter](https://github.com/SpikeHD/shelter-plugins?tab=readme-ov-file#orbolay-bridge))
3. Run the executable!

# TODO

* [x] Voice states
  * [x] Usernames
  * [x] Speaking/not speaking
  * [x] Muted/deafened icons beside name
  * [x] Proper avatar images
* [ ] Voice control
  * [ ] Mute/deafen buttons
* [x] Notifications
  * [x] Message notifications

## Stretch Goals

* [ ] Stream PiP
