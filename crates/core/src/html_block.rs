//! HTML widget records. The app owns the sandboxed preview, not the document engine.

use serde_json::{Map, Value};

pub const COMPONENT_ID: &str = "blk_6358a421bca0001c190a9805";
pub const MAX_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HtmlBlock {
    pub source: String,
    pub record: Map<String, Value>,
    pub envelope: Option<(String, Vec<(String, String)>)>,
}

impl HtmlBlock {
    pub fn new(source: impl Into<String>) -> Self { Self { source: source.into(), ..Self::default() } }

    pub fn from_record(json: &str) -> Option<Self> {
        let Value::Object(mut record) = serde_json::from_str(json).ok()? else { return None };
        let source = record.remove("html")?.as_str()?.to_string();
        Some(Self { source, record, envelope: None })
    }

    pub fn record_json(&self) -> String {
        let mut record = self.record.clone();
        record.insert("html".into(), Value::String(self.source.clone()));
        Value::Object(record).to_string()
    }

    pub fn markup(&self, xml: &str) -> String {
        let escape = |s: &str| s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\n', "&#10;").replace('\r', "&#13;");
        format!("<iframe title=\"HTML\" sandbox=\"allow-scripts\" referrerpolicy=\"no-referrer\" style=\"border:0;width:100%;height:{}px\" data-quiet-html=\"{}\" srcdoc=\"{}\"></iframe>", self.height(), escape(xml), escape(&self.source))
    }

    pub fn height(&self) -> f32 { self.number("height", 360., 160., 2400.) }
    pub fn split(&self) -> f32 { self.number("split", 0.5, 0.2, 0.8) }

    fn number(&self, key: &str, default: f32, min: f32, max: f32) -> f32 {
        self.record.get("quiet_md").and_then(|v| v.get(key)).and_then(Value::as_f64)
            .filter(|v| v.is_finite()).map_or(default, |v| (v as f32).clamp(min, max))
    }

    pub fn set_layout(&mut self, height: f32, split: f32) {
        let settings = self.record.entry("quiet_md").or_insert_with(|| Value::Object(Map::new()));
        if !settings.is_object() { *settings = Value::Object(Map::new()); }
        let settings = settings.as_object_mut().unwrap();
        if height.is_finite() { settings.insert("height".into(), Value::from(height.clamp(160., 2400.))); }
        if split.is_finite() { settings.insert("split".into(), Value::from(split.clamp(0.2, 0.8))); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{docxml, editor::core::{Core, Pos, Selection}, markdown, model::*, paste};

    #[test]
    fn source_identity_and_unknown_record_survive_exports_and_undo() {
        let source = "<!doctype html>\n<style>p{color:red}</style><p>中文 & \"quotes\"</p><script>const s='</script>';</script>";
        let mut html = HtmlBlock::from_record(&serde_json::json!({"html":source,"future":{"revision":7}}).to_string()).unwrap();
        html.envelope = Some(("add-ons".into(), vec![("component-type-id".into(), COMPONENT_ID.into()), ("token".into(), "widget-token".into())]));
        html.set_layout(480., 0.65);
        let block = Block::new(BlockKind::Html(html.clone()), RichText::default());
        let id = block.id;
        let doc = Document { blocks: vec![block], ..Default::default() };
        let xml = docxml::serialize(&doc);
        assert_eq!(docxml::serialize(&docxml::parse(&xml).unwrap()), xml);
        let mut core = Core::new(doc.clone()); core.sel = Selection::caret(Pos::new(id, 0));
        let mut changed = html.clone(); changed.source = "<button onclick=\"this.textContent='ok'\">点击</button>".into();
        core.set_html(id, changed.clone(), true);
        core.undo(); assert_eq!(core.kind_of(id), Some(&BlockKind::Html(html.clone())));
        core.redo(); assert_eq!(core.kind_of(id), Some(&BlockKind::Html(changed)));
        let md = markdown::export(&doc, &Default::default()).text;
        assert!(!md.contains("<title>"));
        let imported = paste::markdown_to_blocks(&md, Default::default());
        assert_eq!(imported.iter().filter(|b| matches!(b.kind, BlockKind::Html(_))).count(), 1);
        assert_eq!(imported[0].kind, BlockKind::Html(html.clone()));
        let markup = format!("<section data-quiet-html=\"{}\">ignored preview</section>", xml.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;"));
        assert_eq!(paste::html_to_blocks(&markup)[0].kind, BlockKind::Html(html));
    }

    #[test]
    fn remote_only_or_invalid_widgets_stay_raw() {
        for record in ["{}", r#"{"html":42}"#, "not-json"] {
            let xml = format!("<add-ons component-type-id=\"{COMPONENT_ID}\" token=\"remote\" record='{}'/>", record);
            let doc = docxml::parse(&xml).unwrap();
            assert!(matches!(doc.blocks[0].kind, BlockKind::Raw { .. }));
            assert!(docxml::serialize(&doc).contains("remote"));
        }
    }

    #[test]
    fn layout_is_bounded_and_future_settings_are_preserved() {
        let mut html = HtmlBlock::from_record(r#"{"html":"","quiet_md":{"height":1e30,"split":-3,"future":true}}"#).unwrap();
        assert_eq!(html.height(), 2400.); assert_eq!(html.split(), 0.2);
        html.set_layout(f32::NAN, f32::INFINITY);
        assert_eq!(html.record["quiet_md"]["future"], true);
        assert!(HtmlBlock::from_record(&html.record_json()).is_some());
    }
}
