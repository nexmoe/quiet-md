//! Document-tree dragging: a floating title, insertion line or dashed parent
//! target, hover expansion and edge scrolling. The store owns the actual move.

use crate::app::{View, Workspace};
use crate::settings::display_title;
use crate::theme::{self, Theme};
use crate::tree::Tree;
use crate::ui::{ToastKind, icon, ui_text};
use gpui::*;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::{Duration, Instant};

const INDENT: f32 = 18.;
const EDGE: f32 = 7.;
const EXPAND_DELAY: Duration = Duration::from_millis(600);

#[derive(Default)]
pub(crate) struct DragBounds {
    pub rows: Rc<RefCell<HashMap<String, Bounds<Pixels>>>>,
    pub root: Rc<Cell<Bounds<Pixels>>>,
}

pub(crate) struct DocDrag {
    id: String,
    ids: HashSet<String>,
    from: Point<Pixels>,
    at: Point<Pixels>,
    started: bool,
    drop: Option<DocDrop>,
    hover: Option<(Option<String>, Instant)>,
    tick: Instant,
}

struct DocDrop {
    parent: Option<String>,
    before: Option<String>,
    into: bool,
    bounds: Bounds<Pixels>,
}

impl Workspace {
    pub(crate) fn press_doc(&mut self, id: &str, at: Point<Pixels>, cx: &mut Context<Self>) {
        let ids = Tree::new(&self.entries).subtree(id).into_iter().collect();
        self.doc_drag = Some(DocDrag { id: id.into(), ids, from: at, at, started: false, drop: None, hover: None, tick: Instant::now() });
        cx.notify();
    }

    pub(crate) fn move_doc_drag(&mut self, ev: &MouseMoveEvent, cx: &mut Context<Self>) {
        let Some(d) = &mut self.doc_drag else { return };
        if ev.pressed_button != Some(MouseButton::Left) {
            self.doc_drag = None;
            cx.notify();
            return;
        }
        d.at = ev.position;
        if !d.started && (d.at - d.from).magnitude() > 4. {
            d.started = true;
            self.row_menu = None;
        }
        if d.started { cx.stop_propagation(); cx.notify(); }
    }

    pub(crate) fn end_doc_drag(&mut self, at: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        let Some(mut d) = self.doc_drag.take() else { return };
        cx.notify();
        if !d.started {
            if self.drag_bounds.rows.borrow().get(&d.id).is_some_and(|b| b.contains(&at)) {
                self.open(&d.id, window, cx);
            }
            return;
        }
        cx.stop_propagation();
        // Recompute at release: a final mouse-up can arrive without a move frame.
        d.drop = self.doc_drop_at(at, &d.id);
        let Some(target) = d.drop else { return };
        match self.store.move_in_tree(&d.id, target.parent.as_deref(), target.before.as_deref()) {
            Ok(false) => {}
            Ok(true) => {
                self.entries = self.store.list();
                self.sort();
                self.reveal_in_tree(&d.id);
                self.sync_sub_pages(cx);
                self.show_toast(crate::settings::text("移动成功"), cx);
            }
            Err(e) => self.show_toast_kind(ToastKind::Error, &crate::localized_format!("移动失败：{}", "Could not move document: {}", crate::settings::diagnostic(&e.to_string())), cx),
        }
    }

    pub(crate) fn dragged_doc(&self, id: &str) -> bool {
        self.doc_drag.as_ref().is_some_and(|d| d.started && d.ids.contains(id))
    }

    fn doc_drop_at(&self, at: Point<Pixels>, source: &str) -> Option<DocDrop> {
        let viewport = self.list_scroll.bounds();
        if !viewport.contains(&at) { return None; }
        let root = self.drag_bounds.root.get();
        let append = |bounds, into| DocDrop { parent: None, before: None, into, bounds };
        if root.contains(&at) { return Some(append(root, true)); }
        if self.folded[1] { return None; }
        let tree = Tree::new(&self.entries);
        let rows = tree.rows(|id| self.expanded.contains(id));
        let bb = self.drag_bounds.rows.borrow();
        let last = rows.last().and_then(|(e, _)| bb.get(&e.id))?;
        if at.y >= last.bottom() {
            return Some(append(Bounds::new(point(last.left() + px(6.), last.bottom()), size(last.size.width - px(6.), px(2.))), false));
        }
        let (mut target, mut depth) = rows.iter().find(|(e, _)| bb.get(&e.id).is_some_and(|b| at.y >= b.top() && at.y < b.bottom() + px(crate::sidebar::ROW_GAP)))?.to_owned();
        let bounds = *bb.get(&target.id)?;
        let before = at.y < bounds.top() + px(EDGE);
        let after = at.y >= bounds.bottom() - px(EDGE);
        let wanted = ((f32::from(at.x - bounds.left()) - 6.).max(0.) / INDENT) as usize;
        let into = !before && !after && wanted >= depth;
        if !into && wanted < depth {
            let ancestors = self.ancestors(&target.id);
            target = *ancestors.get(wanted)?;
            depth = wanted;
        }
        // The whole source subtree is unavailable as a target, including its gaps.
        if tree.subtree(source).iter().any(|id| id == &target.id) { return None; }
        let own = *bb.get(&target.id)?;
        if into {
            return Some(DocDrop { parent: Some(target.id.clone()), before: None, into: true, bounds: own });
        }
        let parent = self.ancestors(&target.id).last().map(|e| e.id.clone());
        let siblings = tree.children(parent.as_deref());
        let index = siblings.iter().position(|e| e.id == target.id)?;
        let before = if before { Some(target.id.clone()) } else { siblings.get(index + 1).map(|e| e.id.clone()) };
        let bottom = tree.subtree(&target.id).iter().filter_map(|id| bb.get(id)).fold(own.bottom(), |bottom, b| bottom.max(b.bottom()));
        let y = if before.as_deref() == Some(target.id.as_str()) { own.top() } else { bottom };
        Some(DocDrop { parent, before, into: false, bounds: Bounds::new(point(own.left() + px(6. + INDENT * depth as f32), y), size((own.size.width - px(6. + INDENT * depth as f32)).max(px(2.)), px(2.))) })
    }

