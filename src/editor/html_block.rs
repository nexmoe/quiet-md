//! HTML block lifecycle, native surface placement and document edits.

use super::{core::{Pos, Selection}, render::R, view::Editor};
use crate::{html_preview::Preview, model::{Block, BlockId, BlockKind}, editor_theme::Theme};
use fdl_core::html_block::{HtmlBlock, MAX_BYTES};
use gpui::{prelude::FluentBuilder, *};
use serde_json::{Value, json};
use std::{cell::RefCell, collections::{HashMap, HashSet}, hash::{Hash, Hasher}, rc::Rc};

struct Slot { preview: Preview, nonce: String, key: Option<StateKey>, epoch: u64, revision: u64, used: u64 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StateKey { source: u64, split: u32, read_only: bool, full: bool, english: bool, theme: u64, epoch: u64, revision: u64 }

impl StateKey {
    fn new(html: &HtmlBlock, read_only: bool, full: bool, epoch: u64, revision: u64) -> Self {
        let mut hash = rustc_hash::FxHasher::default(); html.source.hash(&mut hash);
        Self { source: hash.finish(), split: html.split().to_bits(), read_only, full, english: crate::settings::english(), theme: crate::themes::generation(), epoch, revision }
    }
}

#[derive(Default)]
pub(super) struct HtmlUi {
    slots: Rc<RefCell<HashMap<BlockId, Slot>>>,
    errors: HashMap<BlockId, String>,
    heights: HashMap<BlockId, f32>,
    editing: HashSet<BlockId>,
    pub full: Option<BlockId>,
    pub obscured: bool,
    blocked: bool,
    clock: u64,
    generation: Option<u64>,
    painted: Rc<RefCell<HashSet<BlockId>>>,
    viewport: Bounds<Pixels>,
}

fn css(c: Hsla) -> String { let c = c.to_rgb(); format!("rgba({},{},{},{})", (c.r*255.).round(), (c.g*255.).round(), (c.b*255.).round(), c.a) }

fn state(html: &HtmlBlock, key: StateKey) -> String {
    let mut syntax = vec![]; let mut at = 0;
    if html.source.len() <= 100_000 {
        for (range, color, italic) in crate::highlight::spans("HTML", &html.source) {
            if range.start < at { continue }
            if range.start > at { syntax.push(json!({"text":&html.source[at..range.start]})); }
            syntax.push(json!({"text":&html.source[range.clone()],"color":css(color),"italic":italic})); at = range.end;
        }
    }
    syntax.push(json!({"text":&html.source[at..]}));
    json!({"source":html.source,"epoch":key.epoch,"revision":key.revision,"readOnly":key.read_only,"full":key.full,"english":key.english,"split":html.split(),"maxBytes":MAX_BYTES,
        "syntax":syntax,
        "colors":{"text":css(Theme::text()),"body":css(Theme::bg_body()),"panel":css(Theme::bg_overlay()),"muted":css(Theme::text_caption()),
            "border":css(Theme::code_border()),"accent":css(Theme::primary()),"hover":css(Theme::fill_hover())}}).to_string()
}

pub(super) fn html_in(b: &Block) -> Vec<(BlockId, HtmlBlock)> {
    fn walk(b: &Block, out: &mut Vec<(BlockId, HtmlBlock)>) {
        if let BlockKind::Html(h) = &b.kind { out.push((b.id, h.clone())); }
        if let BlockKind::Table(t) = &b.kind { for c in t.rows.iter().flatten() { for b in &c.blocks { walk(b, out); } } }
        for b in &b.children { walk(b, out); }
    }
    let mut out = vec![];
    if !super::render::flattens(b) { walk(b, &mut out); }
    out
}

impl Editor {
    #[cfg(feature = "devtools")]
    pub(crate) fn inspect_html(&self, index: usize, js: &str) {
        let mut ids = vec![]; self.core.doc.for_each_block(|b| { if matches!(b.kind, BlockKind::Html(_)) { ids.push(b.id); } });
        if let Some(id) = ids.get(index) && let Some(slot) = self.html_blocks.slots.borrow().get(id) { slot.preview.inspect(js); }
    }

