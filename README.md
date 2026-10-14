# Apelles

<p align="center">
  <a href="https://github.com/Paplo-painite/apelles">
    <img alt="Apelles" src="https://raw.githubusercontent.com/Paplo-painite/apelles/main/assets/app-icon/hicolor/512x512/apps/ai.storyteller.apelles.png" width="128" />
  </a>
</p>

<p align="center">
  <strong>Image editing, rebuilt in pure Rust.</strong><br>
  A clean-room, native reimplementation of core Photoshop workflows for desktop, Android web, and browser builds.
</p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-100%25-orange?logo=rust&logoColor=white">
  <img alt="Platform" src="https://img.shields.io/badge/macOS%20%7C%20Windows%20%7C%20Linux%20%7C%20FreeBSD%20%7C%20Web-native-blue">
  <img alt="License" src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-lightgrey">
  <img alt="Status" src="https://img.shields.io/badge/Status-early%20alpha-yellow">
</p>

<p align="center">
  <a href="https://github.com/Paplo-painite/apelles/releases">Releases</a> ·
  <a href="https://github.com/Paplo-painite/apelles/discussions">Discussions</a> ·
  <a href="https://github.com/Paplo-painite/apelles/tree/main/docs">Docs</a> ·
  <a href="https://github.com/Paplo-painite/apelles/issues">Issues</a>
</p>

## Quick start

### Download the app

- Android testing build: <a href="https://github.com/Paplo-painite/apelles/releases/tag/v0.6.0-apelles-android">Download APK</a>
- Web demo: <a href="https://getartcraft.com/apps/apelles">Open the app</a>
- GitHub releases: <a href="https://github.com/Paplo-painite/apelles/releases">Latest builds</a>

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
- Native desktop app with browser/WebAssembly support
- Clean-room engine built in Rust

## Repository layout

```text
apps/           Desktop, CLI, web, and Android-related app targets
crates/         Core Rust crates for image editing, PSD IO, text, rendering, and UI logic
assets/         Icons, fonts, and bundled assets
book/           Documentation book
docs/           Project docs and design notes
packaging/      Platform packaging and installer scripts
```

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
