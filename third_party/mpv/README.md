# mpv runtime

`libmpv-2.dll` is the x86-64 Windows development runtime from the mpv Windows build published for 2026-10-19.

- Build project: https://github.com/shinchiro/mpv-winbuild-cmake
- Release page: https://github.com/shinchiro/mpv-winbuild-cmake/releases/tag/20261019
- Download asset: `mpv-dev-x86_64-20261019-git-6a262d11f6.7z`
- SHA-256 of the downloaded archive: `CA8604754170DB6F1B23139D8449782B1AE294A883211131954D30085985F77B`
- mpv upstream source: https://github.com/mpv-player/mpv/tree/6a262d11f6

mpv is GPL-2.0-or-later by default; the exact licensing of a build also depends on its enabled components and linked libraries. Keep mpv and dependency notices and corresponding source available when redistributing this DLL. The player loads this DLL dynamically from this folder; it is not linked into the Rust executable.
