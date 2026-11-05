<h1 align="center">quiet.md</h1>

<p align="center">
  <b>A fast, quiet, local notes app.</b><br>
  Drawn natively with Rust + GPUI. Your notes stay on your computer, stored in the Feishu (Lark) DocxXML format.
</p>

<p align="center">
  English · <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <a href="https://github.com/s1dashu/quiet-md/releases/latest"><img src="https://img.shields.io/github/v/release/s1dashu/quiet-md?label=release&color=2f6feb" alt="Latest release"></a>
  <a href="https://github.com/s1dashu/quiet-md/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/s1dashu/quiet-md/ci.yml?branch=main&label=CI" alt="CI status"></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey" alt="macOS and Windows">
  <img src="https://img.shields.io/badge/built%20with-Rust%20%2B%20GPUI-dea584?logo=rust" alt="Built with Rust and GPUI">
  <a href="LICENSE"><img src="https://img.shields.io/github/license/s1dashu/quiet-md?color=green" alt="MIT license"></a>
  <img src="https://img.shields.io/badge/status-beta-orange" alt="Beta">
</p>

<p align="center">
  <a href="https://download.quiet.md/latest/quiet-md-macos-arm64.zip"><img src="docs/images/download-macos.svg" height="56" alt="Download for macOS"></a>
  &nbsp;
  <a href="https://download.quiet.md/latest/quiet-md-windows-x64-setup.exe"><img src="docs/images/download-windows.svg" height="56" alt="Download for Windows"></a>
</p>

<p align="center">
  <a href="https://quiet.md/?lang=en"><b>Website</b></a> ·
  <a href="#download"><b>Download</b></a> ·
  <a href="#performance">Performance</a> ·
  <a href="#features">Features</a> ·
  <a href="#contributing">Contributing</a>
</p>

<p align="center">
  <img src="docs/images/home.png" width="100%" alt="The quiet.md home screen: a document tree on the left and all notes in a grid on the right.">
</p>

