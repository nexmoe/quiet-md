use fdl_core::{diagram::{DiagramLayout, DiagramView, TextDiagram, COMPONENT_ID}, docxml, editor::core::Core, markdown, model::*, paste};

fn drawing(doc: &Document) -> &TextDiagram {
    let BlockKind::Diagram(d) = &doc.blocks[0].kind else { panic!("expected drawing: {:?}", doc.blocks[0].kind) };
    d
}

#[test]
fn feishu_experiment_record_imports_with_source_and_view() {
    // Sanitized from the user-provided Feishu experiment; no document or author IDs.
    let sample: serde_json::Value = serde_json::from_str(include_str!("fixtures/feishu-text-drawing.json")).unwrap();
    assert_eq!(sample["block_type_id"], COMPONENT_ID);
    let record = sample["data"].to_string().replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;");
    let xml = format!("<add_ons component_type_id=\"{COMPONENT_ID}\" record=\"{record}\"/>");
    let doc = docxml::parse(&xml).unwrap();
    assert_eq!(drawing(&doc).source, sample["data"]["data"].as_str().unwrap());
    assert_eq!(drawing(&doc).view, DiagramView::CodeChart);
    let saved = docxml::serialize(&doc);
    assert_eq!(docxml::serialize(&docxml::parse(&saved).unwrap()), saved);
}

#[test]
fn native_xml_preserves_source_views_and_nested_cell_newlines() {
    let source = "flowchart LR\n    A[中文 & < > 😀] --> B\n\n";
    for view in [DiagramView::CodeChart, DiagramView::Chart, DiagramView::Code] {
        let mut d = TextDiagram::new(source);
        d.view = view; d.theme = "future-theme".into();
        d.record.insert("unknown".into(), serde_json::json!({"nested": [1, true]}));
        let doc = Document { blocks: vec![Block::new(BlockKind::Diagram(d.clone()), RichText::default())], ..Default::default() };
        let xml = docxml::serialize(&doc);
        let read = docxml::parse(&xml).unwrap();
        assert_eq!(drawing(&read), &d);
        assert_eq!(docxml::serialize(&read), xml);
        assert!(read.plain_text().contains(source));
        let body = xml.split_once('\n').unwrap().1;
        let cell = format!("<table><tbody><tr><td>{body}</td></tr></tbody></table>");
        let read = docxml::parse(&cell).unwrap();
        let BlockKind::Table(t) = &read.blocks[0].kind else { panic!() };
        assert!(matches!(&t.rows[0][0].blocks[0].kind, BlockKind::Diagram(d) if d.source == source));
        let saved = docxml::serialize(&read);
        assert!(saved.contains("&#10;"));
        assert_eq!(docxml::serialize(&docxml::parse(&saved).unwrap()), saved);
        assert!(markdown::export(&read, &Default::default()).text.contains("中文"));
    }
}

#[test]
fn feishu_record_keeps_identity_and_unknown_settings_when_edited() {
    let record = serde_json::json!({"data": "sequenceDiagram\nA->>B: hello", "view":"chart", "theme":"forest", "scale": 1.25, "future": {"version":2}}).to_string();
    let escaped = record.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;");
    let xml = format!("<add_ons component_type_id=\"{COMPONENT_ID}\" token=\"opaque-id\" record=\"{escaped}\"/>");
    let mut doc = docxml::parse(&xml).unwrap();
    let BlockKind::Diagram(d) = &mut doc.blocks[0].kind else { panic!() };
    d.source = "flowchart TD\nA-->B".into(); d.view = DiagramView::Code;
    let saved = docxml::serialize(&doc);
    let read = docxml::parse(&saved).unwrap();
    assert_eq!(drawing(&read).source, "flowchart TD\nA-->B");
    assert_eq!(drawing(&read).record["future"]["version"], 2);
    assert_eq!(drawing(&read).record["scale"], 1.25);
    assert!(saved.contains("token=\"opaque-id\""));
    assert_eq!(docxml::serialize(&read), saved);
    let malformed = format!("<add_ons component_type_id=\"{COMPONENT_ID}\" record=\"invalid\"/>");
    assert!(matches!(docxml::parse(&malformed).unwrap().blocks[0].kind, BlockKind::Raw { .. }));
    let opaque = "<readonly-block type=\"isv\" token=\"source-not-included\"/>";
    assert!(docxml::serialize(&docxml::parse(opaque).unwrap()).contains(opaque));
    let compact = format!("<text-drawing record=\"{escaped}\"/>");
    let read = docxml::parse(&compact).unwrap();
    assert_eq!(drawing(&read).source, "sequenceDiagram\nA->>B: hello");
    assert_eq!(drawing(&read).view, DiagramView::Chart);
    assert_eq!(drawing(&read).theme, "forest");
}

