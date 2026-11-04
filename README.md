<h1 align="center">quiet.md</h1>

<p align="center">
  <b>一个又快又安静的本地笔记应用。</b><br>
  Rust + GPUI 原生绘制，笔记全部存在你的电脑上，兼容飞书文档的 DocxXML 格式。
</p>

<p align="center">
  <a href="https://github.com/s1dashu/quiet-md/releases/latest"><img src="https://img.shields.io/github/v/release/s1dashu/quiet-md?label=release&color=2f6feb" alt="最新版本"></a>
  <a href="https://github.com/s1dashu/quiet-md/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/s1dashu/quiet-md/ci.yml?branch=main&label=CI" alt="CI 状态"></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey" alt="支持 macOS 和 Windows">
  <img src="https://img.shields.io/badge/built%20with-Rust%20%2B%20GPUI-dea584?logo=rust" alt="使用 Rust 和 GPUI 构建">
  <a href="LICENSE"><img src="https://img.shields.io/github/license/s1dashu/quiet-md?color=green" alt="MIT 许可"></a>
  <img src="https://img.shields.io/badge/status-beta-orange" alt="Beta">
</p>

<p align="center">
  <a href="https://download.quiet.md/latest/quiet-md-macos-arm64.zip"><img src="docs/images/download-macos.svg" height="56" alt="下载 macOS 版"></a>
  &nbsp;
  <a href="https://download.quiet.md/latest/quiet-md-windows-x64-setup.exe"><img src="docs/images/download-windows.svg" height="56" alt="下载 Windows 版"></a>
</p>

<p align="center">
  <a href="https://quiet.md"><b>官网</b></a> ·
  <a href="#下载"><b>下载</b></a> ·
  <a href="#性能">性能</a> ·
  <a href="#功能">功能</a> ·
  <a href="CONTRIBUTING.md">参与开发</a>
</p>

<p align="center">
  <img src="docs/images/home.png" width="100%" alt="quiet.md 的主页：左侧是文档树，右侧按网格排列所有笔记。">
</p>

