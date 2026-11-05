//! Off-screen PDF export using the installed WebView2 Runtime, on its own STA thread.

use anyhow::{Context, Result, bail};
use std::{path::{Path, PathBuf}, sync::mpsc, time::{Duration, Instant}};
use webview2_com::{Microsoft::Web::WebView2::Win32::*, *};
use windows::{core::{BOOL, HSTRING, Interface, w}, Win32::{Foundation::{E_FAIL, E_POINTER, HWND, RECT},
    System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED}, UI::WindowsAndMessaging::*}};

struct Apartment;
impl Drop for Apartment { fn drop(&mut self) { unsafe { CoUninitialize(); } } }
struct Host(HWND);
impl Drop for Host { fn drop(&mut self) { unsafe { let _ = DestroyWindow(self.0); } } }
struct Controller(ICoreWebView2Controller);
impl Drop for Controller { fn drop(&mut self) { unsafe { let _ = self.0.Close(); } } }
struct Files(PathBuf);
impl Drop for Files {
    fn drop(&mut self) {
        // The runtime releases its isolated profile shortly after the controller closes.
        for _ in 0..20 {
            if std::fs::remove_dir_all(&self.0).is_ok() || !self.0.exists() { break }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

fn pump() {
    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    std::thread::sleep(Duration::from_millis(5));
}

fn wait<T>(rx: mpsc::Receiver<windows::core::Result<T>>, deadline: Instant) -> Result<T> {
    loop {
        match rx.try_recv() {
            Ok(result) => return result.map_err(Into::into),
            Err(mpsc::TryRecvError::Disconnected) => bail!("PDF callback disconnected"),
            Err(mpsc::TryRecvError::Empty) => {}
        }
        if Instant::now() >= deadline { bail!("PDF export timed out") }
        pump();
    }
}

fn file_url(path: &Path) -> Result<String> {
    let path = path.to_str().context("Invalid PDF temporary path")?.replace('\\', "/");
    let mut url = String::from("file:///");
    for b in path.bytes() {
        if b.is_ascii_alphanumeric() || b"/:.-_~".contains(&b) { url.push(b as char); }
        else { use std::fmt::Write; let _ = write!(url, "%{b:02X}"); }
    }
    Ok(url)
}

pub fn export(html: String, path: PathBuf) -> std::result::Result<bool, String> {
    std::thread::Builder::new().name("quiet-md-pdf".into()).spawn(move || pdf(&html, &path))
        .map_err(|e| e.to_string())?.join().map_err(|_| "PDF worker stopped".to_string())?
        .map(|()| true).map_err(|e| format!("{e:#}"))
}

fn pdf(html: &str, path: &Path) -> Result<()> {
    let files = Files(std::env::temp_dir().join(format!("quiet-md-pdf-{}", uuid::Uuid::new_v4())));
    std::fs::create_dir(&files.0)?;
    let input = files.0.join("document.html");
    std::fs::write(&input, html)?;
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?; }
    let _apartment = Apartment;
    let deadline = Instant::now() + Duration::from_secs(60);
    unsafe {
        let host = Host(CreateWindowExW(Default::default(), w!("STATIC"), w!("quiet.md PDF"), WS_POPUP,
            -20000, -20000, 800, 1000, None, None, None, None)?);
        let options = CoreWebView2EnvironmentOptions::default();
        options.set_additional_browser_arguments("--disable-background-networking --disable-component-update --no-first-run".into());
        let options: ICoreWebView2EnvironmentOptions = options.into();
        let profile = HSTRING::from(files.0.join("profile").as_os_str());
        let (tx, rx) = mpsc::channel();
        let handler = CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(move |result, environment| {
            let _ = tx.send(result.and_then(|()| environment.ok_or_else(|| E_POINTER.into())));
            Ok(())
        }));
        CreateCoreWebView2EnvironmentWithOptions(None, &profile, &options, &handler).context("Microsoft Edge WebView2 Runtime")?;
        let environment = wait(rx, deadline).context("Microsoft Edge WebView2 Runtime")?;
        let (tx, rx) = mpsc::channel();
        environment.CreateCoreWebView2Controller(host.0, &CreateCoreWebView2ControllerCompletedHandler::create(Box::new(move |result, controller| {
            let _ = tx.send(result.and_then(|()| controller.ok_or_else(|| E_POINTER.into())));
            Ok(())
        })))?;
        let controller = Controller(wait(rx, deadline)?);
        controller.0.SetBounds(RECT { left: 0, top: 0, right: 666, bottom: 1000 })?;
        controller.0.SetIsVisible(true)?;
        let web = controller.0.CoreWebView2()?;
        let settings = web.Settings()?;
        settings.SetIsScriptEnabled(false)?;
        settings.SetAreDefaultScriptDialogsEnabled(false)?;
        settings.SetAreDevToolsEnabled(false)?;
        settings.SetAreDefaultContextMenusEnabled(false)?;
        let (tx, rx) = mpsc::channel();
        let mut navigation = 0;
        web.add_NavigationCompleted(&NavigationCompletedEventHandler::create(Box::new(move |_, args| {
            let result = (|| {
                let args = args.ok_or_else(|| windows::core::Error::from(E_POINTER))?;
                let mut success = BOOL::default();
                args.IsSuccess(&mut success)?;
                if !success.as_bool() { return Err(E_FAIL.into()) }
                Ok(())
            })();
            let _ = tx.send(result);
            Ok(())
        })), &mut navigation)?;
        web.Navigate(&HSTRING::from(file_url(&input)?))?;
        wait(rx, deadline)?;
        web.remove_NavigationCompleted(navigation)?;
        let settle = Instant::now() + Duration::from_millis(300);
        while Instant::now() < settle { pump(); }
        let print = environment.cast::<ICoreWebView2Environment6>()?.CreatePrintSettings()?;
        print.SetPageWidth(595.28 / 72.)?; print.SetPageHeight(841.89 / 72.)?;
        print.SetMarginLeft(48. / 72.)?; print.SetMarginRight(48. / 72.)?;
        print.SetMarginTop(40. / 72.)?; print.SetMarginBottom(40. / 72.)?;
        print.SetShouldPrintBackgrounds(true)?; print.SetShouldPrintHeaderAndFooter(false)?;
        let (tx, rx) = mpsc::channel();
        web.cast::<ICoreWebView2_7>()?.PrintToPdf(&HSTRING::from(path.as_os_str()), &print,
            &PrintToPdfCompletedHandler::create(Box::new(move |result, success| {
                let _ = tx.send(result.and_then(|()| if success { Ok(()) } else { Err(E_FAIL.into()) }));
                Ok(())
            })))?;
        wait(rx, deadline)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_html_and_svg_to_pdf_without_a_visible_window() {
        let path = std::env::temp_dir().join(format!("quiet-md-pdf-test-{}.pdf", uuid::Uuid::new_v4()));
        let html = "<!doctype html><meta charset='utf-8'><h1>quiet.md 中文 PDF</h1><svg xmlns='http://www.w3.org/2000/svg' width='400' height='100'><rect x='2' y='2' width='396' height='96' fill='#eef' stroke='#123'/><text x='30' y='55'>Native Mermaid SVG</text></svg>";
        assert!(export(html.into(), path.clone()).unwrap());
        let bytes = std::fs::read(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert!(bytes.starts_with(b"%PDF-")); assert!(bytes.len() > 1000);
    }
}
