# Apelles

<p align="center">
  <a href="https://github.com/Paplo-painite/apelles">
    <img alt="Apelles Android" src="https://raw.githubusercontent.com/Paplo-painite/apelles/main/docs/images/new_icon.png" width="128" />
  </a>
</p>

<p align="center">
  <strong>Image editing, rebuilt in pure Rust.</strong><br>
  Apelles: a powerful mobile-first image editor for Android, built from scratch using Rust.
</p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-100%25-orange?logo=rust&logoColor=white">
  <img alt="Platform" src="https://img.shields.io/badge/Platform-Android-brightgreen?logo=android&logoColor=white">
  <img alt="License" src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-lightgrey">
  <img alt="Status" src="https://img.shields.io/badge/Status-Android%20pre--release-yellow">
</p>

<p align="center">
  <a href="https://github.com/Paplo-painite/apelles/releases/latest">Download APK</a> ·
  <a href="https://github.com/Paplo-painite/apelles/discussions">Discussions</a> ·
  <a href="https://github.com/Paplo-painite/apelles/tree/main/docs">Docs</a> ·
  <a href="https://github.com/Paplo-painite/apelles/issues">Issues</a> ·
  <a href="https://github.com/Paplo-painite/apelles/blob/main/CONTRIBUTING.md">Contributing</a>
</p>

## Download Apelles for Android

[Download the latest APK from GitHub Releases](https://github.com/Paplo-painite/apelles/releases/latest)

The current Android release is a **pre-release testing build**. It supports `arm64-v8a` and
`x86_64`; enable installation from the relevant source on Android before installing it. Check the
release notes for the supported Android version and verify the published SHA-256 digest when
available.

<p align="center">
  <img src="docs/images/android_screenshot.jpg" alt="Apelles Android app editing an ocean waves painting" width="360">
  <br>
  <sub>Apelles running on Android with a mobile-first editing interface for painting, layers, tools, and export.</sub>
</p>

<p align="center">
  <img src="docs/images/apelles-brush-android.jpg" alt="Apelles Android brush tool — brush settings and on-canvas preview" width="360">
  <br>
  <sub>Brush settings, size and opacity controls, and an on-canvas preview inside the Android UI.</sub>
</p>

## Android status

> **Apelles Android is an early, ambitious pre-release — ready for exploration, testing, and
> collaboration, but not yet a fully stable production application.**

Apelles is being built as a native Android application, not a WebView. The current build already
brings together a Rust-powered editing engine, layered workflows, Android Storage Access Framework
file selection, project save/export, autosave and recovery, touch-oriented scaling, Arabic/RTL text
support, and imported font support. These foundations are an exciting starting point, while device
compatibility, performance, advanced editing workflows, and long-session reliability continue to
improve.

### Help build the next version

Apelles needs Android developers, Rust developers, UI/UX designers, testers, Arabic typography
specialists, and creative users who can try real workflows and report what works well and what
needs attention. Your feedback and contributions can help turn this promising first Android build
into a dependable everyday image editor. Please use the issue templates when reporting Android,
Arabic/RTL, or feature-specific feedback.

## What Apelles includes

- Layer-based editing workflows
- Real PSD compatibility and layered document support
- Adjustment layers, masks, type, vector shapes, and filters
- Brush engine and live editing for common Photoshop-style operations
- Native Android app with a mobile-first interface
- Offline-first editing and local crash recovery
- Arabic and RTL text shaping with imported font support
- Clean-room engine built in Rust

## Privacy

Apelles works offline and does not require the Android `INTERNET` permission. Images, projects,
recovery data, and imported fonts remain on the device unless you explicitly export or share them.
The APK does not upload your images to a remote service.

## Quick start

### Build from source

```bash
git clone https://github.com/Paplo-painite/apelles.git
cd apelles
cargo run --release -p apelles -- --help
```

### Build the Android app

See [`README_ANDROID.md`](README_ANDROID.md) and [`android/README.md`](android/README.md) for
Android SDK, NDK, Rust target, and packaging instructions.

```bash
./packaging/android/build-native.sh
```

The build produces an APK for `arm64-v8a` and `x86_64` under
`packaging/android/app/build/outputs/apk/release/`.

## Repository layout

```text
apps/           Android app and supporting web/PWA targets
crates/         Core Rust crates for image editing, PSD IO, text, rendering, and UI logic
assets/         Icons, fonts, and bundled assets
book/           Documentation book
docs/           Project docs and design notes
packaging/      Android packaging and release scripts
```

## Contributing

We welcome contributions from developers, designers, testers, and artists.

Before you start:

- Read [`AGENTS.md`](AGENTS.md)
- Check the current issues and discussions
- Prefer small, focused pull requests
- For Android bugs, include the device model, Android version, ABI, and reproduction steps

Suggested workflow:

```bash
git clone https://github.com/Paplo-painite/apelles.git
cd apelles
cargo test --workspace
```

If you are working on a feature or bug fix, try to keep the change scoped and testable.

## Developer notes

- Android app and packaging instructions: [`README_ANDROID.md`](README_ANDROID.md)
- Android build notes: [`android/README.md`](android/README.md)
- Full project docs: [`book/`](book/)
- Contributor guidance: [`AGENTS.md`](AGENTS.md)
- Release history: [`CHANGELOG.md`](CHANGELOG.md)

## Community and support

- Maintainer: [Paplo-painite](https://github.com/Paplo-painite)
- Discord: https://discord.gg/artcraft
- App page: https://getartcraft.com/apps/apelles

## Development assistance

This Android port and its release preparation were developed by **Paplo-painite** with
development assistance from [Manus AI](https://manus.im). Manus AI helped with Android
integration, offline-first workflows, Arabic/RTL support, documentation, testing, and release
packaging. The project remains community-owned and open source, and all contributions are welcome.

If you would like to use the AI development assistant that helped with this project, visit
[Manus AI](https://manus.im).

## License

Apelles is dual-licensed under the MIT and Apache-2.0 licenses.

See:

- `LICENSE-MIT`
- `LICENSE-APACHE`
- `NOTICE`
- `ATTRIBUTION.md`

## Star history

[![Star History Chart](https://api.star-history.com/svg?repos=Paplo-painite%2Fapelles&type=Date&legend=top-left)](https://www.star-history.com/?repos=Paplo-painite%2Fapelles&type=date&legend=top-left)
