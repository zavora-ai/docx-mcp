//! MCP integration test: drive the `DocxServer` tool methods directly (the same
//! entry points the JSON-RPC layer dispatches to) and assert their JSON results.
//! This commits coverage of the tool surface that was previously only checked
//! via ad-hoc scripts.

use docx_mcp_server::server::*;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::Value;

fn json(s: &str) -> Value {
    serde_json::from_str(s).expect("tool returned valid JSON")
}

#[tokio::test]
async fn tools_build_a_feature_rich_document() {
    let srv = DocxServer::new();

    // create
    let r = srv
        .create_document(Parameters(CreateInput {
            title: Some("MCP".into()),
            format: None,
            data: None,
        }))
        .await;
    let handle = json(&r)["handle"].as_str().expect("handle").to_string();

    // a paragraph to anchor things on
    let r = srv
        .insert_paragraph(Parameters(InsertParaInput {
            document_handle: handle.clone(),
            index: 0,
            text: "Hello world".into(),
            style: Some("Heading1".into()),
            page_break_before: None,
        }))
        .await;
    assert_eq!(json(&r)["index"], 0);

    // settings (Phase 1)
    let r = srv
        .set_document_settings(Parameters(DocumentSettingsInput {
            document_handle: handle.clone(),
            default_tab_stop_inches: Some(0.5),
            mirror_margins: Some(true),
            track_changes: None,
            zoom_percent: Some(120),
            language: Some("en-US".into()),
            update_fields: Some(true),
            auto_hyphenation: None,
        }))
        .await;
    assert_eq!(json(&r)["updated"], true);

    // content control (Phase 2)
    let r = srv
        .add_content_control(Parameters(ContentControlInput {
            document_handle: handle.clone(),
            kind: "checkbox".into(),
            tag: "agree".into(),
            placeholder: None,
            options: None,
            date_format: None,
            checked: Some(true),
        }))
        .await;
    assert_eq!(json(&r)["added"], true);

    // equation (Phase 3)
    let r = srv
        .add_equation(Parameters(EquationInput {
            document_handle: handle.clone(),
            latex: r"\frac{a}{b}".into(),
        }))
        .await;
    assert_eq!(json(&r)["added"], true);

    // shape (Phase 4)
    let r = srv
        .add_shape(Parameters(ShapeInput {
            document_handle: handle.clone(),
            geometry: "ellipse".into(),
            width_inches: Some(2.0),
            height_inches: Some(2.0),
            fill_color: Some("FFCC00".into()),
        }))
        .await;
    assert_eq!(json(&r)["added"], true);

    // chart incl. scatter (Phase 5 + gap fix)
    let r = srv
        .add_chart(Parameters(ChartInput {
            document_handle: handle.clone(),
            kind: "scatter".into(),
            categories: vec!["1".into(), "2".into()],
            series: vec![ChartSeriesInput {
                name: "s".into(),
                values: vec![3.0, 6.0],
            }],
            title: Some("XY".into()),
            width_inches: None,
            height_inches: None,
            label_position: None,
            label_show_value: None,
            label_show_category: None,
            label_show_percent: None,
            label_color: None,
        }))
        .await;
    assert_eq!(json(&r)["added"], true);

    // threaded comments (Phase 6)
    srv.add_comment(Parameters(CommentInput {
        document_handle: handle.clone(),
        id: 1,
        author: "A".into(),
        text: "q".into(),
    }))
    .await;
    let r = srv
        .reply_to_comment(Parameters(CommentReplyInput {
            document_handle: handle.clone(),
            id: 2,
            parent_id: 1,
            author: "B".into(),
            text: "a".into(),
        }))
        .await;
    assert_eq!(json(&r)["added"], true);

    // save and reopen — proves the assembled document is structurally valid
    let out = std::env::temp_dir().join("mcp_integration_test.docx");
    let out_s = out.to_string_lossy().to_string();
    let r = srv
        .save_document(Parameters(SaveInput {
            document_handle: handle.clone(),
            output_path: out_s.clone(),
        }))
        .await;
    assert!(json(&r)["saved"].is_string(), "save failed: {r}");

    let r = srv
        .open_document(Parameters(OpenInput { file_path: out_s }))
        .await;
    assert!(json(&r)["handle"].is_string(), "reopen failed: {r}");

    let _ = std::fs::remove_file(&out);
}

