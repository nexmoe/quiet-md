<h1 align="center">Quiet.md</h1>

<p align="center">
  <b>一个又快又安静的本地笔记应用。</b><br>
  Rust + GPUI 原生绘制，笔记全部存在你的电脑上，兼容飞书文档的 DocxXML 格式。
</p>

<p align="center">
  <a href="README.md">English</a> · 简体中文 · <a href="README.ja.md">日本語</a>
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
  <img src="docs/images/home.png" width="100%" alt="Quiet.md 的主页：左侧是文档树，右侧按网格排列所有笔记。">
</p>

> [!NOTE]
> Quiet.md 目前是 **Beta 版本**，还在快速迭代。遇到 bug 或有想法，欢迎[提 issue](https://github.com/s1dashu/quiet-md/issues)，更欢迎直接提 PR，一起把它打磨好。

Linux 支持目前处于实验阶段，需要从源码构建；参见 [Linux 构建说明与验证范围](CONTRIBUTING.md#linux-experimental)。

## 为什么是 Quiet.md

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

也可以在官网 [Quiet.md](https://quiet.md) 下载。所有版本和 SHA-256 校验值见 [GitHub Releases](https://github.com/s1dashu/quiet-md/releases)。Intel Mac 暂无安装包，可以[从源码构建](CONTRIBUTING.md#building-from-source)。

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

- **编辑**：标题、有序 / 无序列表、待办、引用、代码块（语法高亮）、LaTeX 公式、Mermaid 文本绘图、表格（可合并单元格）、高亮块、分栏、图片、附件、封面。支持 `#`、`-`、`[]`、`>` 等 Markdown 快捷输入和 `/` 斜杠菜单。
- **整理**：文档树、置顶、收藏、全文搜索、回收站。
- **导入导出**：导入 Markdown 和飞书 DocxXML；导出 Markdown、独立 HTML 和 PDF，macOS 还支持直接打印。从飞书或网页粘贴时保留结构。Windows 的 PDF 导出使用已安装的 Microsoft Edge WebView2 Runtime。
- **快捷键**：按 `⌘ /`（Windows 上是 `Ctrl+/`）查看全部快捷键，每一个都能改。快捷键在 Windows 上用 Ctrl 代替 ⌘、Alt 代替 ⌥，应用里和「开始指南」里都按当前系统显示。
- **主题**：内置 Craft、飞书、Notion、Obsidian 四套风格，也可以用 CSS 自己写，见 [docs/THEMES.md](docs/THEMES.md)。

## 贡献者

感谢每一位参与 Quiet.md 的人。欢迎提 issue 和 PR，构建、测试和代码约定见 [CONTRIBUTING.md](CONTRIBUTING.md)。

[![Contributors](https://raw.githubusercontent.com/s1dashu/quiet-md/contributors-assets/docs/images/contributors.svg?v=1)](https://github.com/s1dashu/quiet-md/graphs/contributors)

## 近期更新

- **0.1.7**（11 月 6 日）：加固多窗口保存、冲突恢复和写盘；限制导出读取文件的范围；Markdown 导出保留行内附件；修复 Windows 小狼毫输入法误删文字。
- **0.1.6**（11 月 6 日）：切回窗口和点击空白后可继续输入，宽表格利用页边空间，持续输入时定期保存；macOS 全局快速笔记改为 ⌥ Space。Windows 支持粘贴图片、导出 PDF 和打印，并修复按词删除。
- **0.1.5**（11 月 5 日）：应用更名为 Quiet.md，文档库位置和已有内容不变。
  - 首页新增批量管理：一次选中多篇文档，创建副本、移动或删除。
  - 回收站可以一键清空；插入菜单的图标可以用方向键移动；删除全部文档后不再自动新建空文档。
  - 应用内更新后，侧栏底部会显示一次新版本的变化。
  - Windows：快捷键和内置指南显示为 Ctrl / Alt；启动失败时会弹出提示并记录日志。
- **0.1.4**（11 月 5 日）：
  - 应用内自动更新：有新版本时，侧栏顶部会出现「更新」按钮；下载后先校验签名，重启即完成安装，打开的文档会先保存。自动检查可以在 设置 › 通用 中关闭，检查时只读取一份版本信息，不发送任何数据。0.1.3 及更早的版本需要手动下载一次 0.1.4。
  - 深色模式：设置 › 外观 可选跟随系统、浅色或深色，内置主题和自定义主题都有对应的深色效果；导出和打印仍是浅色。
- **0.1.3**（11 月 4 日）：macOS 系统切到深色模式时，侧栏文字不再看不清。
- **0.1.2**（11 月 4 日）：补齐 macOS 的标准菜单和快捷键（隐藏、最小化、缩放、全屏）；关掉最后一个窗口后，点 Dock 图标可以重新打开。修复部分中文字显示成错误字形的问题。
- **0.1.1**（11 月 3 日）：推出 Windows x64 版，按用户安装，卸载时保留笔记；同时完善中英文界面和内置指南。

每个版本的完整说明见 [Releases](https://github.com/s1dashu/quiet-md/releases)。

## 许可

代码使用 [MIT](LICENSE) 许可。第三方代码、字体和素材见 [THIRD_PARTY.md](THIRD_PARTY.md)。

以下内容不在 MIT 范围内，fork 时请替换：

- `assets/icons/` 里的 67 个 Nucleo 图标使用 Nucleo 商业授权，请勿单独提取再分发；
- 「Quiet.md」这个名字和应用图标。