> [!NOTE]
> quiet.md 目前是 **Beta 版本**，还在快速迭代。遇到 bug 或有想法，欢迎[提 issue](https://github.com/s1dashu/quiet-md/issues)，更欢迎直接提 PR，一起把它打磨好。

## 为什么是 quiet.md

**⚡ 快，原生的那种快。** 整个界面由 Rust 和 [GPUI](https://www.gpui.rs/) 直接用 GPU 绘制，没有 WebView、没有 Electron。百万字文档实测中，输入、删除和滚动的帧处理 P95 均低于 6 ms。

**🔒 本地优先，文件归你。** 没有账号，没有云端，默认不联网。文档库就是电脑上的一个普通文件夹，每篇笔记是一个 XML 文件。用 Git、iCloud、坚果云或任何同步工具管理都可以，哪天不用了，文件也还在。

**📝 兼容飞书 DocxXML，完整的富文本编辑器。** 笔记直接以飞书文档的 DocxXML 格式保存，从飞书复制粘贴、导入导出都能保留结构。标题、待办、表格、分栏、高亮块、代码、公式、图片和附件，写文档常用的块都有。

## 下载

<p align="center">
  <a href="https://download.quiet.md/latest/quiet-md-macos-arm64.zip"><img src="docs/images/download-macos.svg" height="56" alt="下载 macOS 版"></a>
  &nbsp;
  <a href="https://download.quiet.md/latest/quiet-md-windows-x64-setup.exe"><img src="docs/images/download-windows.svg" height="56" alt="下载 Windows 版"></a>
</p>

| 平台 | 系统要求 |
|---|---|
| **macOS**（Apple Silicon） | macOS 13 及以上 |
| **Windows**（x64） | Windows 10 2004 及以上 |

- **macOS**：解压后把 `quiet.md.app` 拖进「应用程序」。安装包已签名并通过 Apple 公证。
- **Windows**：运行安装程序，默认只装到当前用户目录，卸载不会删除笔记。如果首次运行出现 SmartScreen 提示，点「更多信息 → 仍要运行」即可。

也可以在官网 [quiet.md](https://quiet.md) 下载。所有版本和 SHA-256 校验值见 [GitHub Releases](https://github.com/s1dashu/quiet-md/releases)。Intel Mac 暂无安装包，可以[从源码构建](CONTRIBUTING.md#从源码构建)。

首次启动会在空文档库里放一套「开始指南」，十分钟带你过一遍主要功能；已有的笔记不会被覆盖。

## 性能

在一篇 **100 万字**、549 章的混合文档里连续操作 100 次，统计每帧处理耗时：

| 操作 | P50 | P95 |
|---|---:|---:|
| ⌨️ 连续输入 | **5.06 ms** | 5.78 ms |
| ⌫ 连续退格 | **4.99 ms** | 5.55 ms |
| 🖱️ 连续滚动 | **2.08 ms** | 4.14 ms |

120 Hz 屏幕每帧的预算是 8.3 ms，百万字文档下的 P95 也留有余量。

| | |
|---|---|
| macOS 安装包 | 约 18 MB |
| Windows 安装包 | 约 14 MB |
| 运行时 | 无 WebView、无 Electron、无后台服务 |

<sub>Apple M5 Pro / 48 GiB，macOS 26.5，Release 构建。测试方法和复现脚本见 [docs/PERFORMANCE.md](docs/PERFORMANCE.md)。</sub>

## 功能

- **编辑**：标题、有序 / 无序列表、待办、引用、代码块（语法高亮）、LaTeX 公式、表格（可合并单元格）、高亮块、分栏、图片、附件、封面。支持 `#`、`-`、`[]`、`>` 等 Markdown 快捷输入和 `/` 斜杠菜单。
- **整理**：文档树、置顶、收藏、全文搜索、回收站。
- **导入导出**：导入 Markdown 和飞书 DocxXML；导出 Markdown、PDF，也可以直接打印。从飞书或网页粘贴时保留结构。
- **本地文件**：通过「文件 → 打开文件…」（macOS `⌘O` / Windows `Ctrl+O`）或 `quiet_md /path/to/note.md` 打开本地 `.md` / `.mdx`，用现有编辑器预览和编辑，按 `⌘S` / `Ctrl+S` 保存回原文件。保存使用现有 Markdown 导出规则；MDX 按 Markdown 处理，不执行 JSX 组件。
- **快捷键**：按 `⌘ /` 查看全部快捷键，每一个都能改。
- **主题**：内置 Craft、飞书、Notion、Obsidian 四套风格，也可以用 CSS 自己写，见 [docs/THEMES.md](docs/THEMES.md)。

## 你的文件在哪

默认在 `~/Documents/Quiet.md`（Windows 是「文档」下的 `Quiet.md`）。每篇笔记是 `docs/` 里的一份 XML，图片和附件在 `assets/`。保存是原子的：先写临时文件再改名，崩溃也不会留下写了一半的笔记。

目录结构和格式细节见 [docs/FORMAT.md](docs/FORMAT.md)。

## 参与开发

quiet.md 还在 Beta，有很多可以一起做的事：修 bug、补功能、适配更多平台。欢迎提 issue 和 PR。

从源码构建、测试和打包见 [CONTRIBUTING.md](CONTRIBUTING.md)，代码结构见 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)。

## 许可

代码使用 [MIT](LICENSE) 许可。第三方代码、字体和素材见 [THIRD_PARTY.md](THIRD_PARTY.md)。

以下内容不在 MIT 范围内，fork 时请替换：

- `assets/icons/` 里的 67 个 Nucleo 图标使用 Nucleo 商业授权，请勿单独提取再分发；
- 「quiet.md」这个名字和应用图标。
