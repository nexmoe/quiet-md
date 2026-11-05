//! Pele's native Rust port (pele-rs). The editor and HTML export use the same renderer.

use fdl_core::diagram::TextDiagram;
use crate::editor_theme::{self as theme, Theme, craft};
use gpui::{Hsla, Rgba};


pub fn background(name: &str) -> Hsla {
    if name == "dark" { crate::theme::light(|| theme::c(craft::N900)) } else { Theme::bg_body() }
}

pub fn palette(name: &str) -> pele_rs::Palette {
    // The explicit dark preset keeps its colours; the other presets follow the app.
    if name == "dark" { crate::theme::light(|| resolved_palette(name)) } else { resolved_palette(name) }
}

fn resolved_palette(name: &str) -> pele_rs::Palette {
    let color = |c: Hsla| {
        let c = Rgba::from(c);
        format!("rgba({},{},{},{})", (c.r * 255.).round(), (c.g * 255.).round(), (c.b * 255.).round(), c.a)
    };
    let series = ["blue", "orange", "green", "red", "purple", "yellow", "light-blue", "gray"]
        .map(|s| color(theme::text_color(s).unwrap_or_else(Theme::primary)));
    let mut p = pele_rs::Palette {
        bg: color(background(name)), fg: color(Theme::text()), muted: color(Theme::text_caption()),
        line: color(Theme::text_caption()), surface: color(Theme::bg_overlay()), surface_alt: color(Theme::fill_hover()),
        border: color(Theme::text_placeholder()), accent: color(Theme::primary()), critical: color(Theme::danger()), series,
    };
    match name {
        "dark" => {
            p.fg = color(theme::c(craft::N100));
            p.muted = color(theme::c(craft::N400)); p.line = p.muted.clone();
            p.surface = color(theme::c(craft::N800)); p.surface_alt = color(theme::c(craft::N700));
            p.border = color(theme::c(craft::N500));
        }
        "forest" => {
            p.accent = color(theme::text_color("green").unwrap_or_else(Theme::primary));
            p.surface = color(theme::bg_color("light-green").unwrap_or_else(Theme::bg_overlay));
        }
        "neutral" => { p.accent = p.fg.clone(); p.series = std::array::from_fn(|_| p.muted.clone()); }
        _ => {}
    }
    p
}

pub fn render(source: &str, width: f32, palette: pele_rs::Palette, now: u64) -> Result<pele_rs::Rendered, String> {
    let options = pele_rs::RenderOptions {
        palette: Some(palette), max_width: Some(f64::from(width.max(160.))), responsive: Some(false),
        // Fixed source direction survives resizing, as in the widget's chart view.
        auto_direction: false, font_family: Some("Arial".into()), font_family_mono: Some("Courier New".into()),
        links: false, images: false, now: Some(now as f64), ..Default::default()
    };
    let result = pele_rs::render(source, &options).map_err(|e| {
        if e.line > 0 { format!("{}:{}: {}\n{}", e.line, e.column, e.message, e.snippet) } else { e.message }
    })?;
    // The bitmap decoder must not allocate an unbounded surface for a very wide chart.
    if !result.width.is_finite() || !result.height.is_finite() || result.width <= 0. || result.height <= 0.
        || result.width > 8192. || result.height > 8192. || result.width * result.height > 4_000_000. {
        return Err(crate::settings::text("图表过大，请拆分为多个图表").into());
    }
    Ok(result)
}

pub fn export(d: &TextDiagram, width: f32) -> Result<String, String> {
    render(&d.source, width, palette(&d.theme), crate::time::now_ms()).map(|r| r.svg)
}

fn raster_scale(width: f32, height: f32, requested: f32) -> f32 {
    requested.max(1.).min(8192. / width).min(8192. / height).min((16_000_000. / (width * height)).sqrt())
}

