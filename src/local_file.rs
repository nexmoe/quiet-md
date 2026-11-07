//! Local Markdown files reuse the importer, editor and Markdown exporter.
use crate::{app, editor::view::{Editor, EditorEvent}, settings::tr};
use gpui::*;
use std::path::{Path, PathBuf};

pub fn paths_from_urls(urls: Vec<String>) -> Vec<PathBuf> {
    urls.into_iter().filter_map(|s| url::Url::parse(&s).ok()?.to_file_path().ok()).collect()
}

pub fn install(cx: &mut App) {
    cx.on_action(|_: &app::OpenLocalFile, cx| {
        let picked = crate::devtools::take_pick();
        let prompt = picked.is_none().then(|| cx.prompt_for_paths(PathPromptOptions {
            files: true, directories: false, multiple: true,
            prompt: Some(tr("打开", "Open").into()), extensions: Some(vec!["md".into(), "mdx".into()]),
        }));
        cx.spawn(async move |cx| {
            let paths = match prompt { Some(p) => p.await.ok().and_then(Result::ok).flatten(), None => picked };
            if let Some(paths) = paths { let _ = cx.update(|cx| open_paths(&paths, cx)); }
        }).detach();
    });
}

pub fn open_paths(paths: &[PathBuf], cx: &mut App) {
    for path in paths {
        if let Err(e) = open_path(path, cx) {
            if cx.windows().is_empty() { crate::open_main_window(crate::devtools::scripted(), cx); }
            let message = crate::localized_format!("无法打开文件：{}", "Could not open file: {}", path.display());
            let detail = e.to_string();
            if let Some(handle) = cx.active_window().or_else(|| cx.windows().first().copied()) {
                let _ = handle.update(cx, |_, window, cx| {
                    let rx = window.prompt(PromptLevel::Critical, &message, Some(&detail), &[tr("确定", "OK")], cx);
                    cx.spawn(async move |_| { let _ = rx.await; }).detach();
                });
            }
        }
    }
}

fn open_path(path: &Path, cx: &mut App) -> anyhow::Result<()> {
    if !path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("mdx")) {
        anyhow::bail!("{}", tr("请选择 .md 或 .mdx 文件", "Choose an .md or .mdx file"));
    }
    let path = path.canonicalize()?;
    for handle in cx.windows().into_iter().filter_map(|w| w.downcast::<LocalFile>()) {
        if handle.update(cx, |preview, window, _| {
            if preview.path != path { return false; }
            window.activate_window(); true
        }).unwrap_or(false) { return Ok(()); }
    }
    let original = std::fs::read(&path)?;
    std::str::from_utf8(&original)?;
    let doc = crate::markdown_file::read(&path, false)?;
    let title = format!("{} — Quiet.md", path.file_name().unwrap_or_default().to_string_lossy());
    cx.open_window(WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, size(px(900.), px(720.)), cx))),
        titlebar: Some(TitlebarOptions { title: Some(title.into()), ..Default::default() }),
        window_min_size: Some(size(px(480.), px(400.))), focus: !crate::devtools::scripted(), ..Default::default()
    }, |window, cx| cx.new(|cx| {
        let saved = doc.clone();
        let editor = cx.new(|cx| {
            let mut ed = Editor::new(doc, crate::time::now(), Default::default(), window, cx);
            ed.info.hidden = true;
            ed
        });
        editor.read(cx).focus(window);
        let subs = vec![
            cx.subscribe(&editor, |file: &mut LocalFile, editor, event, cx| match event {
                EditorEvent::Changed => { file.dirty = editor.read(cx).core.doc != file.saved; cx.notify(); },
                EditorEvent::Save => { file.save(cx); },
                _ => {},
            }),
            cx.on_app_quit(|file, cx| { file.save(cx); async {} }),
        ];
        let this = cx.weak_entity();
        window.on_window_should_close(cx, move |window, cx| this.update(cx, |file, cx| file.close(false, window, cx)).unwrap_or(false));
        LocalFile { path, editor, original, saved, dirty: false, error: None, closing: false, _subs: subs }
    }))?;
    Ok(())
}

