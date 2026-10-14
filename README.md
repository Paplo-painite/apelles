# Apelles

<p align="center">
  <a href="https://github.com/Paplo-painite/apelles">
    <img alt="Apelles Android" src="https://raw.githubusercontent.com/Paplo-painite/apelles/main/docs/images/new_icon.png" width="128" />
  </a>
</p>

<p align="center">
  <strong>Image editing, rebuilt in pure Rust.</strong><br>
  Apelles: A powerful mobile-first image editor for Android, built from scratch using Rust.
</p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-100%25-orange?logo=rust&logoColor=white">
  <img alt="Platform" src="https://img.shields.io/badge/Platform-Android-brightgreen?logo=android&logoColor=white">
  <img alt="License" src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-lightgrey">
  <img alt="Status" src="https://img.shields.io/badge/Status-early%20alpha-yellow">
</p>

<p align="center">
  <a href="https://github.com/Paplo-painite/apelles/releases">Releases</a> ·
  <a href="https://github.com/Paplo-painite/apelles/discussions">Discussions</a> ·
  <a href="https://github.com/Paplo-painite/apelles/tree/main/docs">Docs</a> ·
  <a href="https://github.com/Paplo-painite/apelles/issues">Issues</a> ·
  <a href="https://github.com/Paplo-painite/apelles/blob/main/CONTRIBUTING.md">Contributing</a>
</p>

## Quick start

### Download the app

- Android testing build: <a href="https://github.com/Paplo-painite/apelles/releases/tag/v0.6.0-apelles-android">Download APK</a>
- Web demo: <a href="https://getartcraft.com/apps/apelles">Open the app</a>
- GitHub releases: <a href="https://github.com/Paplo-painite/apelles/releases">Latest builds</a>

## Download Apelles for Android

<p align="center">
  <img src="docs/images/android_screenshot.jpg" alt="Apelles Android app editing an ocean waves painting" width="360">
  <br>
  <sub>Apelles running on Android with a mobile-first editing interface for painting, layers, tools, and export.</sub>
</p>

<p align="center">
  <img src="docs/images/apelles-brush-android.jpg" alt="Apelles Android brush tool — brush settings and on-canvas preview" width="360">
  <br>
  <sub>Brush tool on Android: the Brush settings panel, size/opacity controls and an on-canvas preview while painting, shown inside the mobile UI.</sub>
</p>

> The APK supports `arm64-v8a` and `x86_64`. It is a **pre-release testing build**; enable installation from the relevant source on Android before installing it.

### Build from source

```bash
git clone https://github.com/Paplo-painite/apelles.git
cd apelles
cargo run --release -p apelles -- --help
```

### Android / PWA build

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cd apps/apelles-web
trunk build --release
```

Then serve the generated files from `dist/web/` over HTTPS and open them on Android in Chrome.

## What Apelles includes

- Layer-based editing workflows
- Real PSD compatibility and layered document support
- Adjustment layers, masks, type, vector shapes, and filters
- Brush engine and live editing for common Photoshop-style operations
- Native Android app with a mobile-first interface
- Clean-room engine built in Rust

## Repository layout

```text
apps/           Android app and supporting web/PWA targets
crates/         Core Rust crates for image editing, PSD IO, text, rendering, and UI logic
assets/         Icons, fonts, and bundled assets
book/           Documentation book
docs/           Project docs and design notes
packaging/      Platform packaging and installer scripts
```

## Contributing

We welcome contributions from developers, designers, testers, and artists.

Before you start:

- Read [`AGENTS.md`](AGENTS.md)
- Check the current issues and discussions
- Prefer small, focused pull requests

Suggested workflow:

```bash
git clone https://github.com/Paplo-painite/apelles.git
cd apelles
cargo test --workspace
```

If you are working on a feature or bug fix, try to keep the change scoped and testable.

## Developer notes

- Full project docs: `book/`
- App and packaging instructions: `packaging/`
- Android web build notes: `android/README.md`
- Contributor guidance: `AGENTS.md`

## Community and support

- Discord: https://discord.gg/artcraft
- Website: https://getartcraft.com/
- App page: https://getartcraft.com/apps/apelles

## License

Apelles is dual-licensed under the MIT and Apache-2.0 licenses.

See:
- `LICENSE-MIT`
- `LICENSE-APACHE`
- `NOTICE`
- `ATTRIBUTION.md`

## Star history

[![Star History Chart](https://api.star-history.com/svg?repos=Paplo-painite%2Fapelles&type=Date&legend=top-left)](https://www.star-history.com/?repos=Paplo-painite%2Fapelles&type=date&legend=top-left)
