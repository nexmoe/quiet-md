//! The text drawing widget's source and record. Rendering belongs to the app.

use serde_json::{Map, Value};

pub const COMPONENT_ID: &str = "blk_631fefbbae02400430b8f9f4";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiagramLayout { pub zoom: f32, pub split: f32, pub pan: [f32; 2] }

impl Default for DiagramLayout {
    fn default() -> Self { Self { zoom: 1., split: 0.5, pan: [0., 0.] } }
}

impl DiagramLayout {
    pub fn normalized(self) -> Self {
        let finite = |n: f32, default: f32| if n.is_finite() { n } else { default };
        Self { zoom: finite(self.zoom, 1.).clamp(0.25, 8.), split: finite(self.split, 0.5).clamp(0.2, 0.8),
            pan: self.pan.map(|n| finite(n, 0.).clamp(-1_000_000., 0.)) }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DiagramView {
    #[default]
    CodeChart,
    Chart,
    Code,
}

impl DiagramView {
    pub fn token(self) -> &'static str {
        match self { Self::CodeChart => "codeChart", Self::Chart => "chart", Self::Code => "code" }
    }

    pub fn parse(s: &str) -> Self {
        match s { "chart" => Self::Chart, "code" => Self::Code, _ => Self::CodeChart }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextDiagram {
    pub source: String,
    pub view: DiagramView,
    pub theme: String,
    /// Unmodelled widget settings survive editing and saving.
    pub record: Map<String, Value>,
    /// An imported add-on keeps its tag and attributes, including its identity.
    pub envelope: Option<(String, Vec<(String, String)>)>,
}

impl Default for TextDiagram {
    fn default() -> Self { Self::new("") }
}

impl TextDiagram {
    pub fn new(source: impl Into<String>) -> Self {
        Self { source: source.into(), view: DiagramView::default(), theme: "default".into(), record: Map::new(), envelope: None }
    }

    pub fn from_record(record: &str) -> Option<Self> {
        let Value::Object(mut record) = serde_json::from_str(record).ok()? else { return None };
        let source = record.remove("data")?.as_str()?.to_string();
        let view = record.remove("view").and_then(|v| v.as_str().map(DiagramView::parse)).unwrap_or_default();
        let theme = record.remove("theme").and_then(|v| v.as_str().map(str::to_string)).unwrap_or_else(|| "default".into());
        Some(Self { source, view, theme, record, envelope: None })
    }

    pub fn record_json(&self) -> String {
        let mut record = self.record.clone();
        record.insert("data".into(), Value::String(self.source.clone()));
        record.insert("view".into(), Value::String(self.view.token().into()));
        record.insert("theme".into(), Value::String(self.theme.clone()));
        Value::Object(record).to_string()
    }

    pub fn layout(&self) -> DiagramLayout {
        let get = |key: &str| self.record.get("quiet_md").and_then(|v| v.get(key));
        let number = |key: &str, default: f32| get(key).and_then(Value::as_f64).map_or(default, |v| v as f32);
        let pan = get("pan").and_then(Value::as_array);
        DiagramLayout { zoom: number("zoom", 1.), split: number("split", 0.5),
            pan: std::array::from_fn(|i| pan.and_then(|p| p.get(i)).and_then(Value::as_f64).unwrap_or(0.) as f32) }.normalized()
    }

    pub fn set_layout(&mut self, layout: DiagramLayout) {
        let layout = layout.normalized();
        if layout == DiagramLayout::default() && !self.record.contains_key("quiet_md") { return }
        let value = self.record.entry("quiet_md").or_insert_with(|| Value::Object(Map::new()));
        if !value.is_object() { *value = Value::Object(Map::new()); }
        let settings = value.as_object_mut().unwrap();
        for key in ["zoom", "split", "pan"] { settings.remove(key); }
        if layout != DiagramLayout::default() {
            settings.insert("zoom".into(), serde_json::json!(layout.zoom));
            settings.insert("split".into(), serde_json::json!(layout.split));
            settings.insert("pan".into(), serde_json::json!(layout.pan));
        }
        if settings.is_empty() { self.record.remove("quiet_md"); }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widget_record_keeps_unknown_settings() {
        let mut d = TextDiagram::from_record(r#"{"data":"graph TD; A-->B","theme":"forest","view":"chart","scale":1.25,"future":{"a":true}}"#).unwrap();
        d.source.push_str("; B-->C");
        let record: Value = serde_json::from_str(&d.record_json()).unwrap();
        assert_eq!(record["scale"], 1.25);
        assert_eq!(record["future"]["a"], true);
        assert_eq!(record["data"], d.source);
        assert_eq!(TextDiagram::from_record(&d.record_json()).unwrap(), d);
        assert!(TextDiagram::from_record(r#"{"data":42}"#).is_none());
    }
}
