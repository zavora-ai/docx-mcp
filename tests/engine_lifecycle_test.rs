use docx_mcp_server::engine;
use zavora_docx::Document;

#[test]
fn test_create_kdp_technical_round_trip() {
    let mut doc = Document::new();
    engine::create_kdp_technical(&mut doc);
    let bytes = doc.to_bytes().expect("serialize");
    assert!(!bytes.is_empty());
    Document::from_bytes(&bytes).expect("reopen");
}

/// Headings added via named paragraph styles must be detected by the TOC scanner
/// and produce a real Word TOC field with PAGEREF entries.
#[test]
fn test_toc_from_named_heading_styles() {
    let mut doc = Document::new();
    engine::create_kdp_technical(&mut doc);

    doc.add_paragraph("Chapter 1").style("Heading1");
    doc.add_paragraph("Body text.");
    doc.add_paragraph("Section 1.1").style("Heading2");
    doc.add_paragraph("Chapter 2").style("Heading1");

    // TOC inserted at the top AFTER headings exist must find all three.
    let found = doc.insert_toc(0, 3);
    assert_eq!(found, 3, "TOC should detect 3 heading-styled paragraphs");

    let bytes = doc.to_bytes().expect("serialize");
    let reopened = Document::from_bytes(&bytes).expect("reopen");
    assert!(reopened.content_count() >= 4);
}

/// Calling insert_toc with no heading-styled paragraphs returns 0 and adds nothing.
#[test]
fn test_toc_empty_when_no_headings() {
    let mut doc = Document::new();
    doc.add_paragraph("Just body text, no headings.");
    let before = doc.content_count();
    let found = doc.insert_toc(0, 3);
    assert_eq!(found, 0);
    assert_eq!(doc.content_count(), before, "no TOC paragraphs added");
}
