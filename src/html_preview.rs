//! A clipped system web surface for HTML blocks; the rest of the editor stays in GPUI.

use gpui::{Bounds, Pixels, Window};

#[derive(Default)]
struct Overlays(std::collections::HashMap<gpui::WindowId, Vec<Bounds<Pixels>>>);
impl gpui::Global for Overlays {}

pub fn begin_frame(window: &Window, cx: &mut gpui::App) { cx.default_global::<Overlays>().0.entry(window.window_handle().window_id()).or_default().clear(); }

/// Shared overlay shells register during prepaint, before any native surface is placed.
/// New overlay shells must include this marker; menus, panels, tooltips and toasts do so centrally.
pub fn overlay_marker() -> impl gpui::IntoElement {
    use gpui::*;
    canvas(|bounds, window, cx| {
        let bounds = bounds.intersect(&window.content_mask().bounds);
        if !bounds.is_empty() { cx.default_global::<Overlays>().0.entry(window.window_handle().window_id()).or_default().push(bounds.dilate(px(8.))); }
    }, |_, _, _, _| {})
        .absolute().inset_0()
}

pub fn clip(bounds: Bounds<Pixels>, window: &Window, cx: &gpui::App) -> Bounds<Pixels> {
    let overlays = cx.try_global::<Overlays>().and_then(|o| o.0.get(&window.window_handle().window_id()));
    clip_overlays(bounds, overlays.map_or(&[], Vec::as_slice))
}

