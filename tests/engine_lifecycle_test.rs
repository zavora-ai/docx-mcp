#[test]
fn test_create_document_no_title() {
    let doc = docx_mcp_server::engine::create_document(None);
    // A fresh Docx should have at least one body child (default empty paragraph)
    // or zero — either is fine, just verify it doesn't panic.
    let _ = doc.document.children.len();
}

#[test]
fn test_create_document_with_title() {
    let doc = docx_mcp_server::engine::create_document(Some("My Report"));
    let _ = doc.document.children.len();
}

#[test]
fn test_save_and_open_round_trip() {
    let doc = docx_mcp_server::engine::create_document(Some("Round Trip"));
    let bytes = docx_mcp_server::engine::save_document(&doc).unwrap();
    assert!(!bytes.is_empty(), "saved bytes should not be empty");

    let reopened = docx_mcp_server::engine::open_document(&bytes).unwrap();
    assert_eq!(
        doc.document.children.len(),
        reopened.document.children.len(),
        "body children count should survive round-trip"
    );
}

#[test]
fn test_open_document_invalid_bytes() {
    let result = docx_mcp_server::engine::open_document(b"not a docx file");
    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("Engine error"),
        "error should be EngineError variant, got: {msg}"
    );
}