    pub(crate) fn obscure_html(&mut self, obscured: bool, cx: &mut Context<Self>) {
        if self.html_blocks.obscured == obscured { return }
        self.html_blocks.obscured = obscured;
        if obscured { for slot in self.html_blocks.slots.borrow().values() { slot.preview.hide(); } }
        cx.notify();
    }

    pub(super) fn begin_html_frame(&mut self, blocked: bool) {
        let viewport = self.list.viewport_bounds();
        let ui = &mut self.html_blocks;
        ui.viewport = viewport;
        ui.blocked = blocked || ui.obscured;
        ui.painted.borrow_mut().clear();
        if ui.blocked { for slot in ui.slots.borrow().values() { slot.preview.hide(); } }
        if ui.generation != Some(self.core.generation) {
            let mut ids = HashSet::new(); self.core.doc.for_each_block(|b| { if matches!(b.kind, BlockKind::Html(_)) { ids.insert(b.id); } });
            ui.slots.borrow_mut().retain(|id, _| ids.contains(id));
            ui.heights.retain(|id, _| ids.contains(id)); ui.editing.retain(|id| ids.contains(id)); ui.errors.retain(|id, _| ids.contains(id));
            ui.full = ui.full.filter(|id| ids.contains(id)); ui.generation = Some(self.core.generation);
        }
    }

    pub(super) fn finish_html_frame(&self) {
        for (id, slot) in self.html_blocks.slots.borrow().iter() {
            if !self.html_blocks.painted.borrow().contains(id) && self.html_blocks.full != Some(*id) { slot.preview.hide(); }
        }
    }

    pub(super) fn html_nonces(&self) -> Vec<(BlockId, u64)> {
        self.html_blocks.heights.iter().map(|(id, height)| (*id, height.to_bits() as u64)).chain(self.html_blocks.editing.iter().map(|id| (*id, 1))).collect()
    }

    pub(super) fn ensure_html(&mut self, blocks: Vec<(BlockId, HtmlBlock)>, window: &mut Window, cx: &mut Context<Self>) {
        for (id, html) in blocks {
            self.html_blocks.clock += 1;
            let used = self.html_blocks.clock;
            if let Some(slot) = self.html_blocks.slots.borrow_mut().get_mut(&id) {
                slot.used = used;
                let key = StateKey::new(&html, self.read_only, self.html_blocks.full == Some(id), slot.epoch, slot.revision);
                if slot.key != Some(key) {
                    let value = state(&html, key);
                    if slot.preview.script(&format!("window.quietHtml&&window.quietHtml({value})")).is_ok() { slot.key = Some(key); }
                }
                continue;
            }
            if self.html_blocks.errors.contains_key(&id) { continue }
            let key = StateKey::new(&html, self.read_only, self.html_blocks.full == Some(id), 0, 0);
            let value = state(&html, key);
            let nonce = uuid::Uuid::new_v4().to_string();
            let shell = include_str!("../html_preview.html").replace("__QUIET_NONCE__", &json!(nonce).to_string())
                .replace("__QUIET_STATE__", &value.replace('<', "\\u003c").replace('\u{2028}', "\\u2028").replace('\u{2029}', "\\u2029"));
            let entity = cx.entity().downgrade(); let app = cx.to_async(); let handle = window.window_handle();
            let preview = Preview::new(window, shell, move |body| {
                let entity = entity.clone(); let mut app = app.clone();
                app.foreground_executor().clone().spawn(async move {
                    let _ = handle.update(&mut app, |_, window, cx| { let _ = entity.update(cx, |e, cx| e.html_message(id, &body, window, cx)); });
                }).detach();
            });
            match preview {
                Ok(preview) => {
                    let mut slots = self.html_blocks.slots.borrow_mut();
                    if slots.len() >= 8 && let Some(old) = slots.iter().filter(|(key, _)| Some(**key) != self.html_blocks.full).min_by_key(|(_, s)| s.used).map(|(id, _)| *id) { slots.remove(&old); }
                    slots.insert(id, Slot { preview, nonce, key: Some(key), epoch: 0, revision: 0, used });
                }
                Err(error) => { self.html_blocks.errors.insert(id, error); }
            }
        }
    }

