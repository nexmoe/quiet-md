//! 更新日志: after an update, a quiet card at the foot of the sidebar lists
//! what this version changed, once. The notes are compiled in (no network);
//! a new library (first launch) never sees it.
use std::sync::atomic::{AtomicBool, Ordering};

use gpui::prelude::*;
use gpui::{div, px, AnyElement, ClickEvent, FontWeight, Window, App};

use crate::theme::Theme;
use crate::ui::icon;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const RADIUS: f32 = 12.;

/// Each version's notes: (version, [(中文, English)]). Only the running
/// version's are shown; a version without notes shows nothing.
const NOTES: &[(&str, &[(&str, &str)])] = &[
    ("0.1.7", &[
        ("退出时检查所有窗口的保存状态，保存冲突支持重新载入和另存副本", "Check every window before quitting; reload or save a copy when saving fails"),
        ("退出时无法保存的内容会保留为恢复副本，下次启动恢复", "Preserve unsaved work as recovery copies when quitting and restore it on next launch"),
        ("Markdown 导出保留行内附件的链接和文件", "Markdown exports include inline attachment links and files"),
        ("导出限制本地文件的读取范围和大小，写盘失败时更可靠", "Safer local-file exports and more reliable handling of write failures"),
        ("Windows：修复小狼毫退格多删字符、回车多换行", "Windows: fix extra deletion and newlines when using the Weasel input method"),
    ]),
    ("0.1.6", &[
        ("点击空白或切回窗口后可以继续输入，回到应用时更流畅", "Keep typing after clicking blank space or returning to a window; faster library refresh"),
        ("宽表格可延伸至页边，竖向滚动不再带动表格横移", "Wide tables use the page margins and stay still during vertical scrolling"),
        ("持续输入时也会及时保存，输入时隐藏块手柄", "Save regularly during continuous typing and hide block handles while writing"),
        ("macOS：全局快速笔记默认快捷键改为 ⌥ Space", "macOS: the default global quick-note shortcut is now Option+Space"),
        ("Windows：支持粘贴截图和图片文件、导出 PDF 和打印，修复按词删除", "Windows: paste screenshots and image files, export PDF and print, and delete words correctly"),
    ]),
    ("0.1.5", &[
        ("应用更名为 Quiet.md", "The app is now called Quiet.md"),
        ("首页新增批量管理：创建副本、移动、删除", "Manage several documents at once from Home: duplicate, move, delete"),
        ("回收站可以一键清空", "Empty the Trash in one step"),
        ("插入菜单的图标可以用方向键移动", "Arrow keys move through the insert menu's icons"),
        ("删除全部文档后不再自动新建空文档", "Deleting every document no longer creates an empty one"),
        ("Windows：快捷键显示为 Ctrl / Alt，启动失败时会提示并记录日志", "Windows: shortcuts read Ctrl / Alt, and a failed launch explains itself and leaves a log"),
    ]),
];

static PENDING: AtomicBool = AtomicBool::new(false);

fn notes() -> Option<&'static [(&'static str, &'static str)]> { NOTES.iter().find(|(v, _)| *v == VERSION).map(|(_, n)| *n) }

/// At launch, once preferences and themes are loaded: a library that existed
/// before this launch and was last opened by an older version gets the card.
/// Records this version either way, so it shows only once.
pub fn startup(existed: bool) {
    let mut p = crate::settings::prefs();
    if p.seen_version == VERSION { return }
    let upgraded = existed && (p.seen_version.is_empty() || crate::update::newer(VERSION, &p.seen_version));
    PENDING.store(upgraded && notes().is_some(), Ordering::Relaxed);
    p.seen_version = VERSION.into();
    if let Err(e) = crate::settings::save(&p) { log::warn!("changelog: {e:#}"); }
}

/// The card, while it has not been closed.
pub fn card(on_close: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Option<AnyElement> {
    if !PENDING.load(Ordering::Relaxed) { return None }
    let notes = notes()?;
    let title = crate::localized_format!("已更新到 {}", "Updated to {}", VERSION);
    let mut list = div().mt(px(6.)).w_full().min_w_0().flex().flex_col().gap(px(4.));
    for &(zh, en) in notes {
        list = list.child(div().w_full().min_w_0().flex().gap(px(6.)).text_size(px(12.)).line_height(px(18.)).text_color(Theme::text_caption())
            .child(div().flex_none().child("·")).child(div().flex_1().min_w_0().child(crate::settings::tr(zh, en))));
    }
    // A continuous-corner white surface; the faint shadow follows its radius.
    Some(crate::ui::continuous_surface(RADIUS, Theme::bg_body()).id("changelog").flex_none().min_w_0().overflow_hidden().mx(px(crate::sidebar::SB_X - 4.)).mb(px(2.)).pl(px(12.)).pr(px(6.)).pt(px(8.)).pb(px(10.))
        .rounded(px(RADIUS)).shadow(crate::theme::shadow_card())
        .child(div().w_full().min_w_0().flex().items_center().justify_between()
            .child(div().text_size(px(13.)).line_height(px(20.)).font_weight(FontWeight::SEMIBOLD).child(title))
            .child(div().id("changelog-close").size(px(20.)).flex().items_center().justify_center().rounded(px(6.)).cursor_pointer()
                .hover(|s| s.bg(Theme::fill_hover())).child(icon("x", 12., Theme::text_caption()))
                .on_click(move |e, w, cx| { PENDING.store(false, Ordering::Relaxed); on_close(e, w, cx); })))
        .child(list)
        .into_any_element())
}
