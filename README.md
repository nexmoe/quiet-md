<h1 align="center">Quiet.md</h1>

<p align="center">
  <b>A fast, quiet, local notes app.</b><br>
  Drawn natively with Rust + GPUI. Your notes stay on your computer, stored in the Feishu (Lark) DocxXML format.
</p>

<p align="center">
  English · <a href="README.zh-CN.md">简体中文</a> · <a href="README.ja.md">日本語</a>
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
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

<p align="center">
  <img src="docs/images/home.png" width="100%" alt="The Quiet.md home screen: a document tree on the left and all notes in a grid on the right.">
</p>

> [!NOTE]
> Quiet.md is in **beta** and changing quickly. If you hit a bug or have an idea, please [open an issue](https://github.com/s1dashu/quiet-md/issues) — or better, send a pull request.

Linux support is experimental and currently built from source; see the [Linux instructions and validation limits](CONTRIBUTING.md#linux-experimental).

## Why Quiet.md

**⚡ Fast, the native kind of fast.** The whole interface is drawn on the GPU by Rust and [GPUI](https://www.gpui.rs/). No WebView, no Electron. In a one-million-character document, typing, deleting and scrolling all stay under 6 ms per frame at P95.

**🔒 Local-first. Your files are yours.** No account, no cloud, and nothing goes over the network by default (the optional update check aside). Your library is an ordinary folder on your computer, and each note is one XML file. Manage it with Git, iCloud or any sync tool you like; if you ever stop using Quiet.md, your files are still there.

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

You can also download from [Quiet.md](https://quiet.md/?lang=en). Every release and its SHA-256 checksums are on [GitHub Releases](https://github.com/s1dashu/quiet-md/releases). There is no Intel Mac build yet; you can [build from source](CONTRIBUTING.md#building-from-source).

On first launch, Quiet.md puts a **Getting started** guide into an empty library — a ten-minute tour of the main features. Existing notes are never overwritten.

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
| Runtime | No WebView, no Electron, no background services |

<sub>Apple M5 Pro / 48 GiB, macOS 26.5, release build. Methodology and scripts to reproduce: [docs/PERFORMANCE.md](docs/PERFORMANCE.md).</sub>

## Features

- **Editing**: headings, ordered and bulleted lists, to-dos, quotes, code blocks with syntax highlighting, LaTeX math, tables (with merged cells), callouts, columns, images, attachments and covers. Markdown shortcuts such as `#`, `-`, `[]` and `>`, plus a `/` slash menu.
- **Organizing**: drag documents in the sidebar to reorder them or move entire subtrees, with pinning, favorites, full-text search and a trash.
- **Import and export**: import Markdown and Feishu (Lark) DocxXML; export Markdown and PDF, or print directly. Pasting from Feishu or a web page keeps the structure.
- **Shortcuts**: press `⌘ /` (`Ctrl+/` on Windows) to see them all. Every one can be rebound. On Windows, Ctrl stands in for ⌘ and Alt for ⌥; the app and the Getting started guides show the keys for your system.
- **Themes**: four built-in styles — Craft, Feishu, Notion and Obsidian — each with a dark variant, or write your own in CSS; see [docs/THEMES.md](docs/THEMES.md).

## Contributors

Thanks to everyone who has helped build Quiet.md. Issues and pull requests are welcome: building, testing and code conventions are in [CONTRIBUTING.md](CONTRIBUTING.md).

[![Contributors](https://raw.githubusercontent.com/s1dashu/quiet-md/contributors-assets/docs/images/contributors.svg?v=1)](https://github.com/s1dashu/quiet-md/graphs/contributors)

## Recent updates

- **0.1.7** (Nov 6): safer multi-window saving, conflict recovery and disk writes; restricted export file access; Markdown exports keep inline attachments. Fixed Chinese input deletion with Weasel on Windows.
- **0.1.6** (Nov 6): smoother window switching and typing, wider tables, regular saves during continuous typing, and Option+Space for global quick notes on macOS. Windows adds image paste, PDF export and printing, with corrected word deletion.
- **0.1.5** (Nov 5): the app is now called Quiet.md; your library and notes stay where they were.
  - Manage several documents at once from Home: duplicate, move or delete them together.
  - Empty the Trash in one step; arrow keys move through the insert menu's icons; deleting every document no longer creates an empty one.
  - After an in-app update, the sidebar shows once what changed in the new version.
  - Windows: shortcuts and the built-in guide read Ctrl / Alt, and a failed launch explains itself and leaves a log.
- **0.1.4** (Nov 5):
  - In-app updates: when a new version is out, an **Update** button appears at the top of the sidebar. The download is verified against its signature, and a restart finishes the install; open documents are saved first. Automatic checks can be turned off in **Settings › General**. A check only fetches a version manifest and sends no data. Version 0.1.3 and earlier need a one-time manual download of 0.1.4.
  - Dark mode: choose System, Light or Dark in **Settings › Appearance**. Built-in and custom themes all have a dark look; export and printing stay light.
- **0.1.3** (Nov 4): sidebar text is readable again when macOS is in dark mode.
- **0.1.2** (Nov 4): the standard macOS menus and shortcuts (Hide, Minimize, Zoom, Full Screen); clicking the Dock icon reopens a window after the last one was closed. Fixed some Chinese characters rendering with the wrong glyphs.
- **0.1.1** (Nov 3): first Windows x64 release, installed per user, keeping your notes on uninstall. Polished the Chinese and English interface and the built-in guide.

Full notes for each version are on [Releases](https://github.com/s1dashu/quiet-md/releases).

## License

The code is licensed under [MIT](LICENSE). Third-party code, fonts and assets are listed in [THIRD_PARTY.md](THIRD_PARTY.md).

The following are not covered by the MIT license and should be replaced in forks:

- The 67 Nucleo icons in `assets/icons/` are under a commercial Nucleo license. Do not extract or redistribute them on their own.
- The name "Quiet.md" and the app icon.
