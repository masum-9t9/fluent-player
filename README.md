# Fluent Player

![Fluent Player](ui/assets/fluent-player.jpg)

A native Windows video player built with Rust, Slint, and mpv. Fluent Player pairs a dark charcoal interface with a warm orange accent and a compact playback workspace.

[![Version](https://img.shields.io/badge/version-0.0.1-orange)](Cargo.toml) [![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-0078D4)](#requirements) [![Rust](https://img.shields.io/badge/Rust-1.92%2B-b7410e)](https://www.rust-lang.org/) [![License](https://img.shields.io/badge/license-GPL--3.0--only-blue)](https://www.gnu.org/licenses/gpl-3.0.html)

## Features

- Native Slint interface with animated controls, reduced-motion option, custom title bar, library navigation, and playback layout.
- Open individual media files or scan a folder; browse recent items, playback history, favorites, and the current playlist.
- mpv playback with seek, previous/next, frame stepping, mute, subtitle/audio track cycling, repeat modes, and A–B looping.
- Picture-in-picture, screenshots, hardware-decoding toggle, playback speed (0.25×–4×), volume (up to 200%), and brightness/contrast/saturation/gamma adjustments.
- Recent-item and playback preferences are stored in the user's local application-data directory.

Supported video extensions include MP4, MKV, WebM, MOV, AVI, WMV, M4V, MPG/MPEG, TS/M2TS, 3GP, OGV, FLV, and VOB. Folder scanning also recognizes MP3, FLAC, and WAV audio.

## Download

Download the Windows application and portable archive from the repository's [Releases](../../releases) page. A portable package includes the executable and required `libmpv-2.dll`; keep them together. The local `release/` folder contains build artifacts and is intentionally excluded from Git.

## Requirements

- Windows 10 or Windows 11, x86-64.
- The release's `libmpv-2.dll` alongside `FluentPlayer.exe` for playback.
- To build: Rust 1.92+, MSVC C++ build tools, Windows SDK (`rc.exe` embeds the app icon), and the x64 `libmpv-2.dll` placed in `third_party/mpv/`. The runtime DLL is distributed separately from the source repository because it exceeds GitHub's regular per-file limit.

## Build from source

```powershell
cargo build --release
```

The executable is written to `target/release/fluent_player.exe`. To prepare the versioned application and portable ZIP under `release/`, run:

```powershell
./scripts/package-release.ps1
```

Before packaging, obtain the x64 mpv runtime described in [`third_party/mpv/README.md`](third_party/mpv/README.md) and place `libmpv-2.dll` in `third_party/mpv/`. The packaging script copies the runtime and notices into the release packages.

## Technology and third-party components

- [Rust](https://www.rust-lang.org/) — application language and toolchain.
- [Slint](https://slint.dev/) — declarative user interface.
- [mpv](https://github.com/mpv-player/mpv) via dynamically loaded `libmpv-2.dll` — media playback. Runtime build provenance and checksum are in [`third_party/mpv/README.md`](third_party/mpv/README.md).
- [mpv Windows build](https://github.com/shinchiro/mpv-winbuild-cmake/releases/tag/20261019) — the runtime build documented for this project.

Fluent Player declares `GPL-3.0-only` in `Cargo.toml` (read the [GNU GPL v3](https://www.gnu.org/licenses/gpl-3.0.html)). mpv and its bundled dependencies have their own license terms; review their notices and source obligations before redistributing binaries. See [`third_party/mpv/README.md`](third_party/mpv/README.md).

## Links

- Developer: **Masum 9T9**
- Website: [fluent-9t9.vercel.app](https://fluent-9t9.vercel.app)
- Developer portfolio: [9t9.pro.bd](https://www.9t9.pro.bd/)
- GitHub: [@masum-9t9](https://github.com/masum-9t9/)
- Facebook: [Masum 9T9 Official](https://www.facebook.com/masum.9t9.official)

## Contributing and support

Bug reports and feature requests: [GitHub Issues](../../issues/new). Pull requests are welcome. Include the Windows version, reproduction steps, and relevant logs when reporting playback problems.

## Project links

This checkout has no Git remote configured, so the repository's canonical URL is unknown. The relative Releases and Issues links above work when published on GitHub. Add the canonical repo/profile links here when available.

---

Copyright © 2026 **Masum 9T9**.
