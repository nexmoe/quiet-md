//! Text drawing: persisted source, three views and a native SVG preview.

use super::render::R;
use super::view::Editor;
use crate::input::{TextField, TextFieldEvent};
use crate::model::{Block, BlockId, BlockKind};
use crate::editor_theme::Theme;
use crate::editor_ui::{popover, menu_row};
use fdl_core::diagram::{DiagramLayout, DiagramView, TextDiagram};
use crate::ui::icon;
use crate::ui::TipExt;
use gpui::{prelude::FluentBuilder, *};
use std::{collections::HashMap, sync::Arc, time::Duration};

#[derive(Clone, Copy, PartialEq)]
enum Drop { Views }

#[derive(Clone, PartialEq)]
struct Key { source: String, palette: pele_rs::Palette, minute: u64, raster: u32 }

struct Preview {
    key: Key,
    image: Option<(Arc<RenderImage>, f32, f32)>,
    svg: Option<Arc<str>>,
    error: Option<String>,
    pending: bool,
    used: u64,
    _task: Option<Task<()>>,
}

#[derive(Default)]
pub(super) struct DiagramUi {
    pub fields: HashMap<BlockId, (Entity<TextField>, Vec<Subscription>)>,
    previews: HashMap<BlockId, Preview>,
    pub full: Option<BlockId>,
    full_source_height: f32,
    drop: Option<(BlockId, Drop)>,
    reader_views: HashMap<BlockId, DiagramView>,
    split: HashMap<BlockId, f32>,
    layouts: HashMap<BlockId, DiagramLayout>,
    split_drag: Option<(BlockId, Pixels, f32, f32)>,
    clock: u64,
}

impl DiagramUi {
    pub fn release(&mut self, cx: &mut App) {
        for p in self.previews.values() { if let Some((image, _, _)) = &p.image { release_image(image.clone(), cx); } }
    }
}

fn release_image(image: Arc<RenderImage>, cx: &mut App) { cx.defer(move |cx| cx.drop_image(image, None)); }

pub(super) fn diagrams_in(b: &Block) -> Vec<(BlockId, TextDiagram)> {
    fn walk(b: &Block, out: &mut Vec<(BlockId, TextDiagram)>) {
        if let BlockKind::Diagram(d) = &b.kind { out.push((b.id, d.clone())); }
        if let BlockKind::Table(t) = &b.kind { for c in t.rows.iter().flatten() { for b in &c.blocks { walk(b, out); } } }
        for b in &b.children { walk(b, out); }
    }
    let mut out = vec![];
    if !super::render::flattens(b) { walk(b, &mut out); }
    out
}