> [!NOTE]
> quiet.md is in **beta** and changing quickly. If you hit a bug or have an idea, please [open an issue](https://github.com/s1dashu/quiet-md/issues) — or better, send a pull request.

## Why quiet.md

**⚡ Fast, the native kind of fast.** The editor is drawn on the GPU by Rust and [GPUI](https://www.gpui.rs/), without Electron. HTML blocks use an isolated system WebView to display webpages. In a one-million-character document, typing, deleting and scrolling all stay under 6 ms per frame at P95.

**🔒 Local-first. Your files are yours.** No account, no cloud, and nothing goes over the network by default (the optional update check aside). Your library is an ordinary folder on your computer, and each note is one XML file. Manage it with Git, iCloud or any sync tool you like; if you ever stop using quiet.md, your files are still there.

**📝 A full rich-text editor that speaks Feishu (Lark) DocxXML.** Notes are saved in the DocxXML format used by Feishu (Lark) Docs, so copy-paste, import and export keep their structure. Headings, to-dos, tables, columns, callouts, code, math, images and attachments — the blocks you reach for when writing documents are all there.

## Download

<p align="center">
  <a href="https://download.quiet.md/latest/quiet-md-macos-arm64.zip"><img src="docs/images/download-macos.svg" height="56" alt="Download for macOS"></a>
  &nbsp;
  <a href="https://download.quiet.md/latest/quiet-md-windows-x64-setup.exe"><img src="docs/images/download-windows.svg" height="56" alt="Download for Windows"></a>
</p>

| Platform | Requirements |
|---|---|
| **macOS** (Apple Silicon) | macOS 13 or later |
| **Windows** (x64) | Windows 10 version 2004 or later |

- **macOS**: unzip and drag `quiet.md.app` into Applications. The app is signed and notarized by Apple.
- **Windows**: run the installer. By default it installs for the current user only, and uninstalling never deletes your notes. If SmartScreen appears on first launch, click **More info → Run anyway**.

You can also download from [quiet.md](https://quiet.md/?lang=en). Every release and its SHA-256 checksums are on [GitHub Releases](https://github.com/s1dashu/quiet-md/releases). There is no Intel Mac build yet; you can [build from source](CONTRIBUTING.md#从源码构建).

On first launch, quiet.md puts a **Getting started** guide into an empty library — a ten-minute tour of the main features. Existing notes are never overwritten.

The interface is available in English and Simplified Chinese.

## Performance

Frame processing time over 100 consecutive operations in a mixed document of **1 million characters** and 549 chapters:

| Operation | P50 | P95 |
|---|---:|---:|
| ⌨️ Typing | **5.06 ms** | 5.78 ms |
| ⌫ Backspace | **4.99 ms** | 5.55 ms |
| 🖱️ Scrolling | **2.08 ms** | 4.14 ms |

A 120 Hz display gives each frame 8.3 ms. Even at P95 on a million-character document, there is headroom left.

| | |
|---|---|
| macOS download | ~18 MB |
| Windows installer | ~14 MB |
| Runtime | Native GPUI editor; system WebView for HTML blocks; no Electron or background services |

<sub>Apple M5 Pro / 48 GiB, macOS 26.5, release build. Methodology and scripts to reproduce: [docs/PERFORMANCE.md](docs/PERFORMANCE.md).</sub>

## Features

- **HTML blocks**: upload or drop an `.html` / `.htm` file, edit its source next to a live HTML/CSS/JavaScript preview, resize the split, and enter fullscreen. HTML source is saved in the document; remote resources in that source may access the network. Windows requires the Microsoft Edge WebView2 Runtime.
- **Editing**: headings, ordered and bulleted lists, to-dos, quotes, code blocks with syntax highlighting, LaTeX math, tables (with merged cells), callouts, columns, images, attachments and covers. Markdown shortcuts such as `#`, `-`, `[]` and `>`, plus a `/` slash menu.
- **Organizing**: a document tree, pinning, favorites, full-text search and a trash.
- **Import and export**: import Markdown and Feishu (Lark) DocxXML; export Markdown and PDF, or print directly. Pasting from Feishu or a web page keeps the structure.
- **Shortcuts**: press `⌘ /` to see them all. Every one can be rebound.
- **Themes**: four built-in styles — Craft, Feishu, Notion and Obsidian — each with a dark variant, or write your own in CSS; see [docs/THEMES.md](docs/THEMES.md).

## Recent updates

- **0.1.4** (Nov 5):
  - In-app updates: when a new version is out, an **Update** button appears at the top of the sidebar. The download is verified against its signature, and a restart finishes the install; open documents are saved first. Automatic checks can be turned off in **Settings › General**. A check only fetches a version manifest and sends no data. Version 0.1.3 and earlier need a one-time manual download of 0.1.4.
  - Dark mode: choose System, Light or Dark in **Settings › Appearance**. Built-in and custom themes all have a dark look; export and printing stay light.
- **0.1.3** (Nov 4): sidebar text is readable again when macOS is in dark mode.
- **0.1.2** (Nov 4): the standard macOS menus and shortcuts (Hide, Minimize, Zoom, Full Screen); clicking the Dock icon reopens a window after the last one was closed. Fixed some Chinese characters rendering with the wrong glyphs.
- **0.1.1** (Nov 3): first Windows x64 release, installed per user, keeping your notes on uninstall. Polished the Chinese and English interface and the built-in guide.

Full notes for each version are on [Releases](https://github.com/s1dashu/quiet-md/releases).

## Where your files live

By default, `~/Documents/Quiet.md` (on Windows, `Quiet.md` inside your Documents folder). Each note is an XML file in `docs/`; images and attachments are in `assets/`.

For the folder layout and format details, see [docs/FORMAT.md](docs/FORMAT.md).

## Contributing

quiet.md is still in beta and there is plenty to do: fixing bugs, adding features, supporting more platforms. Issues and pull requests are welcome.

Quick start from source (Rust toolchain is pinned in `rust-toolchain.toml`; macOS needs the Xcode Command Line Tools, Windows needs the Visual Studio 2022 C++ Build Tools and the Windows SDK):

```sh
cargo build --release --locked
QUIET_MD_HOME=/tmp/quiet-md-dev ./target/release/quiet_md   # use a scratch library, not your real one
scripts/check.sh                                            # tests + clippy with zero warnings
```

The repository is a Cargo workspace: `crates/core` (`fdl_core`) is the document engine — model, DocxXML, editing core and undo, library storage, import and export — and does not depend on GPUI. The root crate is the GPUI app. GPUI and taffy are patched copies in `vendor/` (see [vendor/PATCHES.md](vendor/PATCHES.md)).

Building, testing and packaging are covered in [CONTRIBUTING.md](CONTRIBUTING.md), the code structure in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), and code conventions in [AGENTS.md](AGENTS.md). These developer docs are currently written in Chinese.

## License

The code is licensed under [MIT](LICENSE). Third-party code, fonts and assets are listed in [THIRD_PARTY.md](THIRD_PARTY.md).

The following are not covered by the MIT license and should be replaced in forks:

- The 67 Nucleo icons in `assets/icons/` are under a commercial Nucleo license. Do not extract or redistribute them on their own.
- The name "quiet.md" and the app icon.