    fn html_message(&mut self, id: BlockId, body: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Ok(value) = serde_json::from_str::<Value>(body) else { return };
        if !self.html_blocks.slots.borrow().get(&id).is_some_and(|s| value["nonce"].as_str() == Some(s.nonce.as_str())) { return }
        let Some(BlockKind::Html(mut html)) = self.core.kind_of(id).cloned() else { return };
        match value["type"].as_str().unwrap_or("") {
            "ready" => { if let Some(slot) = self.html_blocks.slots.borrow_mut().get_mut(&id) { slot.key = None; } }
            "source" if !self.read_only => {
                let Some(source) = value["source"].as_str().filter(|s| s.len() <= MAX_BYTES) else { return };
                let Some(revision) = value["revision"].as_u64() else { return };
                {
                    let mut slots = self.html_blocks.slots.borrow_mut(); let Some(slot) = slots.get_mut(&id) else { return };
                    if value["epoch"].as_u64() != Some(slot.epoch) || revision <= slot.revision { return }
                    slot.revision = revision;
                }
                self.core.sel = Selection::caret(Pos::new(id, 0)); html.source = source.into();
                self.core.set_html(id, html, true); self.changed(window, cx);
            }
            "undo" | "redo" if !self.read_only => {
                if value["type"] == "undo" { self.core.undo(); } else { self.core.redo(); }
                self.reset_html_source();
                self.changed(window, cx);
            }
            "split" if !self.read_only => {
                if let Some(split) = value["split"].as_f64().filter(|n| n.is_finite()) {
                    html.set_layout(html.height(), split as f32); self.core.set_html(id, html, false); self.changed(window, cx);
                }
            }
            "editing" => {
                if value["edit"] == true && !self.read_only { self.html_blocks.editing.insert(id); } else { self.html_blocks.editing.remove(&id); }
                self.items_key = None;
            }
            "height" if self.html_blocks.full != Some(id) && !self.html_blocks.editing.contains(&id) => {
                if let Some(height) = value["height"].as_f64().filter(|h| h.is_finite()) {
                    let height = (height as f32).clamp(160., 1600.);
                    if self.html_blocks.heights.get(&id).is_none_or(|old| (*old-height).abs() > 1.) { self.html_blocks.heights.insert(id, height); self.items_key = None; }
                }
            }
            "fullscreen" => { self.html_blocks.full = Some(id); }
            "escape" => { self.html_blocks.full = None; window.focus(&self.focus); }
            "scroll" if self.html_blocks.full.is_none() => { if let Some(dy) = value["dy"].as_f64().filter(|n| n.is_finite()) { self.list.scroll_by(px((dy as f32).clamp(-2000., 2000.))); } }
            "upload" if !self.read_only => self.pick_html(id, window, cx),
            "copy" => cx.write_to_clipboard(ClipboardItem::new_string(html.source)),
            _ => return,
        }
        cx.notify();
    }

    fn pick_html(&mut self, id: BlockId, window: &mut Window, cx: &mut Context<Self>) {
        let picked = crate::devtools::take_pick();
        let rx = picked.is_none().then(|| cx.prompt_for_paths(PathPromptOptions { files: true, directories: false, multiple: false, prompt: Some(crate::settings::text("HTML 块").into()), extensions: Some(vec!["html".into(), "htm".into()]) }));
        cx.spawn_in(window, async move |this, cx| {
            let paths = match rx { Some(rx) => rx.await.ok().and_then(Result::ok).flatten(), None => picked };
            let Some(path) = paths.and_then(|p| p.into_iter().next()) else { return };
            let result = cx.background_executor().spawn(async move {
                if !path.extension().and_then(|s| s.to_str()).is_some_and(|s| s.eq_ignore_ascii_case("html") || s.eq_ignore_ascii_case("htm"))
                    || std::fs::metadata(&path).map_err(|_| "HTML 文件读取失败")?.len() > MAX_BYTES as u64 { return Err("请选择 .html 或 .htm 文件（不超过 8 MB）") }
                use std::io::Read;
                let mut source = String::new();
                std::fs::File::open(path).map_err(|_| "HTML 文件读取失败")?.take(MAX_BYTES as u64 + 1).read_to_string(&mut source).map_err(|_| "HTML 文件读取失败")?;
                if source.len() > MAX_BYTES { return Err("请选择 .html 或 .htm 文件（不超过 8 MB）") }
                Ok(source.trim_start_matches('\u{feff}').to_string())
            }).await;
            let _ = this.update_in(cx, |e, window, cx| {
                if e.read_only { return }
                match result {
                    Ok(source) => if let Some(BlockKind::Html(mut html)) = e.core.kind_of(id).cloned() { html.source = source; e.core.set_html(id, html, false); e.reset_html_source(); e.changed(window, cx); },
                    Err(error) => e.toast(crate::settings::text(error), cx),
                }
            });
        }).detach();
    }