#[tokio::test]
async fn unknown_chart_kind_is_rejected() {
    let srv = DocxServer::new();
    let h = json(
        &srv.create_document(Parameters(CreateInput {
            title: None,
            format: None,
            data: None,
        }))
        .await,
    )["handle"]
        .as_str()
        .unwrap()
        .to_string();
    let r = srv
        .add_chart(Parameters(ChartInput {
            document_handle: h,
            kind: "bogus".into(),
            categories: vec![],
            series: vec![],
            title: None,
            width_inches: None,
            height_inches: None,
            label_position: None,
            label_show_value: None,
            label_show_category: None,
            label_show_percent: None,
            label_color: None,
        }))
        .await;
    assert!(
        json(&r)["error"].is_string(),
        "expected error for bogus kind: {r}"
    );
}

#[tokio::test]
async fn business_template_fills_supplied_data() {
    let srv = DocxServer::new();
    let data = serde_json::json!({
        "company": "Northwind Studio",
        "items": [
            {"description": "Design", "qty": 1, "price": 4500},
            {"description": "Dev", "qty": 12, "price": 350}
        ]
    });
    let r = srv
        .create_document(Parameters(CreateInput {
            title: None,
            format: Some("business:invoice".into()),
            data: Some(data),
        }))
        .await;
    let handle = json(&r)["handle"].as_str().expect("handle").to_string();
    let r = srv
        .to_plain_text(Parameters(ExportInput {
            document_handle: handle,
        }))
        .await;
    let text = json(&r)["text"].as_str().expect("text").to_string();
    assert!(
        text.contains("Northwind Studio"),
        "supplied company missing: {text}"
    );
    // 1×4500 + 12×350 = 8700 → in-table TOTAL is now reachable via plain text.
    assert!(text.contains("$8,700.00"), "computed total missing: {text}");
}

#[tokio::test]
async fn proposal_template_computes_total() {
    let srv = DocxServer::new();
    let data = serde_json::json!({
        "title": "Redesign",
        "items": [
            {"description": "A", "qty": 1, "price": 6000},
            {"description": "B", "qty": 8, "price": 900}
        ]
    });
    let r = srv
        .create_document(Parameters(CreateInput {
            title: None,
            format: Some("business:proposal".into()),
            data: Some(data),
        }))
        .await;
    let handle = json(&r)["handle"].as_str().expect("handle").to_string();
    let r = srv
        .to_plain_text(Parameters(ExportInput {
            document_handle: handle,
        }))
        .await;
    let text = json(&r)["text"].as_str().expect("text").to_string();
    // 6000 + 8×900 = 13200 → auto-summed deliverables total.
    assert!(
        text.contains("$13,200.00"),
        "proposal total missing: {text}"
    );
}