#[test]
fn markdown_and_html_import_keep_editable_source() {
    let source = "flowchart LR\nA[\"```\"] --> B[中文]";
    let doc = Document { blocks: vec![Block::new(BlockKind::Diagram(TextDiagram::new(source)), RichText::default())], ..Default::default() };
    let out = markdown::export(&doc, &Default::default());
    let read = Document { blocks: paste::markdown_to_blocks(&out.text, paste::Off::default()), ..Default::default() };
    assert_eq!(drawing(&read).source, source);
    let html = "<pre><code class=\"language-mermaid\">graph TD\nA --&gt; B</code></pre>";
    let read = Document { blocks: paste::html_to_blocks(html), ..Default::default() };
    assert_eq!(drawing(&read).source, "graph TD\nA --> B");
    let record = TextDiagram::new(source).record_json().replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;");
    let html = format!("<div data-component-type-id=\"{COMPONENT_ID}\" data-record=\"{record}\"><svg><text>flattened preview</text></svg></div>");
    let read = Document { blocks: paste::html_to_blocks(&html), ..Default::default() };
    assert_eq!(read.blocks.len(), 1);
    assert_eq!(drawing(&read).source, source);
}

#[test]
fn insertion_source_and_view_changes_undo_and_redo() {
    let mut c = Core::new(Document::default());
    let before = docxml::serialize(&c.doc);
    let id = c.insert_diagram(TextDiagram::new("graph TD\nA-->B"));
    let inserted = docxml::serialize(&c.doc);
    let BlockKind::Diagram(mut d) = c.block(id).unwrap().kind.clone() else { panic!() };
    d.source.push_str("\nB-->C"); d.view = DiagramView::Chart;
    assert!(c.set_diagram(id, d, false));
    let edited = docxml::serialize(&c.doc);
    assert!(c.undo()); assert_eq!(docxml::serialize(&c.doc), inserted);
    assert!(c.undo()); assert_eq!(docxml::serialize(&c.doc), before);
    assert!(c.redo()); assert!(c.redo()); assert_eq!(docxml::serialize(&c.doc), edited);
}

#[test]
fn layout_survives_save_reopen_and_undo_without_losing_unknown_settings() {
    let mut d = TextDiagram::new("graph LR; A-->B");
    d.record.insert("quiet_md".into(), serde_json::json!({"future": {"enabled": true}}));
    d.record.insert("scale".into(), serde_json::json!(1.25));
    let block = Block::new(BlockKind::Diagram(d), RichText::default());
    let id = block.id;
    let mut c = Core::new(Document { blocks: vec![block], ..Default::default() });
    let before = docxml::serialize(&c.doc);
    let layout = DiagramLayout { zoom: 3.5, split: 0.23, pan: [-123., -456.] };
    assert!(c.set_diagram_layout(id, layout, false));
    let saved = docxml::serialize(&c.doc);
    let reopened = docxml::parse(&saved).unwrap();
    assert_eq!(drawing(&reopened).layout(), layout);
    assert_eq!(drawing(&reopened).record["quiet_md"]["future"]["enabled"], true);
    assert_eq!(drawing(&reopened).record["scale"], 1.25);
    assert_eq!(docxml::serialize(&reopened), saved);
    assert!(!c.set_diagram_layout(id, layout, false));
    assert!(c.undo()); assert_eq!(docxml::serialize(&c.doc), before);
    assert!(c.redo()); assert_eq!(docxml::serialize(&c.doc), saved);
    assert!(c.set_diagram_layout(id, DiagramLayout { zoom: f32::INFINITY, split: -10., pan: [f32::NAN, 20.] }, false));
    assert_eq!(drawing(&c.doc).layout(), DiagramLayout { zoom: 1., split: 0.2, pan: [0., 0.] });
    assert!(c.set_diagram_layout(id, DiagramLayout::default(), false));
    assert_eq!(docxml::serialize(&c.doc), before);
}