    pub(super) fn reset_html_source(&mut self) {
        // External edits supersede any source messages still in flight from a webview.
        for slot in self.html_blocks.slots.borrow_mut().values_mut() { slot.epoch += 1; slot.revision = 0; }
    }

    pub(super) fn html_fullscreen(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let id = self.html_blocks.full?;
        let BlockKind::Html(html) = self.core.kind_of(id)?.clone() else { return None };
        self.ensure_html(vec![(id, html)], window, cx);
        let size = window.viewport_size();
        let surface = self.html_surface(id, true);
        Some(deferred(anchored().position_mode(AnchoredPositionMode::Window).position(point(px(0.), px(0.))).child(div().w(size.width).h(size.height).bg(Theme::bg_body()).flex().flex_col()
            .child(div().h(px(64.)).flex_none().pl(px(if cfg!(target_os = "macos") { 110. } else { 24. })).pr(px(24.)).flex().items_center().gap(px(20.)).border_b_1().border_color(Theme::code_border())
                .child(div().id("html-exit").cursor_pointer().child(format!("× {}", crate::settings::text("退出全屏"))).on_click(cx.listener(|e, _, window, cx| { e.html_blocks.full = None; window.focus(&e.focus); cx.notify(); })))
                .child(crate::editor_ui::icon("braces", 20., Theme::primary())).child(crate::settings::text("HTML 块")))
            .child(div().flex_1().min_h_0().p(px(24.)).child(surface)))).with_priority(20).into_any_element())
    }

    fn html_surface(&self, id: BlockId, full: bool) -> AnyElement {
        let slots = self.html_blocks.slots.clone();
        let painted = self.html_blocks.painted.clone();
        let blocked = self.html_blocks.blocked;
        let viewport = self.html_blocks.viewport;
        canvas(|_, _, _| (), move |bounds, _, window, cx| {
            if blocked { return }
            if let Some(slot) = slots.borrow().get(&id) {
                painted.borrow_mut().insert(id);
                let viewport = if full { bounds } else { viewport };
                slot.preview.place(bounds, crate::html_preview::clip(bounds.intersect(&viewport), window, cx), window);
            }
        }).size_full().into_any_element()
    }
}

impl R<'_> {
    pub(super) fn html_block(&self, b: &Block, html: &HtmlBlock, cx: &mut Context<Editor>) -> AnyElement {
        let id = b.id;
        let height = if self.ed.html_blocks.editing.contains(&id) { html.height().max(360.) } else { self.ed.html_blocks.heights.get(&id).copied().unwrap_or(html.height()) };
        let error = self.ed.html_blocks.errors.get(&id);
        div().w_full().h(px(height)).relative()
            .when(self.ghost, |d| d.bg(Theme::bg_overlay()).child(crate::settings::text("HTML 块")))
            .when(!self.ghost && self.ed.html_blocks.full != Some(id), |d| d.child(self.ed.html_surface(id, false)))
            .when_some(error, |d, error| d.border_1().border_color(Theme::code_border()).rounded(px(8.)).p(px(24.))
                .child(crate::settings::text("HTML 预览不可用")).child(div().text_size(px(12.)).text_color(Theme::text_caption()).child(error.clone()))
                .child(div().id(("html-retry", id.0)).cursor_pointer().child(crate::settings::text("重试")).on_click(cx.listener(move |e, _, _, cx| { e.html_blocks.errors.remove(&id); cx.notify(); }))))
            .into_any_element()
    }
}