#[tokio::test]
async fn priced_templates_share_total_logic() {
    let srv = DocxServer::new();
    for (fmt, label) in [
        ("business:quote", "$2,700.00"),
        ("business:receipt", "$2,700.00"),
    ] {
        let data = serde_json::json!({"items": [
            {"description": "Consulting", "qty": 10, "price": 150},
            {"description": "License", "qty": 1, "price": 1200}
        ]});
        let r = srv
            .create_document(Parameters(CreateInput {
                title: None,
                format: Some(fmt.into()),
                data: Some(data),
            }))
            .await;
        let h = json(&r)["handle"].as_str().expect("handle").to_string();
        let text = json(
            &srv.to_plain_text(Parameters(ExportInput { document_handle: h }))
                .await,
        )["text"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(text.contains(label), "{fmt} total missing: {text}");
    }
}

#[tokio::test]
async fn contract_renders_supplied_clauses() {
    let srv = DocxServer::new();
    let data = serde_json::json!({"clauses": [{"title": "Scope", "body": "Deliver a website."}]});
    let r = srv
        .create_document(Parameters(CreateInput {
            title: None,
            format: Some("business:contract".into()),
            data: Some(data),
        }))
        .await;
    let h = json(&r)["handle"].as_str().expect("handle").to_string();
    let text = json(
        &srv.to_plain_text(Parameters(ExportInput { document_handle: h }))
            .await,
    )["text"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        text.contains("Deliver a website."),
        "clause missing: {text}"
    );
}

#[tokio::test]
async fn list_templates_returns_catalog() {
    let srv = DocxServer::new();
    let r = srv.list_templates().await;
    let v = json(&r);
    let templates = v["templates"].as_array().expect("templates array");
    assert_eq!(templates.len(), 21, "expected 21 business templates");
    let params = v["style_params"].as_array().expect("style_params array");
    let names: Vec<&str> = params.iter().map(|p| p["name"].as_str().unwrap()).collect();
    for k in [
        "accent",
        "logo",
        "logo_align",
        "logo_height",
        "heading_font",
        "body_font",
    ] {
        assert!(names.contains(&k), "style_params missing {k}: {names:?}");
    }
    // Every entry must carry a format, description, and data_keys.
    for t in templates {
        assert!(
            t["format"]
                .as_str()
                .is_some_and(|s| s.starts_with("business:")),
            "bad format: {t}"
        );
        assert!(
            !t["description"].as_str().unwrap_or("").is_empty(),
            "empty description: {t}"
        );
        assert!(
            !t["data_keys"].as_str().unwrap_or("").is_empty(),
            "empty data_keys: {t}"
        );
        let df = t["data_fields"].as_array().expect("data_fields array");
        assert!(!df.is_empty(), "empty data_fields: {t}");
        for fld in df {
            assert!(
                fld["name"].as_str().is_some_and(|s| !s.is_empty()),
                "field missing name: {t}"
            );
            assert!(
                matches!(
                    fld["type"].as_str(),
                    Some("text" | "array<string>" | "array<object>")
                ),
                "bad field type: {fld}"
            );
        }
    }
    // The invoice's items field must be a typed object array carrying its item keys.
    let inv = templates
        .iter()
        .find(|t| t["format"] == "business:invoice")
        .expect("invoice");
    let items = inv["data_fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "items")
        .expect("items field");
    assert_eq!(items["type"], "array<object>");
    assert!(
        items["item_keys"]
            .as_array()
            .unwrap()
            .iter()
            .any(|k| k == "description"),
        "items missing keys"
    );
}

#[tokio::test]
async fn tracked_changes_author_list_resolve() {
    let srv = DocxServer::new();
    let h = json(
        &srv.create_document(Parameters(CreateInput {
            title: Some("TC".into()),
            format: None,
            data: None,
        }))
        .await,
    )["handle"]
        .as_str()
        .unwrap()
        .to_string();
    srv.insert_paragraph(Parameters(InsertParaInput {
        document_handle: h.clone(),
        index: 0,
        text: "base".into(),
        style: None,
        page_break_before: None,
    }))
    .await;

    // Author one insertion + one deletion as the agent.
    srv.add_tracked_insert(Parameters(TrackedInsertInput {
        document_handle: h.clone(),
        paragraph_index: 0,
        text: "INS".into(),
        author: "Agent".into(),
    }))
    .await;
    srv.add_tracked_delete(Parameters(TrackedDeleteInput {
        document_handle: h.clone(),
        paragraph_index: 0,
        text: "DEL".into(),
        author: "Agent".into(),
    }))
    .await;

    // List: 2 changes, both by Agent.
    let listed = json(
        &srv.list_tracked_changes(Parameters(HandleInput {
            document_handle: h.clone(),
        }))
        .await,
    );
    assert_eq!(listed["count"], 2);
    assert!(listed["changes"]
        .as_array()
        .unwrap()
        .iter()
        .all(|c| c["author"] == "Agent"));

    // Accept all → resolved 2, none remain.
    let r = json(
        &srv.resolve_tracked_changes(Parameters(ResolveTrackedInput {
            document_handle: h.clone(),
            accept: true,
            change_id: None,
        }))
        .await,
    );
    assert_eq!(r["resolved"], 2);
    assert_eq!(
        json(
            &srv.list_tracked_changes(Parameters(HandleInput { document_handle: h }))
                .await
        )["count"],
        0
    );
}
