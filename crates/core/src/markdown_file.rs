//! Markdown file import and in-place export, shared by the library and local file windows.

use crate::model::{Block, BlockKind, Document, RichText};
use std::path::{Path, PathBuf};

/// Percent-escapes in a Markdown link (`%20` and friends).
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let Some(v) = std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok())
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| s.to_string())
}

/// Relative and `file://` image links of an imported file, made absolute.
fn resolve_images(blocks: &mut [Block], base: &Path) {
    for b in blocks.iter_mut() {
        resolve_images(&mut b.children, base);
        if let BlockKind::Table(t) = &mut b.kind {
            for cell in t.rows.iter_mut().flatten() {
                resolve_images(&mut cell.blocks, base);
            }
        }
        let BlockKind::Image { src, .. } = &mut b.kind else { continue };
        if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("data:") {
            continue;
        }
        let path = match src.strip_prefix("file://") {
            Some(p) => PathBuf::from(percent_decode(p)),
            None => base.join(percent_decode(src)),
        };
        *src = path.to_string_lossy().to_string();
    }
}

/// Import uses the file name as title; direct editing uses its first H1, if any.
pub fn read(path: &Path, filename_title: bool) -> std::io::Result<Document> {
    let bytes = std::fs::read(path)?;
    let text = String::from_utf8_lossy(&bytes);
    let text = text.trim_start_matches('\u{feff}');
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default().trim().to_string();
    let off = crate::paste::Off { file: true, ..Default::default() };
    let mut blocks = crate::paste::markdown_to_blocks(text, off);
    let mut title = if filename_title { RichText::plain(&stem) } else { RichText::default() };
    if blocks.first().is_some_and(|b| matches!(b.kind, BlockKind::Heading { level: 1, .. }) && (!filename_title || b.text.text.trim() == stem)) {
        let heading = blocks.remove(0);
        if !filename_title { title = heading.text; }
    }
    if let Some(dir) = path.parent() {
        resolve_images(&mut blocks, dir);
    }
    if blocks.is_empty() {
        blocks.push(Block::text(""));
    }
    Ok(Document { title, icon: None, cover: None, blocks })
}

/// Save through the existing Markdown exporter, retaining local image references.
pub fn write(path: &Path, doc: &Document) -> std::io::Result<Vec<u8>> {
    use std::io::Write;
    let mut doc = doc.clone();
    fn relative(blocks: &mut [Block], base: &Path) {
        for b in blocks {
            if let BlockKind::Image { src, .. } = &mut b.kind
                && let Ok(p) = Path::new(src).strip_prefix(base)
            { *src = p.to_string_lossy().replace('\\', "/"); }
            relative(&mut b.children, base);
            if let BlockKind::Table(t) = &mut b.kind { for cell in t.rows.iter_mut().flatten() { relative(&mut cell.blocks, base); } }
        }
    }
    relative(&mut doc.blocks, path.parent().unwrap_or(Path::new(".")));
    let text = crate::markdown::export(&doc, &crate::markdown::Options { assets: Some(""), preserve_paths: true, ..Default::default() }).text;
    let permissions = std::fs::metadata(path)?.permissions();
    if permissions.readonly() { return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "File is read-only")); }
    let temp = path.with_file_name(format!(".quiet-md-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;
        file.write_all(text.as_bytes())?;
        file.set_permissions(permissions)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temp, path)
    })();
    if result.is_err() { let _ = std::fs::remove_file(temp); }
    result?;
    Ok(text.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::{BlockKind, read, percent_decode, write};

    #[test]
    fn imports_markdown_file() {
        let dir = std::env::temp_dir().join(format!("fdl-import-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("周报.md");
        std::fs::write(&path, "\u{feff}# 周报\n\n第一段\n\n- a\n- b\n\n![图](https://e.com/a.png)\n\n## 小结\n").unwrap();
        let doc = read(&path, true).unwrap();
        assert_eq!(doc.title.text, "周报");
        let kinds: Vec<_> = doc.blocks.iter().map(|b| std::mem::discriminant(&b.kind)).collect();
        assert_eq!(kinds.len(), 5);
        assert_eq!(doc.blocks[0].text.text, "第一段");
        assert!(matches!(doc.blocks[1].kind, BlockKind::Bullet));
        assert!(matches!(&doc.blocks[3].kind, BlockKind::Image { src, .. } if src == "https://e.com/a.png"));
        assert!(matches!(doc.blocks[4].kind, BlockKind::Heading { level: 2, .. }));
        assert_eq!(percent_decode("a%20b%E5%9B%BE.png"), "a b图.png");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn previews_mdx_without_changing_source_or_copying_images() {
        let dir = std::env::temp_dir().join(format!("fdl-preview-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("images")).unwrap();
        let path = dir.join("本地预览.mdx");
        let image = dir.join("images/图 片.png");
        let source = "\u{feff}# 本地预览\r\n\r\n**正文**\r\n\r\n> ![图](images/%E5%9B%BE%20%E7%89%87.png)\r\n\r\n<Demo />\r\n";
        std::fs::write(&path, source).unwrap();
        std::fs::write(&image, b"preview image").unwrap();
        let doc = read(&path, false).unwrap();
        assert_eq!(doc.title.text, "本地预览");
        assert_eq!(doc.blocks[0].text.text, "正文");
        assert!(doc.blocks[0].text.runs.iter().any(|r| r.marks.bold));
        assert!(matches!(&doc.blocks[1].children[0].kind, BlockKind::Image { src, .. } if src.as_str() == image.to_string_lossy().as_ref()));
        assert!(doc.blocks.iter().any(|b| b.text.text == "<Demo />"));
        assert_eq!(std::fs::read_to_string(path).unwrap(), source);
        assert_eq!(std::fs::read(image).unwrap(), b"preview image");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn edits_local_markdown_and_keeps_relative_images() {
        use crate::editor::core::{Core, Pos, Selection};
        let dir = std::env::temp_dir().join(format!("fdl-edit-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("different-file-name.mdx");
        std::fs::write(&path, "# Actual title\n\nbody\n\n![image](photo.png)\n").unwrap();
        let mut core = Core::new(read(&path, false).unwrap());
        assert_eq!(core.doc.title.text, "Actual title");
        core.sel = Selection::caret(Pos::new(core.doc.blocks[0].id, 0));
        core.insert_text("updated ");
        let bytes = write(&path, &core.doc).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(String::from_utf8(bytes).unwrap(), "# Actual title\n\nupdated body\n\n![](photo.png)\n");
        std::fs::write(&path, "body without a heading\n").unwrap();
        let doc = read(&path, false).unwrap();
        assert!(doc.title.text.is_empty());
        write(&path, &doc).unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), "body without a heading\n");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