fn button(id: (&'static str, u64), label: &'static str) -> Stateful<Div> {
    div().id(id).px(px(7.)).h(px(28.)).flex().items_center().rounded(px(4.))
        .text_size(px(12.)).text_color(Theme::text_caption()).cursor_pointer()
        .hover(|s| s.bg(Theme::fill_hover())).child(crate::settings::text(label))
}

impl Editor {
    fn save_diagram_layout(&mut self, id: BlockId, coalesce: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.read_only { return }
        let Some(BlockKind::Diagram(d)) = self.core.kind_of(id) else { return };
        let mut layout = d.layout();
        layout.split = self.diagrams.split.get(&id).copied().unwrap_or(0.5);
        self.core.sel = super::core::Selection::caret(super::core::Pos::new(id, 0));
        if self.core.set_diagram_layout(id, layout, coalesce) { self.changed(window, cx); self.autoscroll = false; }
    }

    fn finish_diagram_drag(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let split = self.diagrams.split_drag.take().map(|v| v.0);
        if let Some(id) = split { self.save_diagram_layout(id, false, window, cx); }
    }

    fn diagram_drag(&mut self, ev: &MouseMoveEvent, window: &mut Window, cx: &mut Context<Self>) {
        if ev.pressed_button != Some(MouseButton::Left) { self.finish_diagram_drag(window, cx); return }
        if let Some((id, start, ratio, width)) = self.diagrams.split_drag {
            self.diagrams.split.insert(id, (ratio + f32::from(ev.position.x - start) / width).clamp(0.2, 0.8));
            cx.stop_propagation(); cx.notify();
        }
    }

    pub(super) fn diagram_escape(&mut self, cx: &mut Context<Self>) -> bool {
        if self.diagrams.drop.take().is_some() || self.diagrams.full.take().is_some() { cx.notify(); true } else { false }
    }

    pub(super) fn ensure_diagrams(&mut self, found: Vec<(BlockId, TextDiagram)>, window: &mut Window, cx: &mut Context<Self>) {
        for (id, d) in found {
            self.diagrams.clock += 1;
            let used = self.diagrams.clock;
            let source_height = if self.diagrams.full == Some(id) { self.diagrams.full_source_height.max(200.) } else { 240. };
            let layout = d.layout();
            if self.diagrams.layouts.get(&id) != Some(&layout) {
                self.diagrams.split.insert(id, layout.split);
                self.diagrams.layouts.insert(id, layout);
            }
            if let Some((field, _)) = self.diagrams.fields.get(&id) {
                if field.read(cx).min_height != source_height || field.read(cx).color != Theme::code_text() {
                    field.update(cx, |f, cx| { f.min_height = source_height; f.color = Theme::code_text(); cx.notify(); });
                }
                if field.read(cx).text != d.source && !field.read(cx).focus_handle(cx).is_focused(window) {
                    field.update(cx, |f, cx| f.set_text(&d.source, cx));
                }
            } else {
                let field = cx.new(|cx| {
                    let mut f = TextField::new("输入 Mermaid 源码", cx);
                    f.multiline = true; f.wrap = true; f.mono = true; f.history_events = true; f.min_height = source_height;
                    f.size = 13.; f.line_height = 22.; f.color = Theme::code_text();
                    f.set_text(&d.source, cx);
                    f
                });
                let sub = cx.subscribe_in(&field, window, move |e, field, ev, window, cx| {
                    if e.read_only {
                        if let Some(BlockKind::Diagram(d)) = e.core.block(id).map(|b| &b.kind) {
                            let source = d.source.clone(); field.update(cx, |f, cx| f.set_text(&source, cx));
                        }
                        return;
                    }
                    match ev {
                        TextFieldEvent::Changed => {
                            e.core.sel = super::core::Selection::caret(super::core::Pos::new(id, 0));
                            if let Some(BlockKind::Diagram(mut d)) = e.core.block(id).map(|b| b.kind.clone()) {
                                d.source = field.read(cx).text.clone();
                                e.core.set_diagram(id, d, true);
                                e.changed(window, cx);
                                e.autoscroll = false;
                            }
                        }
                        TextFieldEvent::Undo | TextFieldEvent::Redo => {
                            if *ev == TextFieldEvent::Undo { e.core.undo(); } else { e.core.redo(); }
                            if let Some(BlockKind::Diagram(d)) = e.core.block(id).map(|b| &b.kind) {
                                let source = d.source.clone();
                                field.update(cx, |f, cx| f.set_text(&source, cx));
                            } else { window.focus(&e.focus); }
                            e.changed(window, cx);
                            if e.core.sel.head.block == id && e.core.block(id).is_some_and(|b| matches!(b.kind, BlockKind::Diagram(_))) { e.autoscroll = false; }
                        }
                        TextFieldEvent::Cancel => {
                            e.diagrams.drop = None;
                            e.diagrams.full = None;
                            window.focus(&e.focus);
                            e.core.select_block(id);
                            cx.notify();
                        }
                        _ => {}
                    }
                });
                self.diagrams.fields.insert(id, (field, vec![sub]));
            }
            let raster = (window.scale_factor() * 2.).ceil() / 2.;
            let key = Key { source: d.source.clone(), palette: crate::diagram::palette(&d.theme), minute: crate::time::now_ms() / 60_000, raster: raster.to_bits() };
            if let Some(p) = self.diagrams.previews.get_mut(&id).filter(|p| p.key == key) { p.used = used; continue }
            // Keep the last good image while a new edit is being parsed, or has a syntax error.
            let (image, svg) = self.diagrams.previews.remove(&id).map(|p| (p.image, p.svg)).unwrap_or_default();
            if d.source.trim().is_empty() {
                if let Some((image, _, _)) = image { release_image(image, cx); }
                self.diagrams.previews.insert(id, Preview { key, image: None, svg: None, error: None, pending: false, used, _task: None });
                continue;
            }
            let wanted = key.clone();
            let renderer = cx.svg_renderer();
            let task = cx.spawn(async move |this, cx| {
                cx.background_executor().timer(Duration::from_millis(120)).await;
                let work = wanted.clone();
                let result = cx.background_executor().spawn(async move {
                    let r = crate::diagram::render(&work.source, 690., work.palette, work.minute * 60_000)?;
                    let image = crate::diagram::raster(renderer, &r.svg, r.width as f32, r.height as f32, f32::from_bits(work.raster))?;
                    Ok(((image, r.width as f32, r.height as f32), Arc::<str>::from(r.svg)))
                }).await;
                let _ = this.update(cx, |e, cx| {
                    let Some(p) = e.diagrams.previews.get_mut(&id).filter(|p| p.key == wanted) else { return };
                    p.pending = false;
                    match result {
                        Ok((image, svg)) => {
                            if let Some((old, _, _)) = p.image.take() { release_image(old, cx); }
                            p.image = Some(image); p.svg = Some(svg); p.error = None;
                        }
                        Err(err) => p.error = Some(err),
                    }
                    cx.notify();
                });
            });
            self.diagrams.previews.insert(id, Preview { key, image, svg, error: None, pending: true, used, _task: Some(task) });
        }
        // Scrolling through a library must not keep every rasterized chart and field alive.
        let cost = |p: &Preview| p.image.as_ref().map_or(0., |(image, _, _)| { let s = image.size(0); s.width.0 as f32 * s.height.0 as f32 * 4. });
        let mut bytes: f32 = self.diagrams.previews.values().map(cost).sum();
        if self.diagrams.previews.len() > 48 || bytes > 64. * 1024. * 1024. {
            let keep = self.diagrams.full;
            let focused = self.diagrams.fields.iter().find(|(_, (f, _))| f.read(cx).focus_handle(cx).is_focused(window)).map(|(id, _)| *id);
            let mut remove: Vec<_> = self.diagrams.previews.iter().filter(|(id, _)| Some(**id) != keep && Some(**id) != focused).map(|(id, p)| (p.used, *id)).collect();
            remove.sort_unstable();
            for (_, id) in remove {
                if self.diagrams.previews.len() <= 32 && bytes <= 64. * 1024. * 1024. { break }
                if let Some(p) = self.diagrams.previews.remove(&id) {
                    bytes -= cost(&p);
                    if let Some((image, _, _)) = p.image { release_image(image, cx); }
                }
                self.diagrams.fields.remove(&id);
                self.diagrams.split.remove(&id); self.diagrams.layouts.remove(&id);
            }
        }
    }

    fn preview_diagram(&mut self, id: BlockId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(p) = self.diagrams.previews.get(&id) else { return };
        let Some((image, w, h)) = &p.image else { return };
        let Some(svg) = &p.svg else { return };
        self.open_diagram_lightbox(id, image.clone(), *w, *h, svg.clone(), window, cx);
    }

    fn update_diagram(&mut self, id: BlockId, change: impl FnOnce(&mut TextDiagram), window: &mut Window, cx: &mut Context<Self>) {
        if self.read_only { return }
        if let Some(BlockKind::Diagram(mut d)) = self.core.block(id).map(|b| b.kind.clone()) {
            change(&mut d);
            let source = d.source.clone();
            self.core.sel = super::core::Selection::caret(super::core::Pos::new(id, 0));
            self.core.set_diagram(id, d, false);
            if let Some((f, _)) = self.diagrams.fields.get(&id) { f.update(cx, |f, cx| f.set_text(&source, cx)); }
            self.diagrams.drop = None;
            self.changed(window, cx);
            self.autoscroll = false;
        }
    }

    pub(super) fn diagram_fullscreen(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let id = self.diagrams.full?;
        self.hover = None;
        let Some(BlockKind::Diagram(d)) = self.core.block(id).map(|b| b.kind.clone()) else { self.diagrams.full = None; return None };
        let pane = self.pane_size();
        // List row callbacks cannot borrow the list's viewport while it is laying out.
        self.diagrams.full_source_height = (f32::from(pane.height) - 108.).max(200.);
        self.ensure_diagrams(vec![(id, d.clone())], window, cx);
        let width = (f32::from(pane.width) - 32.).max(240.);
        let height = (f32::from(pane.height) - 68.).max(240.);
        Some(div().absolute().inset_0().bg(Theme::bg_body()).p(px(16.)).occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_move(cx.listener(|e, ev, window, cx| { e.diagram_drag(ev, window, cx); cx.stop_propagation(); }))
            .on_mouse_up(MouseButton::Left, cx.listener(|e, _, window, cx| e.finish_diagram_drag(window, cx)))
            .on_mouse_up_out(MouseButton::Left, cx.listener(|e, _, window, cx| e.finish_diagram_drag(window, cx)))
            .child(widget(self, id, &d, width, Some(height), cx)).into_any_element())
    }
}

impl R<'_> {
    pub(super) fn diagram(&self, id: BlockId, d: &TextDiagram, width: f32, cx: &mut Context<Editor>) -> AnyElement {
        if self.ghost {
            let mut el = div().w_full().h(px(336.)).bg(Theme::bg_body()).p(px(12.));
            if let Some((image, w, h)) = self.ed.diagrams.previews.get(&id).and_then(|p| p.image.as_ref()) {
                let scale = ((width - 24.) / w).min(300. / h).min(1.);
                el = el.child(img(image.clone()).w(px(w * scale)).h(px(h * scale)));
            } else { el = el.child(d.source.clone()); }
            return el.into_any_element();
        }
        widget(self.ed, id, d, width, None, cx)
    }
}