pub fn raster(renderer: gpui::SvgRenderer, svg: &str, width: f32, height: f32, requested: f32) -> Result<std::sync::Arc<gpui::RenderImage>, String> {
    let pixmap = renderer.render_pixmap(svg.as_bytes(), gpui::SvgSize::ScaleFactor(raster_scale(width, height, requested))).map_err(|e| e.to_string())?;
    let (w, h) = (pixmap.width(), pixmap.height());
    let mut bytes = pixmap.take();
    // GPUI expects BGRA without premultiplied alpha.
    for p in bytes.as_chunks_mut::<4>().0 {
        p.swap(0, 2);
        if p[3] > 0 { let a = p[3] as f32 / 255.; for c in &mut p[..3] { *c = (*c as f32 / a) as u8; } }
    }
    let buffer = image::ImageBuffer::from_raw(w, h, bytes).ok_or("Invalid diagram raster")?;
    Ok(std::sync::Arc::new(gpui::RenderImage::new(vec![image::Frame::new(buffer)])))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLES: &[(&str, &str)] = &[
        ("流程图", "flowchart TD\n    A[开始] --> B{判断}\n    B -->|是| C[处理]\n    B -->|否| D[结束]\n    C --> D"),
        ("时序图", "sequenceDiagram\n    participant A as 客户端\n    participant B as 服务端\n    A->>B: 请求\n    B-->>A: 响应"),
        ("类图", "classDiagram\n    class Animal {\n        +String name\n        +speak()\n    }\n    Animal <|-- Cat"),
        ("状态图", "stateDiagram-v2\n    [*] --> 待处理\n    待处理 --> 进行中\n    进行中 --> 已完成\n    已完成 --> [*]"),
        ("实体关系图", "erDiagram\n    USER ||--o{ NOTE : owns\n    USER {\n        string name\n    }\n    NOTE {\n        string title\n    }"),
        ("甘特图", "gantt\n    title 项目计划\n    dateFormat YYYY-MM-DD\n    section 开发\n    设计 :a, 2026-01-01, 3d\n    实现 :after a, 5d"),
        ("饼图", "pie showData\n    title 时间分配\n    \"开发\" : 60\n    \"设计\" : 25\n    \"测试\" : 15"),
        ("思维导图", "mindmap\n    root((项目))\n        设计\n        开发\n        测试"),
        ("时间轴", "timeline\n    title 项目进度\n    设计 : 原型\n    开发 : 实现\n    发布 : 上线"),
        ("用户旅程", "journey\n    title 使用旅程\n    section 开始\n      打开应用: 5: 用户\n      创建笔记: 5: 用户"),
        ("Git 图", "gitGraph\n    commit\n    branch feature\n    commit\n    checkout main\n    merge feature"),
        ("象限图", "quadrantChart\n    title 优先级\n    x-axis 低成本 --> 高成本\n    y-axis 低价值 --> 高价值\n    功能A: [0.3, 0.8]\n    功能B: [0.7, 0.4]"),
    ];

    fn example_source(index: usize, english: bool) -> String {
        let mut source = EXAMPLES[index].1.to_string();
        if english {
            for (zh, en) in [("待处理", "Pending"), ("进行中", "InProgress"), ("已完成", "Done"), ("项目计划", "Project plan"),
                ("项目进度", "Project timeline"), ("时间分配", "Time allocation"), ("使用旅程", "User journey"),
                ("打开应用", "Open app"), ("创建笔记", "Create note"), ("客户端", "Client"), ("服务端", "Server"),
                ("低成本", "Low cost"), ("高成本", "High cost"), ("低价值", "Low value"), ("高价值", "High value"),
                ("功能A", "FeatureA"), ("功能B", "FeatureB"), ("优先级", "Priority"), ("开始", "Start"), ("判断", "Decision"),
                ("处理", "Process"), ("结束", "End"), ("是", "Yes"), ("否", "No"), ("请求", "Request"), ("响应", "Response"),
                ("开发", "Development"), ("设计", "Design"), ("实现", "Implementation"), ("测试", "Testing"), ("项目", "Project"),
                ("原型", "Prototype"), ("发布", "Release"), ("上线", "Launch"), ("用户", "User")] { source = source.replace(zh, en); }
        }
        source
    }

    #[test]
    fn preview_raster_uses_device_pixels_and_bgra_colors() {
        let renderer = gpui::SvgRenderer::new(std::sync::Arc::new(crate::assets::Assets));
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32"><rect width="64" height="32" fill="#ff0000"/></svg>"##;
        let image = raster(renderer.clone(), svg, 64., 32., 2.).unwrap();
        assert_eq!(image.size(0).width.0, 128);
        assert_eq!(image.size(0).height.0, 64);
        assert_eq!(&image.as_bytes(0).unwrap()[..4], &[0, 0, 255, 255]);
        assert_eq!(raster(renderer, svg, 64., 32., 4.).unwrap().size(0).width.0, 256);
        let scale = raster_scale(1000., 1000., 16.);
        assert!(1000. * 1000. * scale * scale <= 16_000_000.);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn emoji_fallback_preserves_the_latin_label_and_color() {
        let renderer = gpui::SvgRenderer::new(std::sync::Arc::new(crate::assets::Assets));
        let label = |suffix: &str| format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="56"><text x="8" y="32" font-family="Arial" font-size="16">Unicode works too: {suffix}</text></svg>"#);
        let latin = renderer.render_pixmap(label("").as_bytes(), gpui::SvgSize::ScaleFactor(2.)).unwrap();
        let emoji = renderer.render_pixmap(label("🌋").as_bytes(), gpui::SvgSize::ScaleFactor(2.)).unwrap();
        // Adding the official mindmap's emoji must not replace the Latin prefix with LastResort boxes.
        for y in 0..latin.height() { for x in 0..260 {
            assert_eq!(latin.pixel(x, y), emoji.pixel(x, y), "Latin prefix changed at {x},{y}");
        } }
        assert!(emoji.pixels().iter().any(|p| p.alpha() > 128 && p.red().abs_diff(p.blue()) > 40), "missing color emoji");
    }

    #[test]
    fn dark_preset_stays_dark_when_the_app_is_dark() {
        // Appearance is process-wide; isolate it from the other tests.
        if std::env::var_os("QUIET_MD_DIAGRAM_APPEARANCE_TEST").is_none() {
            let result = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "diagram::tests::dark_preset_stays_dark_when_the_app_is_dark"])
                .env("QUIET_MD_DIAGRAM_APPEARANCE_TEST", "1")
                .env("QUIET_MD_HOME", std::env::temp_dir().join(format!("quiet-md-diagram-appearance-{}", std::process::id())))
                .output().unwrap();
            assert!(result.status.success(), "{}\n{}", String::from_utf8_lossy(&result.stdout), String::from_utf8_lossy(&result.stderr));
            return;
        }
        crate::theme::set_dark(true);
        assert_eq!(palette("dark"), crate::theme::light(|| palette("dark")));
        assert_eq!(background("dark"), crate::theme::light(|| background("dark")));
        assert_ne!(palette("default"), crate::theme::light(|| palette("default")));
        assert_ne!(background("default"), crate::theme::light(|| background("default")));
    }

    #[test]
    fn representative_diagram_types_render_as_standalone_svg() {
        for (i, (name, _)) in EXAMPLES.iter().enumerate() { for english in [false, true] {
            let source = example_source(i, english);
            let r = render(&source, 640., pele_rs::Palette::light(), 0).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(r.width > 0. && r.height > 0., "{name}");
            assert!(!r.svg.contains("var("), "{name}: unresolved CSS variables");
            assert!(!r.svg.contains("<foreignObject"), "{name}: browser-only labels");
        } }
    }

    #[test]
    fn invalid_and_oversized_source_is_reported_without_losing_text() {
        assert!(render("flowchart TD\nA[broken", 640., pele_rs::Palette::light(), 0).is_err());
        assert!(render(&"x".repeat(50_001), 640., pele_rs::Palette::light(), 0).is_err());
        let r = render("flowchart LR\nA[<script>alert&lt;1&gt;</script>]", 640., pele_rs::Palette::light(), 0);
        if let Ok(r) = r { assert!(!r.svg.contains("<script>")); }
    }
}