struct LocalFile {
    path: PathBuf,
    editor: Entity<Editor>,
    original: Vec<u8>,
    saved: crate::model::Document,
    dirty: bool,
    error: Option<String>,
    closing: bool,
    _subs: Vec<Subscription>,
}

impl LocalFile {
    fn save(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.dirty { return true; }
        let doc = &self.editor.read(cx).core.doc;
        let result = std::fs::read(&self.path).and_then(|bytes| {
            if bytes != self.original { return Err(std::io::Error::other(tr("文件已被其他程序修改，请重新打开后再保存", "File changed on disk; reopen it before saving"))); }
            crate::markdown_file::write(&self.path, doc)
        });
        match result {
            Ok(bytes) => { self.original = bytes; self.saved = doc.clone(); self.dirty = false; self.error = None; },
            Err(e) => self.error = Some(crate::localized_format!("保存失败：{}", "Could not save: {}", e)),
        }
        cx.notify();
        self.error.is_none()
    }

    fn close(&mut self, quit: bool, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if !self.dirty { return true; }
        if self.closing { return false; }
        self.closing = true;
        let message = crate::localized_format!("保存对 {} 的更改？", "Save changes to {}?", self.path.display());
        let rx = window.prompt(PromptLevel::Warning, &message, None, &[tr("保存", "Save"), tr("不保存", "Don't Save"), tr("取消", "Cancel")], cx);
        cx.spawn_in(window, async move |this, cx| {
            let answer = rx.await.ok();
            let _ = this.update_in(cx, |file, window, cx| {
                file.closing = false;
                if answer == Some(1) || (answer == Some(0) && file.save(cx)) {
                    file.dirty = false;
                    window.remove_window();
                    if quit { cx.defer(|cx| cx.dispatch_action(&app::Quit)); }
                }
            });
        }).detach();
        false
    }
}

pub fn save_before_quit(cx: &mut App) -> bool {
    for handle in cx.windows().into_iter().filter_map(|w| w.downcast::<LocalFile>()) {
        if !handle.update(cx, |file, window, cx| file.close(true, window, cx)).unwrap_or(false) { return false; }
    }
    true
}

impl Render for LocalFile {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.error.clone().unwrap_or_else(|| if self.dirty { tr("未保存", "Unsaved") } else { tr("已保存", "Saved") }.into());
        div().size_full().flex().flex_col().bg(crate::editor_theme::Theme::bg_body())
            .font_family(crate::theme::UI_FONT)
            .on_action(cx.listener(|file, _: &app::CloseWindow, window, cx| { if file.close(false, window, cx) { window.remove_window(); } }))
            .on_action(cx.listener(|file, _: &crate::editor::keymap::Save, _, cx| { file.save(cx); }))
            .on_action(cx.listener(|preview, _: &app::FindInDoc, window, cx| preview.editor.update(cx, |ed, cx| ed.open_find(false, window, cx))))
            .on_action(cx.listener(|file, _: &app::ReplaceInDoc, window, cx| file.editor.update(cx, |ed, cx| ed.open_find(true, window, cx))))
            .on_action(|_: &app::Minimize, window, _| window.minimize_window())
            .on_action(|_: &app::Zoom, window, _| window.zoom_window())
            .on_action(|_: &app::ToggleFullScreen, window, _| window.toggle_fullscreen())
            .child(div().px(px(24.)).py(px(8.)).text_size(px(12.)).text_color(crate::theme::Theme::text_caption()).child(format!("{} · {status}", self.path.display())))
            .child(div().flex_1().min_h_0().child(self.editor.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::paths_from_urls;

    #[test]
    fn finder_urls_preserve_local_paths() {
        let paths = [std::env::temp_dir().join("中文 空格 #%?.md"), std::env::temp_dir().join("Second.MDX")];
        let urls = paths.iter().map(|p| url::Url::from_file_path(p).unwrap().to_string()).collect();
        assert_eq!(paths_from_urls(urls), paths);
        assert!(paths_from_urls(vec!["https://example.com/file.md".into(), "not a URL".into()]).is_empty());
        #[cfg(not(target_os = "windows"))]
        assert!(paths_from_urls(vec!["file://remote-host/note.md".into()]).is_empty());
    }
}