#[test]
fn html_component_identity_survives_editing_and_xml_reopening() {
    let html = format!("<div data-component-type-id=\"{COMPONENT_ID}\" data-token=\"opaque&amp;id\" data-block-id=\"block-7\" aria-label=\"chart\" data-record='{{\"data\":\"graph TD; A--&gt;B\",\"future\":42}}'></div>");
    let mut doc = Document { blocks: paste::html_to_blocks(&html), ..Default::default() };
    let BlockKind::Diagram(d) = &mut doc.blocks[0].kind else { panic!() };
    d.source.push_str("; B-->C");
    let saved = docxml::serialize(&doc);
    let reopened = docxml::parse(&saved).unwrap();
    assert_eq!(drawing(&reopened), drawing(&doc));
    let attrs = &drawing(&reopened).envelope.as_ref().unwrap().1;
    for (key, value) in [("data-token", "opaque&id"), ("data-block-id", "block-7"), ("aria-label", "chart")] {
        assert!(attrs.iter().any(|(k, v)| k == key && v == value));
    }
    assert_eq!(drawing(&reopened).record["future"], 42);
    let malicious = html.replace("<div ", "<div data-quiet-md-envelope='[\"bad&gt;tag\",[]]' ");
    let doc = Document { blocks: paste::html_to_blocks(&malicious), ..Default::default() };
    assert_eq!(drawing(&doc).envelope.as_ref().unwrap().0, "add_ons");
    assert!(docxml::parse(&docxml::serialize(&doc)).is_ok());
    let duplicate = serde_json::json!(["add_ons", [["component_type_id", COMPONENT_ID], ["token", "first"], ["token", "duplicate"]]]).to_string().replace('"', "&quot;");
    let doc = Document { blocks: paste::html_to_blocks(&html.replace("<div ", &format!("<div data-quiet-md-envelope=\"{duplicate}\" "))), ..Default::default() };
    assert_eq!(drawing(&doc).envelope.as_ref().unwrap().1.iter().filter(|(k, _)| k == "token").count(), 0);
    assert!(docxml::parse(&docxml::serialize(&doc)).is_ok());
}

#[test]
fn wrapping_drawings_in_different_cells_keeps_the_cells_separate() {
    let doc = docxml::parse("<table><tbody><tr><td><text-drawing>graph TD; A--&gt;B</text-drawing></td><td><p>first</p><p>second</p></td></tr></tbody></table>").unwrap();
    let BlockKind::Table(t) = &doc.blocks[0].kind else { panic!() };
    let ids = [t.rows[0][0].blocks[0].id, t.rows[0][1].blocks[1].id];
    let mut c = Core::new(doc);
    c.turn_into(&ids, BlockKind::Quote);
    let BlockKind::Table(t) = &c.doc.blocks[0].kind else { panic!() };
    assert!(matches!(t.rows[0][0].blocks[0].kind, BlockKind::Quote));
    assert!(matches!(t.rows[0][0].blocks[0].children[0].kind, BlockKind::Diagram(_)));
    assert_eq!(t.rows[0][1].blocks.len(), 2);
    assert!(matches!(t.rows[0][1].blocks[1].kind, BlockKind::Quote));
}