fn clip_overlays(mut bounds: Bounds<Pixels>, overlays: &[Bounds<Pixels>]) -> Bounds<Pixels> {
    use gpui::{point, size, px};
    for overlay in overlays {
        if !bounds.intersects(overlay) { continue }
        let cut = bounds.intersect(overlay);
        // A native host has a rectangular clip. Keep its largest uncovered rectangle.
        bounds = [Bounds::new(bounds.origin, size(bounds.size.width, cut.top() - bounds.top())),
            Bounds::new(point(bounds.left(), cut.bottom()), size(bounds.size.width, bounds.bottom() - cut.bottom())),
            Bounds::new(bounds.origin, size(cut.left() - bounds.left(), bounds.size.height)),
            Bounds::new(point(cut.right(), bounds.top()), size(bounds.right() - cut.right(), bounds.size.height))]
            .into_iter().max_by(|a, b| (f32::from(a.size.width) * f32::from(a.size.height)).total_cmp(&(f32::from(b.size.width) * f32::from(b.size.height)))).unwrap();
        if bounds.size.width <= px(0.) || bounds.size.height <= px(0.) { break }
    }
    bounds
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, size, px};
    fn rect(x: f32, y: f32, w: f32, h: f32) -> Bounds<Pixels> { Bounds::new(point(px(x), px(y)), size(px(w), px(h))) }

    #[test]
    fn overlays_clip_only_intersecting_previews() {
        let first = rect(370., 207., 820., 160.); let second = rect(370., 409., 820., 160.);
        let toolbar = rect(351., 324., 439., 56.);
        assert_eq!(clip_overlays(first, &[toolbar]), rect(370., 207., 820., 117.));
        assert_eq!(clip_overlays(second, &[toolbar]), second);
        assert_eq!(clip_overlays(first, &[]), first);
        assert!(clip_overlays(first, &[first]).is_empty());
        let slash = rect(363., 254., 261., 365.);
        let visible = clip_overlays(first, &[toolbar, slash]);
        assert!(!visible.is_empty()); assert!(!visible.intersects(&toolbar)); assert!(!visible.intersects(&slash));
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use native::Preview;

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod native {
    use super::*;
    use raw_window_handle::{HandleError, HasWindowHandle, RawWindowHandle, WindowHandle};
    use wry::{WebView, WebViewBuilder, dpi::{LogicalPosition, LogicalSize}};

    pub struct Preview { web: WebView, host: Host }

    impl Preview {
        pub fn new(window: &Window, html: String, callback: impl Fn(String) + 'static) -> Result<Self, String> {
            let host = Host::new(window)?;
            let web = WebViewBuilder::new().with_html(html).with_visible(false).with_focused(false)
                .with_incognito(true).with_general_autofill_enabled(false).with_devtools(false)
                .with_navigation_handler(|url| url == "about:blank" || url == "about:srcdoc")
                .with_new_window_req_handler(|_, _| wry::NewWindowResponse::Deny)
                .with_download_started_handler(|_, _| false)
                .with_ipc_handler(move |request| callback(request.body().clone()))
                .build_as_child(&host).map_err(|e| e.to_string())?;
            Ok(Self { web, host })
        }

        pub fn script(&self, js: &str) -> Result<(), String> { self.web.evaluate_script(js).map_err(|e| e.to_string()) }
        pub fn inspect(&self, js: &str) { let _ = self.web.evaluate_script_with_callback(js, |value| eprintln!("html: {value}")); }

        pub fn hide(&self) { let _ = self.web.set_visible(false); self.host.show(false); }

        pub fn place(&self, bounds: Bounds<Pixels>, viewport: Bounds<Pixels>, window: &Window) {
            let clip = bounds.intersect(&viewport);
            if clip.size.width <= gpui::px(1.) || clip.size.height <= gpui::px(1.) { self.hide(); return }
            self.host.place(clip, window);
            let rect = wry::Rect { position: LogicalPosition::new(f32::from(bounds.left() - clip.left()) as f64, f32::from(bounds.top() - clip.top()) as f64).into(),
                size: LogicalSize::new(f32::from(bounds.size.width) as f64, f32::from(bounds.size.height) as f64).into() };
            let _ = self.web.set_bounds(rect);
            let _ = self.web.set_visible(true);
            self.host.show(true);
        }
    }

    #[cfg(target_os = "macos")]
    struct Host { view: *mut objc::runtime::Object }

    #[cfg(target_os = "macos")]
    impl Host {
        fn new(window: &Window) -> Result<Self, String> {
            use objc::{class, msg_send, sel, sel_impl};
            use objc::runtime::{Object, YES};
            let handle = HasWindowHandle::window_handle(window).map_err(|e| e.to_string())?;
            let RawWindowHandle::AppKit(h) = handle.as_raw() else { return Err("Expected AppKit window".into()) };
            unsafe {
                let view: *mut Object = msg_send![class!(NSView), new];
                let _: () = msg_send![view, setWantsLayer:YES];
                let layer: *mut Object = msg_send![view, layer];
                let _: () = msg_send![layer, setMasksToBounds:YES];
                let _: () = msg_send![h.ns_view.as_ptr() as *mut Object, addSubview:view];
                Ok(Self { view })
            }
        }

        fn place(&self, b: Bounds<Pixels>, _: &Window) {
            use core_graphics::geometry::{CGPoint, CGRect, CGSize};
            use objc::{msg_send, sel, sel_impl};
            use objc::runtime::{Object, BOOL, YES};
            unsafe {
                let parent: *mut Object = msg_send![self.view, superview];
                let base: CGRect = msg_send![parent, bounds];
                let flipped: BOOL = msg_send![parent, isFlipped];
                let y = if flipped == YES { f32::from(b.top()) as f64 } else { base.size.height - f32::from(b.bottom()) as f64 };
                let frame = CGRect::new(&CGPoint::new(f32::from(b.left()) as f64, y), &CGSize::new(f32::from(b.size.width) as f64, f32::from(b.size.height) as f64));
                let _: () = msg_send![self.view, setFrame:frame];
            }
        }

        fn show(&self, on: bool) { use objc::{msg_send, sel, sel_impl}; use objc::runtime::{NO, YES}; unsafe { let _: () = msg_send![self.view, setHidden:if on { NO } else { YES }]; } }
    }

    #[cfg(target_os = "macos")]
    impl HasWindowHandle for Host {
        fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
            let h = raw_window_handle::AppKitWindowHandle::new(std::ptr::NonNull::new(self.view.cast()).unwrap());
            Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::AppKit(h)) })
        }
    }

    #[cfg(target_os = "macos")]
    impl Drop for Host {
        fn drop(&mut self) { use objc::{msg_send, sel, sel_impl}; unsafe {
            let _: () = msg_send![self.view, removeFromSuperview];
            let _: () = msg_send![self.view, release];
        } }
    }

    #[cfg(target_os = "windows")]
    struct Host { hwnd: windows::Win32::Foundation::HWND }

    #[cfg(target_os = "windows")]
    impl Host {
        fn new(window: &Window) -> Result<Self, String> {
            use windows::{core::w, Win32::{Foundation::HWND, UI::WindowsAndMessaging::*}};
            let handle = HasWindowHandle::window_handle(window).map_err(|e| e.to_string())?;
            let RawWindowHandle::Win32(h) = handle.as_raw() else { return Err("Expected Win32 window".into()) };
            let hwnd = unsafe { CreateWindowExW(WINDOW_EX_STYLE::default(), w!("STATIC"), w!(""), WS_CHILD | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                0, 0, 1, 1, Some(HWND(h.hwnd.get() as *mut _)), None, None, None) }.map_err(|e| e.to_string())?;
            Ok(Self { hwnd })
        }

        fn place(&self, b: Bounds<Pixels>, window: &Window) {
            use windows::Win32::UI::WindowsAndMessaging::*;
            let scale = window.scale_factor();
            unsafe { let _ = SetWindowPos(self.hwnd, None, (f32::from(b.left()) * scale).round() as i32, (f32::from(b.top()) * scale).round() as i32,
                (f32::from(b.size.width) * scale).round() as i32, (f32::from(b.size.height) * scale).round() as i32, SWP_NOACTIVATE | SWP_NOZORDER); }
        }

        fn show(&self, on: bool) { use windows::Win32::UI::WindowsAndMessaging::*; unsafe { let _ = ShowWindow(self.hwnd, if on { SW_SHOWNOACTIVATE } else { SW_HIDE }); } }
    }

    #[cfg(target_os = "windows")]
    impl HasWindowHandle for Host {
        fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
            let h = raw_window_handle::Win32WindowHandle::new(std::num::NonZeroIsize::new(self.hwnd.0 as isize).unwrap());
            Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Win32(h)) })
        }
    }

    #[cfg(target_os = "windows")]
    impl Drop for Host { fn drop(&mut self) { unsafe { let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(self.hwnd); } } }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub struct Preview;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
impl Preview {
    pub fn new(_: &Window, _: String, _: impl Fn(String) + 'static) -> Result<Self, String> { Err(crate::settings::text("当前系统不支持").into()) }
    pub fn script(&self, _: &str) -> Result<(), String> { Ok(()) }
    pub fn inspect(&self, _: &str) {}
    pub fn hide(&self) {}
    pub fn place(&self, _: Bounds<Pixels>, _: Bounds<Pixels>, _: &Window) {}
}