    /// Called once a frame. A still pointer can open a folder and keep scrolling.
    pub(crate) fn tick_doc_drag(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.sidebar_open || !matches!(self.view, View::Doc | View::Home) || self.batch.is_some() {
            self.doc_drag = None;
        }
        let Some(d) = self.doc_drag.as_ref().filter(|d| d.started) else { return };
        let target = self.doc_drop_at(d.at, &d.id);
        let viewport = self.list_scroll.bounds();
        let now = Instant::now();
        let d = self.doc_drag.as_mut().unwrap();
        let hover = target.as_ref().filter(|t| t.into).map(|t| t.parent.clone());
        if d.hover.as_ref().map(|(id, _)| id) != hover.as_ref() {
            d.hover = hover.map(|id| (id, now));
        }
        if let Some((id, since)) = &d.hover && now.duration_since(*since) >= EXPAND_DELAY {
            if let Some(id) = id { self.expanded.insert(id.clone()); }
            else { self.folded[1] = false; }
        }
        let dt = now.duration_since(d.tick).as_secs_f32().min(0.05);
        d.tick = now;
        let dir = if viewport.contains(&d.at) && d.at.y < viewport.top() + px(24.) { 1. }
            else if viewport.contains(&d.at) && d.at.y > viewport.bottom() - px(24.) { -1. } else { 0. };
        if dir != 0. {
            let old = self.list_scroll.offset();
            let y = (old.y + px(dir * 420. * dt)).clamp(-self.list_scroll.max_offset().height, px(0.));
            self.list_scroll.set_offset(point(old.x, y));
        }
        d.drop = target;
        window.request_animation_frame();
        cx.notify();
    }

    pub(crate) fn render_doc_drag(&self, window: &Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let d = self.doc_drag.as_ref().filter(|d| d.started)?;
        let entry = self.entries.iter().find(|e| e.id == d.id)?;
        let mut layer = div().id("doc-drag-overlay").absolute().inset_0().occlude()
            .cursor(if d.drop.is_some() { CursorStyle::ClosedHand } else { CursorStyle::OperationNotAllowed })
            .on_mouse_move(cx.listener(|w, ev: &MouseMoveEvent, _, cx| w.move_doc_drag(ev, cx)))
            .on_mouse_up(MouseButton::Left, cx.listener(|w, ev: &MouseUpEvent, window, cx| w.end_doc_drag(ev.position, window, cx)))
            .on_mouse_up_out(MouseButton::Left, cx.listener(|w, ev: &MouseUpEvent, window, cx| w.end_doc_drag(ev.position, window, cx)));
        if let Some(target) = &d.drop {
            let b = target.bounds;
            let viewport = self.list_scroll.bounds();
            if b.top() >= viewport.top() && b.bottom() <= viewport.bottom() {
                layer = layer.child(if target.into {
                    div().absolute().left(b.left()).top(b.top()).w(b.size.width).h(b.size.height)
                        .rounded(px(crate::sidebar::ROW_R)).bg(Theme::primary().opacity(0.08)).border_1().border_dashed().border_color(Theme::primary())
                } else {
                    div().absolute().left(b.left()).top(b.top()).w(b.size.width).h(px(2.)).bg(Theme::primary())
                        .child(div().absolute().left(px(-2.)).top(px(-2.)).size(px(6.)).rounded_full().bg(Theme::primary()))
                });
            }
        }
        let width = px(220.);
        let x = (d.at.x + px(14.)).min(window.viewport_size().width - width - px(8.)).max(px(8.));
        let y = (d.at.y + px(12.)).min(window.viewport_size().height - px(40.)).max(px(8.));
        layer = layer.child(div().absolute().left(x).top(y).w(width).h(px(32.)).px(px(10.)).flex().items_center().gap(px(8.))
            .rounded(px(5.)).border_1().border_color(Theme::line_border()).bg(Theme::bg_body()).shadow(theme::shadow_menu())
            .child(icon(if Tree::new(&self.entries).has_children(&d.id) { "folder" } else { "doc" }, 16., Theme::text_caption()))
            .child(div().flex_1().min_w_0().overflow_hidden().whitespace_nowrap().text_ellipsis().text_size(px(12.)).child(ui_text(display_title(&entry.title)))));
        Some(deferred(layer).with_priority(5).into_any_element())
    }
}