fn widget(e: &Editor, id: BlockId, d: &TextDiagram, width: f32, full_height: Option<f32>, cx: &mut Context<Editor>) -> AnyElement {
    let full = full_height.is_some();
    let view = if e.read_only { e.diagrams.reader_views.get(&id).copied().unwrap_or(d.view) } else { d.view };
    let open = e.diagrams.drop == Some((id, Drop::Views));
    let mut views = div().relative().child(button(("diagram-views", id.0), "视图")
        .child(icon("chevron-down", 12., Theme::text_caption()))
        .on_mouse_down(MouseButton::Left, cx.listener(move |e: &mut Editor, _, _, cx| {
            e.diagrams.drop = if open { None } else { Some((id, Drop::Views)) };
            cx.stop_propagation(); cx.notify();
        })));
    if open {
        let mut menu = popover().id(("diagram-drop", id.0)).absolute().top(px(30.)).right_0().w(px(208.)).py(px(4.)).occlude();
        for (i, (label, selected)) in [("代码和图表", DiagramView::CodeChart), ("仅代码", DiagramView::Code), ("仅图表", DiagramView::Chart)].into_iter().enumerate() {
            menu = menu.child(menu_row(("diagram-option", i), "diagram-options".into())
                .child(div().flex_1().child(crate::settings::text(label)))
                .when(view == selected, |d| d.child("✓"))
                .on_mouse_down(MouseButton::Left, cx.listener(move |e: &mut Editor, _, window, cx| {
                    cx.stop_propagation();
                    if e.read_only { e.diagrams.reader_views.insert(id, selected); e.diagrams.drop = None; cx.notify(); }
                    else { e.update_diagram(id, |d| d.view = selected, window, cx); }
                })));
        }
        views = views.child(menu);
    }
    let toolbar = div().absolute().top(px(4.)).right(px(4.)).flex().items_center().gap(px(2.)).h(px(28.))
        .bg(Theme::bg_body()).rounded(px(4.)).child(views)
        .child(div().h(px(14.)).w(px(1.)).mx(px(3.)).bg(Theme::line_border()))
        .child(div().id(("diagram-full", id.0)).size(px(28.)).flex().items_center().justify_center().rounded(px(4.)).cursor_pointer()
            .hover(|s| s.bg(Theme::fill_hover())).tip(crate::settings::text(if full { "退出全屏" } else { "全屏" }), None)
            .child(icon("expand", 14., Theme::text_caption()))
            .on_mouse_down(MouseButton::Left, cx.listener(move |e: &mut Editor, _, window, cx| {
                cx.stop_propagation(); e.diagrams.full = if full { None } else { Some(id) }; e.diagrams.drop = None;
                window.focus(&e.focus); cx.notify();
            })));
    let side_by_side = width >= 520. && view == DiagramView::CodeChart;
    let ratio = e.diagrams.split.get(&id).copied().unwrap_or(0.5);
    let code_width = if side_by_side { (width - 6.) * ratio } else { width };
    let chart_width = if side_by_side { (width - 6.) * (1. - ratio) } else { width };
    let height = full_height.unwrap_or(280.);
    let body_height = if view == DiagramView::CodeChart && !side_by_side { height * 2. } else { height };
    let mut body = div().flex().min_w_0().h(px(body_height)).min_h(px(body_height)).when(!side_by_side, |d| d.flex_col());
    if view != DiagramView::Chart {
        let source: AnyElement = if e.read_only || !full && e.diagrams.full == Some(id) {
            div().font_family(crate::editor_theme::MONO_FONT).text_size(px(13.)).line_height(px(22.)).child(d.source.clone()).into_any_element()
        } else if let Some((field, _)) = e.diagrams.fields.get(&id) { field.clone().into_any_element() } else { div().child(d.source.clone()).into_any_element() };
        body = body.child(div().id(("diagram-source-scroll", id.0)).w(px(code_width)).min_w_0().h(px(height)).overflow_y_scroll()
            .bg(Theme::bg_body()).pt(px(24.)).pb(px(16.)).px(px(16.)).child(div().flex().items_start().gap(px(8.))
                .child(div().flex_none().w(px(20.)).text_size(px(12.)).line_height(px(22.)).text_color(Theme::text_caption())
                    .children((1..=d.source.lines().count().max(1)).map(|n| div().h(px(22.)).child(n.to_string()))))
                .child(source)));
    }
    if side_by_side {
        body = body.child(div().id(("diagram-split", id.0)).w(px(6.)).h(px(height)).flex_none().cursor(CursorStyle::ResizeLeftRight)
            .bg(Theme::line_border()).hover(|s| s.bg(Theme::primary_fill_1()))
            .on_mouse_down(MouseButton::Left, cx.listener(move |e: &mut Editor, ev: &MouseDownEvent, _, cx| {
                e.diagrams.split_drag = Some((id, ev.position.x, ratio, width - 6.));
                cx.stop_propagation();
            })));
    }
    if view != DiagramView::Code {
        let p = e.diagrams.previews.get(&id);
        let bg = crate::diagram::background(&d.theme);
        let mut chart = div().w(px(chart_width)).min_w_0().h(px(height)).overflow_hidden().bg(bg);
        if let Some((image, w, h)) = p.and_then(|p| p.image.as_ref()) {
            let fit = ((chart_width - 24.).max(32.) / w).min((height - 24.).max(32.) / h).min(1.);
            let (iw, ih) = (w * fit, h * fit);
            chart = chart.child(div().w(px((iw + 24.).max(chart_width))).h(px((ih + 24.).max(height))).flex().items_center().justify_center()
                .when(p.is_some_and(|p| p.error.is_some()), |d| d.opacity(0.4))
                .child(div().w(px(iw)).h(px(ih)).cursor(CursorStyle::ZoomIn)
                    .on_mouse_down(MouseButton::Left, cx.listener(move |e: &mut Editor, _, window, cx| {
                        cx.stop_propagation(); e.preview_diagram(id, window, cx);
                    }))
                    .child(img(image.clone()).w(px(iw)).h(px(ih)))));
        } else {
            chart = chart.child(div().size_full().flex().items_center().justify_center().p(px(16.)).text_size(px(13.)).text_color(Theme::text_caption())
                .child(crate::settings::text(if p.is_some_and(|p| p.pending) { "正在渲染…" } else { "输入源码展示图表" })));
        }
        let mut pane = div().relative().w(px(chart_width)).h(px(height)).min_h(px(height)).child(chart);
        if let Some(error) = p.and_then(|p| p.error.as_ref()) {
            pane = pane.child(div().absolute().left_0().right_0().bottom_0().max_h(px(100.)).overflow_hidden().p(px(8.))
                .bg(Theme::danger_cell()).text_color(Theme::danger()).text_size(px(12.)).child(error.clone()));
        }
        body = body.child(pane);
    }
    div().id(("text-drawing", id.0)).relative().w_full().min_w_0().border_1().border_color(Theme::line_border()).rounded(px(8.))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_mouse_move(cx.listener(|e: &mut Editor, ev, window, cx| e.diagram_drag(ev, window, cx)))
        .on_mouse_up(MouseButton::Left, cx.listener(|e: &mut Editor, _, window, cx| e.finish_diagram_drag(window, cx)))
        .on_mouse_up_out(MouseButton::Left, cx.listener(|e: &mut Editor, _, window, cx| e.finish_diagram_drag(window, cx)))
        .child(body).child(toolbar).into_any_element()
}
