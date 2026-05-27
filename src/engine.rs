use std::io::Cursor;

use docx_rs::Docx;

use crate::error::DocxMcpError;

// ── Document lifecycle ──────────────────────────────────────────────

/// Create a new empty document, optionally setting the title as a custom property.
///
/// Note: docx-rs v0.4 does not expose a public API for setting the core
/// `dc:title` property, so we store the title as a custom document property.
pub fn create_document(title: Option<&str>) -> Docx {
    use docx_rs::{Style, StyleType, RunFonts};

    let mut doc = Docx::new();
    if let Some(t) = title {
        doc = doc.custom_property("title", t);
    }

    // Add built-in heading styles so Word renders them correctly
    let headings: &[(&str, &str, usize)] = &[
        ("Heading1", "heading 1", 48),
        ("Heading2", "heading 2", 36),
        ("Heading3", "heading 3", 28),
        ("Heading4", "heading 4", 24),
        ("Heading5", "heading 5", 22),
        ("Heading6", "heading 6", 20),
    ];
    for &(id, name, size) in headings {
        let style = Style::new(id, StyleType::Paragraph)
            .name(name)
            .bold()
            .size(size)
            .fonts(RunFonts::new().ascii("Calibri").hi_ansi("Calibri"));
        doc = doc.add_style(style);
    }

    doc
}

/// Parse raw DOCX bytes into a `Docx` instance.
pub fn open_document(bytes: &[u8]) -> Result<Docx, DocxMcpError> {
    docx_rs::read_docx(bytes).map_err(|e| DocxMcpError::EngineError {
        message: format!("Failed to parse DOCX: {e}"),
    })
}

/// Serialize a `Docx` to DOCX (ZIP) bytes.
///
/// `build()` consumes `self`, so we clone the reference first.
pub fn save_document(docx: &Docx) -> Result<Vec<u8>, DocxMcpError> {
    let mut buf = Vec::new();
    let cursor = Cursor::new(&mut buf);
    docx.clone()
        .build()
        .pack(cursor)
        .map_err(|e| DocxMcpError::EngineError {
            message: format!("Failed to pack DOCX: {e}"),
        })?;
    Ok(buf)
}

// ── Read operations ─────────────────────────────────────────────────

use crate::types::enums::SearchMode;
use crate::types::responses::{
    BodyChildInfo, DocumentDescription, NumberingInfo, PaginatedParagraphs, ParagraphDetail,
    ParagraphSummary, RunDetail, SearchResult, TableData,
};
use docx_rs::{DocumentChild, ParagraphChild, RunChild, TableChild, TableCellContent, TableRowChild};
use regex::Regex;

/// Extract the full text from a paragraph by iterating its runs.
fn extract_paragraph_text(para: &docx_rs::Paragraph) -> String {
    let mut text = String::new();
    for child in &para.children {
        if let ParagraphChild::Run(run) = child {
            for rc in &run.children {
                if let RunChild::Text(t) = rc {
                    text.push_str(&t.text);
                }
            }
        }
    }
    text
}

/// Extract the style name from a paragraph's property.
fn extract_style_name(para: &docx_rs::Paragraph) -> Option<String> {
    para.property.style.as_ref().map(|s| s.val.clone())
}

/// Detect heading level from paragraph style name or outline level.
fn detect_heading_level(para: &docx_rs::Paragraph) -> Option<String> {
    // Check outline level first
    if let Some(ref ol) = para.property.outline_lvl {
        if ol.v < 6 {
            return Some(format!("H{}", ol.v + 1));
        }
    }
    // Check style name for heading patterns like "Heading1", "Heading 1", etc.
    if let Some(ref style) = para.property.style {
        let val = &style.val;
        // Common patterns: "Heading1", "Heading 1", "heading 1"
        let lower = val.to_lowercase();
        if lower.starts_with("heading") {
            let num_part = lower.trim_start_matches("heading").trim();
            if let Ok(n) = num_part.parse::<usize>() {
                if (1..=6).contains(&n) {
                    return Some(format!("H{n}"));
                }
            }
        }
    }
    None
}

/// Extract run detail from a single Run.
fn extract_run_detail(run: &docx_rs::Run) -> RunDetail {
    let mut text = String::new();
    for rc in &run.children {
        if let RunChild::Text(t) = rc {
            text.push_str(&t.text);
        }
    }

    let rp = &run.run_property;

    // Bold: check if the Option is Some (Bold defaults to val=true when created)
    let bold = rp.bold.is_some();
    let italic = rp.italic.is_some();
    let underline = rp.underline.is_some();

    // For font, size, color — the inner fields are private, so we serialize to extract values
    let font = rp.fonts.as_ref().and_then(|f| {
        // RunFonts serializes to JSON with camelCase fields
        serde_json::to_value(f).ok().and_then(|v| {
            // Try ascii first, then hiAnsi, then eastAsia
            v.get("ascii")
                .and_then(|a| a.as_str().map(String::from))
                .or_else(|| v.get("hiAnsi").and_then(|a| a.as_str().map(String::from)))
                .or_else(|| v.get("eastAsia").and_then(|a| a.as_str().map(String::from)))
        })
    });

    let size = rp.sz.as_ref().and_then(|sz| {
        // Sz serializes as a u32 value directly
        serde_json::to_value(sz)
            .ok()
            .and_then(|v| v.as_u64().map(|n| n as usize))
    });

    let color = rp.color.as_ref().and_then(|c| {
        // Color serializes as a string directly
        serde_json::to_value(c)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
    });

    RunDetail {
        text,
        bold,
        italic,
        underline,
        font,
        size,
        color,
    }
}

/// Describe the document structure: count paragraphs, tables, body children info, headers/footers.
pub fn describe_document(docx: &Docx) -> DocumentDescription {
    let mut paragraph_count = 0usize;
    let mut table_count = 0usize;
    let mut body_children = Vec::new();

    for (i, child) in docx.document.children.iter().enumerate() {
        match child {
            DocumentChild::Paragraph(para) => {
                paragraph_count += 1;
                body_children.push(BodyChildInfo {
                    index: i,
                    child_type: "paragraph".to_string(),
                    heading_level: detect_heading_level(para),
                    style_name: extract_style_name(para),
                });
            }
            DocumentChild::Table(_) => {
                table_count += 1;
                body_children.push(BodyChildInfo {
                    index: i,
                    child_type: "table".to_string(),
                    heading_level: None,
                    style_name: None,
                });
            }
            _ => {
                // Skip bookmarks, comments, structured data tags, etc.
            }
        }
    }

    // Section count: at minimum 1 (the document's section_property), plus any
    // section properties embedded in paragraph properties
    let mut section_count = 1usize;
    for child in &docx.document.children {
        if let DocumentChild::Paragraph(para) = child {
            if para.property.section_property.is_some() {
                section_count += 1;
            }
        }
    }

    // Detect headers/footers from document_rels
    let has_headers = docx.document_rels.header_count > 0;
    let has_footers = docx.document_rels.footer_count > 0;

    DocumentDescription {
        paragraph_count,
        table_count,
        body_children,
        section_count,
        has_headers,
        has_footers,
    }
}

/// Read paragraphs with pagination. Skips tables, returns only paragraphs with their original BodyIndex.
pub fn read_paragraphs(docx: &Docx, offset: usize, limit: usize) -> PaginatedParagraphs {
    // Collect all paragraphs with their original body index
    let all_paragraphs: Vec<(usize, &docx_rs::Paragraph)> = docx
        .document
        .children
        .iter()
        .enumerate()
        .filter_map(|(i, child)| {
            if let DocumentChild::Paragraph(para) = child {
                Some((i, para.as_ref()))
            } else {
                None
            }
        })
        .collect();

    let total_count = all_paragraphs.len();
    let page = all_paragraphs
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|(idx, para)| ParagraphSummary {
            index: idx,
            text: extract_paragraph_text(para),
            style: extract_style_name(para),
            heading_level: detect_heading_level(para),
        })
        .collect::<Vec<_>>();

    let returned = page.len();

    PaginatedParagraphs {
        paragraphs: page,
        total_count,
        offset,
        returned,
    }
}

/// Read a single paragraph with full detail. Returns error if the index points to a table.
pub fn read_paragraph(docx: &Docx, index: usize) -> Result<ParagraphDetail, DocxMcpError> {
    let child = crate::doc_ref::resolve_body_child(docx, index)?;

    let para = match child {
        DocumentChild::Paragraph(p) => p,
        DocumentChild::Table(_) => {
            return Err(DocxMcpError::InvalidInput {
                message: format!(
                    "Body child at index {} is a table, not a paragraph",
                    index
                ),
            });
        }
        _ => {
            return Err(DocxMcpError::InvalidInput {
                message: format!(
                    "Body child at index {} is not a paragraph",
                    index
                ),
            });
        }
    };

    let text = extract_paragraph_text(para);

    let runs: Vec<RunDetail> = para
        .children
        .iter()
        .filter_map(|child| {
            if let ParagraphChild::Run(run) = child {
                Some(extract_run_detail(run))
            } else {
                None
            }
        })
        .collect();

    let style = extract_style_name(para);
    let heading_level = detect_heading_level(para);

    let numbering = para.property.numbering_property.as_ref().and_then(|np| {
        let num_id = np.id.as_ref().map(|nid| nid.id)?;
        let level = np.level.as_ref().map(|l| l.val).unwrap_or(0);
        Some(NumberingInfo {
            level,
            num_id,
        })
    });

    let alignment = para.property.alignment.as_ref().map(|j| j.val.clone());

    Ok(ParagraphDetail {
        index,
        text,
        runs,
        style,
        heading_level,
        numbering,
        alignment,
    })
}

/// Read a table's content as structured rows/cells. Returns error if the index points to a paragraph.
pub fn read_table(docx: &Docx, table_index: usize) -> Result<TableData, DocxMcpError> {
    let child = crate::doc_ref::resolve_body_child(docx, table_index)?;

    let table = match child {
        DocumentChild::Table(t) => t,
        DocumentChild::Paragraph(_) => {
            return Err(DocxMcpError::InvalidInput {
                message: format!(
                    "Body child at index {} is a paragraph, not a table",
                    table_index
                ),
            });
        }
        _ => {
            return Err(DocxMcpError::InvalidInput {
                message: format!(
                    "Body child at index {} is not a table",
                    table_index
                ),
            });
        }
    };

    let mut rows_data: Vec<Vec<String>> = Vec::new();
    let mut max_cols = 0usize;

    for table_child in &table.rows {
        let TableChild::TableRow(row) = table_child;
        let mut row_cells: Vec<String> = Vec::new();

        for cell_child in &row.cells {
            let TableRowChild::TableCell(cell) = cell_child;
            // Extract text from all paragraphs in the cell
            let mut cell_text = String::new();
            for content in &cell.children {
                if let TableCellContent::Paragraph(para) = content {
                    if !cell_text.is_empty() {
                        cell_text.push('\n');
                    }
                    cell_text.push_str(&extract_paragraph_text(para));
                }
            }
            row_cells.push(cell_text);
        }

        if row_cells.len() > max_cols {
            max_cols = row_cells.len();
        }
        rows_data.push(row_cells);
    }

    let row_count = rows_data.len();

    Ok(TableData {
        rows: rows_data,
        row_count,
        column_count: max_cols,
    })
}

/// Search text across all paragraphs. Supports exact, substring, and regex modes.
pub fn search_text(
    docx: &Docx,
    query: &str,
    mode: SearchMode,
) -> Result<Vec<SearchResult>, DocxMcpError> {
    // Pre-compile regex if needed
    let compiled_regex = if matches!(mode, SearchMode::Regex) {
        Some(Regex::new(query).map_err(|e| DocxMcpError::InvalidInput {
            message: format!("Invalid regex pattern '{}': {}", query, e),
        })?)
    } else {
        None
    };

    let mut results = Vec::new();

    for (i, child) in docx.document.children.iter().enumerate() {
        if let DocumentChild::Paragraph(para) = child {
            let text = extract_paragraph_text(para);

            let matched = match &mode {
                SearchMode::Exact => text == query,
                SearchMode::Substring => text.contains(query),
                SearchMode::Regex => compiled_regex.as_ref().unwrap().is_match(&text),
            };

            if matched {
                let matched_text = match &mode {
                    SearchMode::Exact => text.clone(),
                    SearchMode::Substring => query.to_string(),
                    SearchMode::Regex => {
                        // Return the actual matched portion
                        compiled_regex
                            .as_ref()
                            .unwrap()
                            .find(&text)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_else(|| text.clone())
                    }
                };

                results.push(SearchResult {
                    index: i,
                    matched_text,
                    paragraph_text: text,
                });
            }
        }
    }

    Ok(results)
}

// ── Write operations ────────────────────────────────────────────────

use crate::doc_ref::count_body_children;
use crate::types::enums::HeadingLevel;
use docx_rs::{Paragraph, Run, RunFonts};

/// Formatting options for a new run.
pub struct RunFormat {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub font: Option<String>,
    pub size: Option<usize>,
    pub color: Option<String>,
}

/// Map a HeadingLevel enum to the corresponding Word style name.
fn heading_level_to_style(level: &HeadingLevel) -> &'static str {
    match level {
        HeadingLevel::H1 => "Heading1",
        HeadingLevel::H2 => "Heading2",
        HeadingLevel::H3 => "Heading3",
        HeadingLevel::H4 => "Heading4",
        HeadingLevel::H5 => "Heading5",
        HeadingLevel::H6 => "Heading6",
    }
}

/// Apply RunFormat options to a Run, returning the modified Run.
fn apply_run_format(mut run: Run, format: &RunFormat) -> Run {
    if let Some(true) = format.bold {
        run = run.bold();
    }
    if let Some(true) = format.italic {
        run = run.italic();
    }
    if let Some(true) = format.underline {
        run = run.underline("single");
    }
    if let Some(ref font) = format.font {
        run = run.fonts(RunFonts::new().ascii(font).hi_ansi(font));
    }
    if let Some(size) = format.size {
        run = run.size(size);
    }
    if let Some(ref color) = format.color {
        run = run.color(color);
    }
    run
}

/// Insert a new paragraph at the given body index (or append if index == count).
/// Returns the index where the paragraph was inserted.
pub fn insert_paragraph(
    docx: &mut Docx,
    index: usize,
    text: Option<&str>,
    heading: Option<HeadingLevel>,
    style: Option<&str>,
    page_break_before: bool,
) -> Result<usize, DocxMcpError> {
    let count = count_body_children(docx);
    if index > count {
        return Err(DocxMcpError::IndexOutOfBounds {
            message: "Insert paragraph index out of bounds".into(),
            index,
            max: count,
        });
    }

    let mut para = Paragraph::new();

    // Determine run formatting based on style
    let effective_style = style.or_else(|| heading.as_ref().map(|h| heading_level_to_style(h)));
    let mut run = Run::new();
    if let Some(t) = text {
        run = run.add_text(t);
    }

    // Apply run-level font/size for known styles (ensures Word renders correctly)
    match effective_style {
        Some("Heading1") => { run = run.bold().size(48).fonts(RunFonts::new().ascii("Garamond").hi_ansi("Garamond")); }
        Some("Heading2") => { run = run.bold().size(28).fonts(RunFonts::new().ascii("Garamond").hi_ansi("Garamond")); }
        Some("Heading3") => { run = run.bold().size(24).fonts(RunFonts::new().ascii("Garamond").hi_ansi("Garamond")); }
        Some("TitlePage") => { run = run.bold().size(56).fonts(RunFonts::new().ascii("Garamond").hi_ansi("Garamond")); }
        Some("Subtitle") => { run = run.italic().size(28).fonts(RunFonts::new().ascii("Garamond").hi_ansi("Garamond")); }
        Some("Author") => { run = run.size(28).fonts(RunFonts::new().ascii("Garamond").hi_ansi("Garamond")); }
        Some("Copyright") => { run = run.size(18).fonts(RunFonts::new().ascii("Garamond").hi_ansi("Garamond")); }
        Some("ChapterNum") => { run = run.size(24).fonts(RunFonts::new().ascii("Garamond").hi_ansi("Garamond")); }
        Some("BodyText") | Some("BodyTextIndent") => { run = run.size(22).fonts(RunFonts::new().ascii("Garamond").hi_ansi("Garamond")); }
        _ => {}
    }

    if text.is_some() {
        para = para.add_run(run);
    }

    // Apply heading style
    if let Some(ref h) = heading {
        para = para.style(heading_level_to_style(h));
    }

    // Apply named style
    if let Some(s) = style {
        para = para.style(s);
    }

    // Apply paragraph-level formatting for known styles
    match effective_style {
        Some("Heading1") | Some("TitlePage") | Some("Subtitle") | Some("Author") | Some("ChapterNum") => {
            para = para.align(AlignmentType::Center);
        }
        Some("BodyTextIndent") => {
            para = para.indent(None, Some(SpecialIndentType::FirstLine(432)), None, None);
        }
        _ => {}
    }

    if page_break_before {
        para = para.page_break_before(true);
    }

    docx.document
        .children
        .insert(index, DocumentChild::Paragraph(Box::new(para)));

    Ok(index)
}

/// Replace occurrences of `search` with `replacement` in all paragraph runs.
/// Returns the total number of replacements made.
pub fn replace_text(
    docx: &mut Docx,
    search: &str,
    replacement: &str,
    first_only: bool,
) -> usize {
    let mut total_replacements = 0usize;

    for child in &mut docx.document.children {
        if let DocumentChild::Paragraph(para) = child {
            for para_child in &mut para.children {
                if let ParagraphChild::Run(run) = para_child {
                    for rc in &mut run.children {
                        if let RunChild::Text(t) = rc {
                            let count = t.text.matches(search).count();
                            if count > 0 {
                                if first_only && total_replacements == 0 {
                                    // Replace only the first occurrence
                                    t.text = t.text.replacen(search, replacement, 1);
                                    total_replacements += 1;
                                    return total_replacements;
                                } else if first_only && total_replacements > 0 {
                                    // Already replaced one, stop
                                    return total_replacements;
                                } else {
                                    t.text = t.text.replace(search, replacement);
                                    total_replacements += count;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    total_replacements
}

/// Delete a body child at the given index, or a specific run within a paragraph.
/// Returns the updated body children count.
pub fn delete_content(
    docx: &mut Docx,
    index: usize,
    run_index: Option<usize>,
) -> Result<usize, DocxMcpError> {
    let count = count_body_children(docx);
    if index >= count {
        return Err(DocxMcpError::IndexOutOfBounds {
            message: "Delete content index out of bounds".into(),
            index,
            max: count,
        });
    }

    match run_index {
        Some(ri) => {
            // Delete a specific run from the paragraph
            let child = &mut docx.document.children[index];
            let para = match child {
                DocumentChild::Paragraph(p) => p,
                DocumentChild::Table(_) => {
                    return Err(DocxMcpError::InvalidInput {
                        message: format!(
                            "Body child at index {} is a table, not a paragraph",
                            index
                        ),
                    });
                }
                _ => {
                    return Err(DocxMcpError::InvalidInput {
                        message: format!(
                            "Body child at index {} is not a paragraph",
                            index
                        ),
                    });
                }
            };

            // Count runs to validate run_index
            let run_positions: Vec<usize> = para
                .children
                .iter()
                .enumerate()
                .filter_map(|(i, c)| {
                    if matches!(c, ParagraphChild::Run(_)) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();

            let run_count = run_positions.len();
            if ri >= run_count {
                return Err(DocxMcpError::IndexOutOfBounds {
                    message: format!(
                        "Run index out of bounds in paragraph at body index {}",
                        index
                    ),
                    index: ri,
                    max: run_count,
                });
            }

            // Remove the run at the actual position in children vec
            let actual_pos = run_positions[ri];
            para.children.remove(actual_pos);

            // Paragraph is retained even if empty (no runs left)
            Ok(count_body_children(docx))
        }
        None => {
            // Remove the entire body child
            docx.document.children.remove(index);
            Ok(count_body_children(docx))
        }
    }
}

/// Add a formatted run to the paragraph at the given body index.
/// Returns an error if the index points to a table.
pub fn insert_run(
    docx: &mut Docx,
    index: usize,
    text: &str,
    format: Option<RunFormat>,
) -> Result<(), DocxMcpError> {
    let child = crate::doc_ref::resolve_body_child_mut(docx, index)?;

    let para = match child {
        DocumentChild::Paragraph(p) => p,
        DocumentChild::Table(_) => {
            return Err(DocxMcpError::InvalidInput {
                message: format!(
                    "Body child at index {} is a table, not a paragraph",
                    index
                ),
            });
        }
        _ => {
            return Err(DocxMcpError::InvalidInput {
                message: format!(
                    "Body child at index {} is not a paragraph",
                    index
                ),
            });
        }
    };

    let mut run = Run::new().add_text(text);

    if let Some(fmt) = format {
        run = apply_run_format(run, &fmt);
    }

    para.children.push(ParagraphChild::Run(Box::new(run)));

    Ok(())
}

/// Clear all runs in a paragraph and replace with a single run containing the given text.
/// Preserves paragraph-level style. Returns an error if the index points to a table.
pub fn update_paragraph_text(
    docx: &mut Docx,
    index: usize,
    text: &str,
) -> Result<(), DocxMcpError> {
    let child = crate::doc_ref::resolve_body_child_mut(docx, index)?;

    let para = match child {
        DocumentChild::Paragraph(p) => p,
        DocumentChild::Table(_) => {
            return Err(DocxMcpError::InvalidInput {
                message: format!(
                    "Body child at index {} is a table, not a paragraph",
                    index
                ),
            });
        }
        _ => {
            return Err(DocxMcpError::InvalidInput {
                message: format!(
                    "Body child at index {} is not a paragraph",
                    index
                ),
            });
        }
    };

    // Remove all Run children, preserving non-run children (bookmarks, etc.)
    para.children.retain(|c| !matches!(c, ParagraphChild::Run(_)));

    // Add a single new run with the text
    let run = Run::new().add_text(text);
    para.children.push(ParagraphChild::Run(Box::new(run)));

    Ok(())
}

// ── Batch operations ────────────────────────────────────────────────

use crate::types::enums::BatchOperationType;
use crate::types::inputs::{
    BatchOperation, DeleteContentInput, InsertParagraphInput, InsertRunInput, ReplaceTextInput,
    UpdateParagraphTextInput,
};

/// Execute a sequence of write operations against the document.
/// Stops on the first error, returning the operation index and reason.
/// Returns the count of successfully completed operations.
pub fn batch_write(
    docx: &mut Docx,
    operations: &[BatchOperation],
) -> Result<usize, DocxMcpError> {
    let mut completed = 0usize;

    for (i, op) in operations.iter().enumerate() {
        let result: Result<(), DocxMcpError> = match op.operation_type {
            BatchOperationType::InsertParagraph => {
                let input: InsertParagraphInput =
                    serde_json::from_value(op.params.clone()).map_err(|e| {
                        DocxMcpError::InvalidInput {
                            message: format!("Operation {i} deserialization failed: {e}"),
                        }
                    })?;
                insert_paragraph(
                    docx,
                    input.index,
                    input.text.as_deref(),
                    input.heading_level,
                    input.style.as_deref(),
                    input.page_break_before.unwrap_or(false),
                )
                .map(|_| ())
            }
            BatchOperationType::ReplaceText => {
                let input: ReplaceTextInput =
                    serde_json::from_value(op.params.clone()).map_err(|e| {
                        DocxMcpError::InvalidInput {
                            message: format!("Operation {i} deserialization failed: {e}"),
                        }
                    })?;
                replace_text(
                    docx,
                    &input.search,
                    &input.replacement,
                    input.replace_first.unwrap_or(false),
                );
                Ok(())
            }
            BatchOperationType::DeleteContent => {
                let input: DeleteContentInput =
                    serde_json::from_value(op.params.clone()).map_err(|e| {
                        DocxMcpError::InvalidInput {
                            message: format!("Operation {i} deserialization failed: {e}"),
                        }
                    })?;
                delete_content(docx, input.index, input.run_index).map(|_| ())
            }
            BatchOperationType::InsertRun => {
                let input: InsertRunInput =
                    serde_json::from_value(op.params.clone()).map_err(|e| {
                        DocxMcpError::InvalidInput {
                            message: format!("Operation {i} deserialization failed: {e}"),
                        }
                    })?;
                let fmt = RunFormat {
                    bold: input.bold,
                    italic: input.italic,
                    underline: input.underline,
                    font: input.font,
                    size: input.size,
                    color: input.color,
                };
                insert_run(docx, input.paragraph_index, &input.text, Some(fmt))
            }
            BatchOperationType::UpdateParagraphText => {
                let input: UpdateParagraphTextInput =
                    serde_json::from_value(op.params.clone()).map_err(|e| {
                        DocxMcpError::InvalidInput {
                            message: format!("Operation {i} deserialization failed: {e}"),
                        }
                    })?;
                update_paragraph_text(docx, input.paragraph_index, &input.text)
            }
        };

        result.map_err(|e| DocxMcpError::EngineError {
            message: format!("Operation {i} failed: {e}"),
        })?;

        completed += 1;
    }

    Ok(completed)
}

// ── Formatting operations ───────────────────────────────────────────

use crate::types::enums::Alignment;
use docx_rs::{
    Bold, BoldCs, Color, Indent, Italic, ItalicCs, Justification, LineSpacing, OutlineLvl,
    ParagraphStyle, RunStyle, Sz, SzCs, Underline,
};

/// Paragraph-level formatting options. Only `Some` fields are applied.
pub struct ParagraphFormat {
    pub alignment: Option<Alignment>,
    pub line_spacing: Option<f32>,
    pub space_before: Option<u32>,
    pub space_after: Option<u32>,
    pub indent_left: Option<u32>,
    pub indent_right: Option<u32>,
    pub indent_first_line: Option<u32>,
    pub heading_level: Option<HeadingLevel>,
}

/// Map our Alignment enum to the docx-rs AlignmentType string used by Justification.
fn alignment_to_string(a: &Alignment) -> &'static str {
    match a {
        Alignment::Left => "left",
        Alignment::Center => "center",
        Alignment::Right => "right",
        Alignment::Justify => "both",
    }
}

/// Apply formatting properties directly to an existing run's `run_property` by mutation.
fn apply_format_to_run_property(rp: &mut docx_rs::RunProperty, format: &RunFormat) {
    if let Some(bold) = format.bold {
        if bold {
            rp.bold = Some(Bold::new());
            rp.bold_cs = Some(BoldCs::new());
        } else {
            rp.bold = Some(Bold::new().disable());
            rp.bold_cs = Some(BoldCs::new().disable());
        }
    }
    if let Some(italic) = format.italic {
        if italic {
            rp.italic = Some(Italic::new());
            rp.italic_cs = Some(ItalicCs::new());
        } else {
            rp.italic = Some(Italic::new().disable());
            rp.italic_cs = Some(ItalicCs::new().disable());
        }
    }
    if let Some(underline) = format.underline {
        if underline {
            rp.underline = Some(Underline::new("single"));
        } else {
            rp.underline = None;
        }
    }
    if let Some(ref font) = format.font {
        rp.fonts = Some(RunFonts::new().ascii(font).hi_ansi(font));
    }
    if let Some(size) = format.size {
        rp.sz = Some(Sz::new(size));
        rp.sz_cs = Some(SzCs::new(size));
    }
    if let Some(ref color) = format.color {
        rp.color = Some(Color::new(color));
    }
}

/// Apply bold/italic/underline/font/size/color to a run or range of runs.
/// If `run_end` is provided, applies to all runs from `run_index` to `run_end` inclusive.
pub fn set_run_format(
    docx: &mut Docx,
    index: usize,
    run_index: usize,
    run_end: Option<usize>,
    format: RunFormat,
) -> Result<(), DocxMcpError> {
    let child = crate::doc_ref::resolve_body_child_mut(docx, index)?;

    let para = match child {
        DocumentChild::Paragraph(p) => p,
        DocumentChild::Table(_) => {
            return Err(DocxMcpError::InvalidInput {
                message: format!("Body child at index {} is a table, not a paragraph", index),
            });
        }
        _ => {
            return Err(DocxMcpError::InvalidInput {
                message: format!("Body child at index {} is not a paragraph", index),
            });
        }
    };

    // Collect indices of Run children within the paragraph's children vec
    let run_positions: Vec<usize> = para
        .children
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            if matches!(c, ParagraphChild::Run(_)) {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    let run_count = run_positions.len();
    let end = run_end.unwrap_or(run_index);

    if run_index >= run_count {
        return Err(DocxMcpError::IndexOutOfBounds {
            message: format!("Run index out of bounds in paragraph at body index {}", index),
            index: run_index,
            max: run_count,
        });
    }
    if end >= run_count {
        return Err(DocxMcpError::IndexOutOfBounds {
            message: format!("Run end index out of bounds in paragraph at body index {}", index),
            index: end,
            max: run_count,
        });
    }

    // Apply format to each run in the range
    for ri in run_index..=end {
        let actual_pos = run_positions[ri];
        if let ParagraphChild::Run(run) = &mut para.children[actual_pos] {
            apply_format_to_run_property(&mut run.run_property, &format);
        }
    }

    Ok(())
}

/// Apply paragraph-level formatting. Only provided (Some) fields are changed.
/// Returns InvalidInput if the body child is a table.
pub fn set_paragraph_format(
    docx: &mut Docx,
    index: usize,
    format: ParagraphFormat,
) -> Result<(), DocxMcpError> {
    let child = crate::doc_ref::resolve_body_child_mut(docx, index)?;

    let para = match child {
        DocumentChild::Paragraph(p) => p,
        DocumentChild::Table(_) => {
            return Err(DocxMcpError::InvalidInput {
                message: format!("Body child at index {} is a table, not a paragraph", index),
            });
        }
        _ => {
            return Err(DocxMcpError::InvalidInput {
                message: format!("Body child at index {} is not a paragraph", index),
            });
        }
    };

    // Alignment
    if let Some(ref alignment) = format.alignment {
        para.property.alignment = Some(Justification::new(alignment_to_string(alignment)));
    }

    // Line spacing — we use the line value in twips (1/20 of a point).
    // The input is a float multiplier (e.g. 1.5 = 1.5x line spacing).
    // Word uses 240 twips = single spacing, so multiply by 240.
    if let Some(spacing_val) = format.line_spacing {
        let line_twips = (spacing_val * 240.0) as i32;
        let mut ls = LineSpacing::new().line(line_twips);
        // Also preserve existing before/after if set via separate fields
        ls = ls.line_rule(docx_rs::LineSpacingType::Auto);
        para.property.line_spacing = Some(ls);
    }

    // Space before / after — set via LineSpacing's before/after fields
    // These are in twips (twentieths of a point)
    if format.space_before.is_some() || format.space_after.is_some() {
        let mut ls = para.property.line_spacing.take().unwrap_or_else(LineSpacing::new);
        if let Some(before) = format.space_before {
            ls = ls.before(before);
        }
        if let Some(after) = format.space_after {
            ls = ls.after(after);
        }
        para.property.line_spacing = Some(ls);
    }

    // Indentation
    if format.indent_left.is_some() || format.indent_right.is_some() || format.indent_first_line.is_some() {
        let left = format.indent_left.map(|v| v as i32);
        let end = format.indent_right.map(|v| v as i32);
        let special = format.indent_first_line.map(|v| {
            docx_rs::SpecialIndentType::FirstLine(v as i32)
        });
        let mut indent = Indent::new(left, special, end, None);
        // If only some fields provided, try to preserve existing values
        if let Some(ref existing) = para.property.indent {
            if left.is_none() {
                indent.start = existing.start;
            }
            if end.is_none() {
                indent.end = existing.end;
            }
            if special.is_none() {
                indent.special_indent = existing.special_indent;
            }
        }
        para.property.indent = Some(indent);
    }

    // Heading level — set via paragraph style
    if let Some(ref heading) = format.heading_level {
        let style_name = heading_level_to_style(heading);
        para.property.style = Some(ParagraphStyle::new(Some(style_name)));
        // Also set outline level for proper heading semantics
        let level: usize = match heading {
            HeadingLevel::H1 => 0,
            HeadingLevel::H2 => 1,
            HeadingLevel::H3 => 2,
            HeadingLevel::H4 => 3,
            HeadingLevel::H5 => 4,
            HeadingLevel::H6 => 5,
        };
        para.property.outline_lvl = Some(OutlineLvl::new(level));
    }

    Ok(())
}

/// Apply a named style to a paragraph or a specific run within it.
/// If `run_index` is None, applies to the paragraph. If Some, applies to that run.
pub fn apply_style(
    docx: &mut Docx,
    index: usize,
    run_index: Option<usize>,
    style_name: &str,
) -> Result<(), DocxMcpError> {
    let child = crate::doc_ref::resolve_body_child_mut(docx, index)?;

    let para = match child {
        DocumentChild::Paragraph(p) => p,
        DocumentChild::Table(_) => {
            return Err(DocxMcpError::InvalidInput {
                message: format!("Body child at index {} is a table, not a paragraph", index),
            });
        }
        _ => {
            return Err(DocxMcpError::InvalidInput {
                message: format!("Body child at index {} is not a paragraph", index),
            });
        }
    };

    match run_index {
        None => {
            // Apply style to the paragraph
            para.property.style = Some(ParagraphStyle::new(Some(style_name)));
        }
        Some(ri) => {
            // Find the run and apply style to it
            let run_positions: Vec<usize> = para
                .children
                .iter()
                .enumerate()
                .filter_map(|(i, c)| {
                    if matches!(c, ParagraphChild::Run(_)) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();

            let run_count = run_positions.len();
            if ri >= run_count {
                return Err(DocxMcpError::IndexOutOfBounds {
                    message: format!(
                        "Run index out of bounds in paragraph at body index {}",
                        index
                    ),
                    index: ri,
                    max: run_count,
                });
            }

            let actual_pos = run_positions[ri];
            if let ParagraphChild::Run(run) = &mut para.children[actual_pos] {
                run.run_property.style = Some(RunStyle::new(style_name));
            }
        }
    }

    Ok(())
}

// ── Table operations ────────────────────────────────────────────────

use crate::doc_ref::TableCellRef;
use crate::types::enums::MergeDirection;
use docx_rs::{Table, TableCell, TableRow, VMergeType};

/// Insert a table with the given dimensions at `position` (or append).
/// Returns `(body_index, rows, cols)`.
pub fn add_table(
    docx: &mut Docx,
    rows: usize,
    cols: usize,
    position: Option<usize>,
) -> Result<(usize, usize, usize), DocxMcpError> {
    let count = count_body_children(docx);
    let pos = position.unwrap_or(count);

    if pos > count {
        return Err(DocxMcpError::IndexOutOfBounds {
            message: "Table insert position out of bounds".into(),
            index: pos,
            max: count,
        });
    }

    // Build rows × cols of empty cells, each containing an empty paragraph
    let table_rows: Vec<TableRow> = (0..rows)
        .map(|_| {
            let cells: Vec<TableCell> = (0..cols)
                .map(|_| TableCell::new().add_paragraph(Paragraph::new()))
                .collect();
            TableRow::new(cells)
        })
        .collect();

    let table = Table::new(table_rows);
    docx.document
        .children
        .insert(pos, DocumentChild::Table(Box::new(table)));

    Ok((pos, rows, cols))
}

/// Clear a table cell and set its content to a single paragraph with the given text.
/// Returns `IndexOutOfBounds` if the cell reference is invalid.
pub fn set_table_cell(
    docx: &mut Docx,
    cell_ref: &TableCellRef,
    text: &str,
) -> Result<(), DocxMcpError> {
    let cell = crate::doc_ref::resolve_table_cell_mut(docx, cell_ref)?;

    // Clear existing children and add a single paragraph with the text
    cell.children.clear();
    cell.children.push(TableCellContent::Paragraph(
        Paragraph::new().add_run(Run::new().add_text(text)),
    ));

    Ok(())
}

/// Append a new row to the table at `table_index`, matching the existing column count.
/// If `cell_texts` is provided, populate cells with those texts (extras ignored, missing filled empty).
/// Returns the new row's index within the table.
pub fn add_table_row(
    docx: &mut Docx,
    table_index: usize,
    cell_texts: Option<Vec<String>>,
) -> Result<usize, DocxMcpError> {
    let child = crate::doc_ref::resolve_body_child_mut(docx, table_index)?;

    let table = match child {
        DocumentChild::Table(t) => t,
        _ => {
            return Err(DocxMcpError::InvalidInput {
                message: format!("Body child at index {} is not a table", table_index),
            });
        }
    };

    // Determine column count from existing rows
    let col_count = table
        .rows
        .first()
        .map(|tc| {
            let TableChild::TableRow(row) = tc;
            row.cells.len()
        })
        .unwrap_or(1);

    let texts = cell_texts.unwrap_or_default();

    let cells: Vec<TableCell> = (0..col_count)
        .map(|i| {
            let cell_text = texts.get(i).map(|s| s.as_str()).unwrap_or("");
            if cell_text.is_empty() {
                TableCell::new().add_paragraph(Paragraph::new())
            } else {
                TableCell::new().add_paragraph(
                    Paragraph::new().add_run(Run::new().add_text(cell_text)),
                )
            }
        })
        .collect();

    let new_row = TableRow::new(cells);
    let row_index = table.rows.len();
    table.rows.push(TableChild::TableRow(new_row));

    Ok(row_index)
}

/// Merge table cells either horizontally or vertically.
///
/// For **horizontal** merge: merges cells in a single row from `start.1` to `end.1`
/// by setting `grid_span` on the first cell and removing the spanned cells.
/// `start` and `end` must be in the same row (`start.0 == end.0`).
///
/// For **vertical** merge: merges cells in a single column from row `start.0` to `end.0`
/// by setting `VMerge::Restart` on the first cell and `VMerge::Continue` on subsequent cells.
/// `start` and `end` must be in the same column (`start.1 == end.1`).
pub fn merge_table_cells(
    docx: &mut Docx,
    table_index: usize,
    start: (usize, usize),
    end: (usize, usize),
    direction: MergeDirection,
) -> Result<(), DocxMcpError> {
    let child = crate::doc_ref::resolve_body_child_mut(docx, table_index)?;

    let table = match child {
        DocumentChild::Table(t) => t,
        _ => {
            return Err(DocxMcpError::InvalidInput {
                message: format!("Body child at index {} is not a table", table_index),
            });
        }
    };

    let row_count = table.rows.len();

    match direction {
        MergeDirection::Horizontal => {
            // Horizontal merge: same row, span across columns
            let row_idx = start.0;
            let col_start = start.1;
            let col_end = end.1;

            if row_idx >= row_count {
                return Err(DocxMcpError::IndexOutOfBounds {
                    message: "Merge row index out of bounds".into(),
                    index: row_idx,
                    max: row_count,
                });
            }

            let TableChild::TableRow(row) = &mut table.rows[row_idx];
            let cell_count = row.cells.len();

            if col_start >= cell_count || col_end >= cell_count {
                return Err(DocxMcpError::IndexOutOfBounds {
                    message: "Merge column index out of bounds".into(),
                    index: col_end,
                    max: cell_count,
                });
            }

            if col_start > col_end {
                return Err(DocxMcpError::InvalidInput {
                    message: "Start column must be <= end column for horizontal merge".into(),
                });
            }

            let span = col_end - col_start + 1;

            // Set grid_span on the first cell by replacing its property
            let TableRowChild::TableCell(first_cell) = &mut row.cells[col_start];
            first_cell.property = first_cell.property.clone().grid_span(span);

            // Remove the spanned cells (from end to start+1 to preserve indices)
            for i in (col_start + 1..=col_end).rev() {
                row.cells.remove(i);
            }
        }
        MergeDirection::Vertical => {
            // Vertical merge: same column, span across rows
            let col_idx = start.1;
            let row_start = start.0;
            let row_end = end.0;

            if row_start >= row_count || row_end >= row_count {
                return Err(DocxMcpError::IndexOutOfBounds {
                    message: "Merge row index out of bounds".into(),
                    index: row_end,
                    max: row_count,
                });
            }

            if row_start > row_end {
                return Err(DocxMcpError::InvalidInput {
                    message: "Start row must be <= end row for vertical merge".into(),
                });
            }

            // Validate column index in all affected rows
            for r in row_start..=row_end {
                let TableChild::TableRow(row) = &table.rows[r];
                if col_idx >= row.cells.len() {
                    return Err(DocxMcpError::IndexOutOfBounds {
                        message: format!("Column index out of bounds in row {}", r),
                        index: col_idx,
                        max: row.cells.len(),
                    });
                }
            }

            // Set VMerge::Restart on the first cell
            let TableChild::TableRow(first_row) = &mut table.rows[row_start];
            let TableRowChild::TableCell(first_cell) = &mut first_row.cells[col_idx];
            first_cell.property = first_cell.property.clone().vertical_merge(VMergeType::Restart);

            // Set VMerge::Continue on subsequent cells
            for r in (row_start + 1)..=row_end {
                let TableChild::TableRow(row) = &mut table.rows[r];
                let TableRowChild::TableCell(cell) = &mut row.cells[col_idx];
                cell.property = cell.property.clone().vertical_merge(VMergeType::Continue);
            }
        }
    }

    Ok(())
}

// ── Image operations ────────────────────────────────────────────────

use crate::types::enums::ImagePlacement;
use docx_rs::Pic;

/// Insert an image into the document as a paragraph containing the picture.
///
/// `image_bytes` should be raw PNG image bytes.
/// `placement` controls inline vs anchored (floating).
/// `width` and `height` are in EMUs (English Metric Units). If not provided,
/// the image is inserted with a default size derived from the bytes.
/// `position` is the body index to insert at; if None, appends at end.
///
/// Returns the BodyIndex where the image paragraph was inserted.
pub fn add_image(
    docx: &mut Docx,
    image_bytes: &[u8],
    placement: ImagePlacement,
    width: Option<u32>,
    height: Option<u32>,
    position: Option<usize>,
) -> Result<usize, DocxMcpError> {
    let count = count_body_children(docx);
    let pos = position.unwrap_or(count);

    if pos > count {
        return Err(DocxMcpError::IndexOutOfBounds {
            message: "Image insert position out of bounds".into(),
            index: pos,
            max: count,
        });
    }

    // Create the picture element. We use new_with_dimensions for a default
    // 1x1 px size, then override with caller-provided EMU dimensions.
    let default_w: u32 = 300;
    let default_h: u32 = 300;
    let mut pic = Pic::new_with_dimensions(image_bytes.to_vec(), default_w, default_h);

    // Apply caller-specified dimensions (in EMUs)
    let current_w = pic.size.0;
    let current_h = pic.size.1;
    if let (Some(w), Some(h)) = (width, height) {
        pic = pic.size(w, h);
    } else if let Some(w) = width {
        pic = pic.size(w, current_h);
    } else if let Some(h) = height {
        pic = pic.size(current_w, h);
    }

    // Set placement mode
    match placement {
        ImagePlacement::Anchored => {
            pic = pic.floating();
        }
        ImagePlacement::Inline => {
            // Inline is the default position_type
        }
    }

    // Build a paragraph containing the image run
    let run = Run::new().add_image(pic);
    let para = Paragraph::new().add_run(run);

    docx.document
        .children
        .insert(pos, DocumentChild::Paragraph(Box::new(para)));

    Ok(pos)
}

// ── List operations ─────────────────────────────────────────────────

use crate::types::enums::ListType;
use crate::types::inputs::ListItem;
use docx_rs::{
    AbstractNumbering, IndentLevel, Level, LevelJc, LevelText, NumberFormat, NumberingId,
    Numbering as DocxNumbering, Start, SpecialIndentType,
};

/// Insert a bulleted or numbered list into the document.
///
/// Each `ListItem` becomes a paragraph with numbering applied.
/// `list_type` determines bullet vs decimal numbering.
/// `position` is the body index to start inserting at; if None, appends at end.
///
/// Returns `(start_index, end_index)` — the range of body indices occupied by the list.
pub fn add_list(
    docx: &mut Docx,
    list_type: ListType,
    items: &[ListItem],
    position: Option<usize>,
) -> Result<(usize, usize), DocxMcpError> {
    if items.is_empty() {
        return Err(DocxMcpError::InvalidInput {
            message: "List items cannot be empty".into(),
        });
    }

    let count = count_body_children(docx);
    let start_pos = position.unwrap_or(count);

    if start_pos > count {
        return Err(DocxMcpError::IndexOutOfBounds {
            message: "List insert position out of bounds".into(),
            index: start_pos,
            max: count,
        });
    }

    // Determine the next available abstract numbering ID by inspecting existing numberings.
    // The default abstract numbering uses ID 1, so we start from 2+.
    let abstract_num_id = docx
        .numberings
        .abstract_nums
        .iter()
        .map(|a| a.id)
        .max()
        .unwrap_or(1)
        + 1;

    let num_id = docx
        .numberings
        .numberings
        .iter()
        .map(|n| n.id)
        .max()
        .unwrap_or(1)
        + 1;

    // Determine the maximum nesting level needed
    let max_level = items
        .iter()
        .map(|item| item.level.unwrap_or(0))
        .max()
        .unwrap_or(0);

    // Build the abstract numbering definition with levels for each nesting depth
    let num_format = match list_type {
        ListType::Bulleted => "bullet",
        ListType::Numbered => "decimal",
    };

    let mut abstract_num = AbstractNumbering::new(abstract_num_id);

    for lvl in 0..=max_level {
        let level_text = match list_type {
            ListType::Bulleted => {
                // Use different bullet chars per level
                match lvl % 3 {
                    0 => "\u{2022}".to_string(),  // •
                    1 => "\u{25E6}".to_string(),  // ◦
                    _ => "\u{25AA}".to_string(),   // ▪
                }
            }
            ListType::Numbered => format!("%{}.", lvl + 1),
        };

        let indent_left = ((lvl + 1) * 420) as i32;
        let level = Level::new(
            lvl,
            Start::new(1),
            NumberFormat::new(num_format),
            LevelText::new(&level_text),
            LevelJc::new("left"),
        )
        .indent(
            Some(indent_left),
            Some(SpecialIndentType::Hanging(420)),
            None,
            None,
        );

        abstract_num = abstract_num.add_level(level);
    }

    // Register the numbering definitions directly on the numberings struct
    docx.numberings.abstract_nums.push(abstract_num);
    docx.numberings
        .numberings
        .push(DocxNumbering::new(num_id, abstract_num_id));

    // Insert each list item as a paragraph with numbering
    for (i, item) in items.iter().enumerate() {
        let level = item.level.unwrap_or(0);
        let para = Paragraph::new()
            .add_run(Run::new().add_text(&item.text))
            .numbering(NumberingId::new(num_id), IndentLevel::new(level));

        docx.document.children.insert(
            start_pos + i,
            DocumentChild::Paragraph(Box::new(para)),
        );
    }

    let end_index = start_pos + items.len() - 1;
    Ok((start_pos, end_index))
}

// ── Section / header-footer operations ──────────────────────────────

use crate::types::enums::{HeaderFooterType, SectionBreakType};
use docx_rs::{Footer, Header, PageMargin, PageSize, SectionProperty, SectionType};

/// Insert a section break by adding a paragraph with a SectionProperty.
///
/// `break_type` maps to the docx-rs SectionType.
/// Optional `page_size` and `margins` configure the new section's page layout.
pub fn add_section_break(
    docx: &mut Docx,
    break_type: SectionBreakType,
    page_size: Option<(u32, u32)>,
    margins: Option<(Option<u32>, Option<u32>, Option<u32>, Option<u32>)>,
) -> Result<(), DocxMcpError> {
    let section_type = match break_type {
        SectionBreakType::NextPage => SectionType::NextPage,
        SectionBreakType::Continuous => SectionType::Continuous,
        SectionBreakType::EvenPage => SectionType::EvenPage,
        SectionBreakType::OddPage => SectionType::OddPage,
    };

    let mut sec_prop = SectionProperty::new();
    sec_prop.section_type = Some(section_type);

    // Apply page size — default to KDP 6x9 if not specified
    let (w, h) = page_size.unwrap_or((KDP_PAGE_W, KDP_PAGE_H));
    sec_prop = sec_prop.page_size(PageSize::new().size(w, h));

    // Apply margins — default to KDP margins if not specified
    if let Some((top, bottom, left, right)) = margins {
        let mut margin = PageMargin::new();
        if let Some(t) = top {
            margin = margin.top(t as i32);
        }
        if let Some(b) = bottom {
            margin = margin.bottom(b as i32);
        }
        if let Some(l) = left {
            margin = margin.left(l as i32);
        }
        if let Some(r) = right {
            margin = margin.right(r as i32);
        }
        sec_prop = sec_prop.page_margin(margin);
    } else {
        // Default KDP margins
        sec_prop = sec_prop.page_margin(
            PageMargin::new().top(1080).bottom(1080).left(1260).right(1080).header(720).footer(720)
        );
    }

    // Insert a paragraph with the section property attached.
    // In OOXML, a section break is represented by a paragraph whose properties
    // contain a <w:sectPr> element.
    let para = Paragraph::new().section_property(sec_prop);
    docx.document
        .children
        .push(DocumentChild::Paragraph(Box::new(para)));

    Ok(())
}

/// Set header or footer content on the document.
///
/// `hf_type` determines which header/footer slot to set.
/// `content` is the text to place in the header/footer.
/// `section_index` is currently unused — docx-rs applies headers/footers
/// to the document's final section property. For multi-section documents,
/// section-specific headers would need to be set on paragraph-level section properties.
pub fn set_header_footer(
    docx: &mut Docx,
    hf_type: HeaderFooterType,
    content: &str,
    _section_index: Option<usize>,
) -> Result<(), DocxMcpError> {
    let para = Paragraph::new().add_run(Run::new().add_text(content));

    // We need to take ownership of `docx` fields temporarily because the
    // Docx builder methods consume self. Instead, we directly mutate the
    // document's section_property and related fields.

    match hf_type {
        HeaderFooterType::DefaultHeader => {
            let header = Header::new().add_paragraph(para);
            let count = docx.document_rels.header_count + 1;
            let rid = format!("rIdHeader{}", count);
            docx.document.section_property = docx
                .document
                .section_property
                .clone()
                .header(header, &rid);
            docx.document_rels.header_count = count;
            docx.content_type = docx.content_type.clone().add_header();
        }
        HeaderFooterType::FirstPageHeader => {
            let header = Header::new().add_paragraph(para);
            let count = docx.document_rels.header_count + 1;
            let rid = format!("rIdHeader{}", count);
            docx.document.section_property = docx
                .document
                .section_property
                .clone()
                .first_header(header, &rid);
            docx.document_rels.header_count = count;
            docx.content_type = docx.content_type.clone().add_header();
        }
        HeaderFooterType::EvenPageHeader => {
            let header = Header::new().add_paragraph(para);
            let count = docx.document_rels.header_count + 1;
            let rid = format!("rIdHeader{}", count);
            docx.document.section_property = docx
                .document
                .section_property
                .clone()
                .even_header(header, &rid);
            docx.document_rels.header_count = count;
            docx.content_type = docx.content_type.clone().add_header();
            docx.settings = docx.settings.clone().even_and_odd_headers();
        }
        HeaderFooterType::DefaultFooter => {
            let footer = Footer::new().add_paragraph(para);
            let count = docx.document_rels.footer_count + 1;
            let rid = format!("rIdFooter{}", count);
            docx.document.section_property = docx
                .document
                .section_property
                .clone()
                .footer(footer, &rid);
            docx.document_rels.footer_count = count;
            docx.content_type = docx.content_type.clone().add_footer();
        }
        HeaderFooterType::FirstPageFooter => {
            let footer = Footer::new().add_paragraph(para);
            let count = docx.document_rels.footer_count + 1;
            let rid = format!("rIdFooter{}", count);
            docx.document.section_property = docx
                .document
                .section_property
                .clone()
                .first_footer(footer, &rid);
            docx.document_rels.footer_count = count;
            docx.content_type = docx.content_type.clone().add_footer();
        }
        HeaderFooterType::EvenPageFooter => {
            let footer = Footer::new().add_paragraph(para);
            let count = docx.document_rels.footer_count + 1;
            let rid = format!("rIdFooter{}", count);
            docx.document.section_property = docx
                .document
                .section_property
                .clone()
                .even_footer(footer, &rid);
            docx.document_rels.footer_count = count;
            docx.content_type = docx.content_type.clone().add_footer();
            docx.settings = docx.settings.clone().even_and_odd_headers();
        }
    }

    Ok(())
}

// ── Export operations ───────────────────────────────────────────────

/// Determine if a paragraph's numbering is bulleted or numbered by inspecting
/// the abstract numbering definitions. Returns Some("bullet") or Some("decimal")
/// if the paragraph has numbering, None otherwise.
fn detect_list_type(docx: &Docx, para: &docx_rs::Paragraph) -> Option<&'static str> {
    let np = para.property.numbering_property.as_ref()?;
    let num_id = np.id.as_ref()?.id;

    // Find the Numbering entry that matches this num_id
    let numbering_entry = docx
        .numberings
        .numberings
        .iter()
        .find(|n| n.id == num_id)?;

    let abstract_num_id = numbering_entry.abstract_num_id;

    // Find the AbstractNumbering to check the format
    let abstract_num = docx
        .numberings
        .abstract_nums
        .iter()
        .find(|a| a.id == abstract_num_id)?;

    // Check the first level's number format
    let level = np.level.as_ref().map(|l| l.val).unwrap_or(0);
    let level_def = abstract_num.levels.iter().find(|l| l.level == level);

    if let Some(lvl) = level_def {
        // Serialize the level to inspect the number format
        if let Ok(val) = serde_json::to_value(lvl) {
            if let Some(fmt) = val.get("numberFormat").and_then(|f| f.as_str()) {
                return match fmt {
                    "bullet" => Some("bullet"),
                    _ => Some("decimal"),
                };
            }
        }
    }

    // Default: if we have numbering but can't determine type, treat as numbered
    Some("decimal")
}

/// Export the document as plain text.
///
/// - Paragraphs are separated by newlines
/// - Table cells are tab-separated within rows, rows separated by newlines
pub fn to_plain_text(docx: &Docx) -> String {
    let mut output = String::new();

    for child in &docx.document.children {
        match child {
            DocumentChild::Paragraph(para) => {
                let text = extract_paragraph_text(para);
                output.push_str(&text);
                output.push('\n');
            }
            DocumentChild::Table(table) => {
                for table_child in &table.rows {
                    let TableChild::TableRow(row) = table_child;
                    let cells: Vec<String> = row
                        .cells
                        .iter()
                        .map(|cell_child| {
                            let TableRowChild::TableCell(cell) = cell_child;
                            let mut cell_text = String::new();
                            for content in &cell.children {
                                if let TableCellContent::Paragraph(para) = content {
                                    if !cell_text.is_empty() {
                                        cell_text.push(' ');
                                    }
                                    cell_text.push_str(&extract_paragraph_text(para));
                                }
                            }
                            cell_text
                        })
                        .collect();
                    output.push_str(&cells.join("\t"));
                    output.push('\n');
                }
            }
            _ => {}
        }
    }

    output
}

/// Render a single run's text with markdown inline formatting (bold/italic).
fn run_to_markdown(run: &docx_rs::Run) -> String {
    let detail = extract_run_detail(run);
    if detail.text.is_empty() {
        return String::new();
    }
    let mut text = detail.text;
    if detail.bold && detail.italic {
        text = format!("***{}***", text);
    } else if detail.bold {
        text = format!("**{}**", text);
    } else if detail.italic {
        text = format!("*{}*", text);
    }
    text
}

/// Export the document as Markdown.
///
/// - Headings as `#` syntax
/// - Bold as `**`, italic as `*`
/// - Bulleted lists as `- `, numbered lists as `N. `
/// - Tables as pipe-delimited with header separator
pub fn to_markdown(docx: &Docx) -> String {
    let mut output = String::new();
    let mut numbered_counter: usize = 0;

    for child in &docx.document.children {
        match child {
            DocumentChild::Paragraph(para) => {
                // Build the run text with inline formatting
                let mut line = String::new();
                for pc in &para.children {
                    if let ParagraphChild::Run(run) = pc {
                        line.push_str(&run_to_markdown(run));
                    }
                }

                // Check heading level
                if let Some(heading) = detect_heading_level(para) {
                    // heading is like "H1", "H2", etc.
                    let level: usize = heading[1..].parse().unwrap_or(1);
                    let hashes = "#".repeat(level);
                    output.push_str(&format!("{} {}", hashes, line));
                    output.push('\n');
                    numbered_counter = 0;
                } else if let Some(list_type) = detect_list_type(docx, para) {
                    match list_type {
                        "bullet" => {
                            output.push_str(&format!("- {}", line));
                            numbered_counter = 0;
                        }
                        _ => {
                            numbered_counter += 1;
                            output.push_str(&format!("{}. {}", numbered_counter, line));
                        }
                    }
                    output.push('\n');
                } else {
                    output.push_str(&line);
                    output.push('\n');
                    numbered_counter = 0;
                }
            }
            DocumentChild::Table(table) => {
                numbered_counter = 0;
                let mut rows_data: Vec<Vec<String>> = Vec::new();

                for table_child in &table.rows {
                    let TableChild::TableRow(row) = table_child;
                    let cells: Vec<String> = row
                        .cells
                        .iter()
                        .map(|cell_child| {
                            let TableRowChild::TableCell(cell) = cell_child;
                            let mut cell_text = String::new();
                            for content in &cell.children {
                                if let TableCellContent::Paragraph(para) = content {
                                    if !cell_text.is_empty() {
                                        cell_text.push(' ');
                                    }
                                    cell_text.push_str(&extract_paragraph_text(para));
                                }
                            }
                            cell_text
                        })
                        .collect();
                    rows_data.push(cells);
                }

                if !rows_data.is_empty() {
                    // First row as header
                    let header = &rows_data[0];
                    output.push_str(&format!("| {} |", header.join(" | ")));
                    output.push('\n');

                    // Separator row
                    let sep: Vec<&str> = header.iter().map(|_| "---").collect();
                    output.push_str(&format!("| {} |", sep.join(" | ")));
                    output.push('\n');

                    // Remaining rows
                    for row in rows_data.iter().skip(1) {
                        output.push_str(&format!("| {} |", row.join(" | ")));
                        output.push('\n');
                    }
                }
            }
            _ => {}
        }
    }

    output
}

/// Render a single run as an HTML string with inline formatting.
fn run_to_html(run: &docx_rs::Run) -> String {
    let detail = extract_run_detail(run);
    if detail.text.is_empty() {
        return String::new();
    }

    let mut text = html_escape(&detail.text);

    // Wrap in bold/italic tags
    if detail.bold {
        text = format!("<strong>{}</strong>", text);
    }
    if detail.italic {
        text = format!("<em>{}</em>", text);
    }

    // Build inline style for font/size/color
    let mut styles = Vec::new();
    if let Some(ref font) = detail.font {
        styles.push(format!("font-family: {}", font));
    }
    if let Some(size) = detail.size {
        // docx-rs sizes are in half-points, convert to points
        let pt = size as f64 / 2.0;
        styles.push(format!("font-size: {}pt", pt));
    }
    if let Some(ref color) = detail.color {
        styles.push(format!("color: #{}", color));
    }

    if !styles.is_empty() {
        text = format!("<span style=\"{}\">{}</span>", styles.join("; "), text);
    }

    text
}

/// Escape HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Export the document as an HTML fragment.
///
/// - Headings as `<hN>` elements
/// - Bold as `<strong>`, italic as `<em>`
/// - Font/size/color as inline styles
/// - Tables as `<table>/<tr>/<td>`
/// - Lists as `<ul>/<ol>` with `<li>`
/// - No `<html>`, `<head>`, `<body>` wrapper
pub fn to_html(docx: &Docx) -> String {
    let mut output = String::new();

    // We need to accumulate consecutive list items into <ul>/<ol> blocks.
    // Track the current list state.
    let mut current_list_type: Option<&str> = None; // "bullet" or "decimal"

    let children = &docx.document.children;
    let mut i = 0;

    while i < children.len() {
        match &children[i] {
            DocumentChild::Paragraph(para) => {
                let list_type = detect_list_type(docx, para);

                match list_type {
                    Some(lt) => {
                        // Start a new list block if not already in one, or if type changed
                        if current_list_type != Some(lt) {
                            // Close previous list if open
                            if let Some(prev) = current_list_type {
                                output.push_str(if prev == "bullet" {
                                    "</ul>\n"
                                } else {
                                    "</ol>\n"
                                });
                            }
                            // Open new list
                            output.push_str(if lt == "bullet" {
                                "<ul>\n"
                            } else {
                                "<ol>\n"
                            });
                            current_list_type = Some(lt);
                        }

                        // Render the list item
                        let mut runs_html = String::new();
                        for pc in &para.children {
                            if let ParagraphChild::Run(run) = pc {
                                runs_html.push_str(&run_to_html(run));
                            }
                        }
                        output.push_str(&format!("<li>{}</li>\n", runs_html));
                    }
                    None => {
                        // Close any open list
                        if let Some(prev) = current_list_type.take() {
                            output.push_str(if prev == "bullet" {
                                "</ul>\n"
                            } else {
                                "</ol>\n"
                            });
                        }

                        // Render runs
                        let mut runs_html = String::new();
                        for pc in &para.children {
                            if let ParagraphChild::Run(run) = pc {
                                runs_html.push_str(&run_to_html(run));
                            }
                        }

                        if let Some(heading) = detect_heading_level(para) {
                            let level: usize = heading[1..].parse().unwrap_or(1);
                            output.push_str(&format!(
                                "<h{0}>{1}</h{0}>\n",
                                level, runs_html
                            ));
                        } else {
                            output.push_str(&format!("<p>{}</p>\n", runs_html));
                        }
                    }
                }
            }
            DocumentChild::Table(table) => {
                // Close any open list
                if let Some(prev) = current_list_type.take() {
                    output.push_str(if prev == "bullet" {
                        "</ul>\n"
                    } else {
                        "</ol>\n"
                    });
                }

                output.push_str("<table>\n");
                for table_child in &table.rows {
                    let TableChild::TableRow(row) = table_child;
                    output.push_str("<tr>");
                    for cell_child in &row.cells {
                        let TableRowChild::TableCell(cell) = cell_child;
                        let mut cell_text = String::new();
                        for content in &cell.children {
                            if let TableCellContent::Paragraph(para) = content {
                                if !cell_text.is_empty() {
                                    cell_text.push(' ');
                                }
                                cell_text.push_str(&html_escape(&extract_paragraph_text(para)));
                            }
                        }
                        output.push_str(&format!("<td>{}</td>", cell_text));
                    }
                    output.push_str("</tr>\n");
                }
                output.push_str("</table>\n");
            }
            _ => {}
        }
        i += 1;
    }

    // Close any trailing open list
    if let Some(prev) = current_list_type {
        output.push_str(if prev == "bullet" {
            "</ul>\n"
        } else {
            "</ol>\n"
        });
    }

    output
}

// ── KDP Book Creation ──────────────────────────────────────────────

use docx_rs::{
    AlignmentType, FieldCharType, InstrText, LineSpacingType,
    Style, StyleType, TableOfContents,
};

/// KDP 6x9 page setup constants (in twips: 1 inch = 1440)
const KDP_PAGE_W: u32 = 8640;   // 6"
const KDP_PAGE_H: u32 = 12960;  // 9"

/// Create a new document pre-configured for Amazon KDP 6x9 formatting.
/// Includes page size, margins, Garamond font, line spacing, heading styles,
/// page numbers in footer, and proper indent styles.
pub fn create_kdp_document(title: Option<&str>) -> Docx {
    let margin = PageMargin {
        top: 1080,    // 0.75"
        bottom: 1080, // 0.75"
        left: 1260,   // 0.875" (inside/gutter)
        right: 1080,  // 0.75" (outside)
        header: 720,
        footer: 720,
        gutter: 0,
    };

    let font = "Garamond";

    // Styles
    let body_style = Style::new("BodyText", StyleType::Paragraph)
        .name("Body Text")
        .size(22)
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font))
        .line_spacing(LineSpacing::new().line(312).line_rule(LineSpacingType::Auto));

    let body_indent = Style::new("BodyTextIndent", StyleType::Paragraph)
        .name("Body Text Indent")
        .based_on("BodyText")
        .size(22)
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font))
        .line_spacing(LineSpacing::new().line(312).line_rule(LineSpacingType::Auto))
        .indent(None, Some(SpecialIndentType::FirstLine(432)), None, None);

    let heading1 = Style::new("Heading1", StyleType::Paragraph)
        .name("heading 1")
        .size(48)
        .bold()
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font))
        .align(AlignmentType::Center)
        .line_spacing(LineSpacing::new().before(240).after(240));

    let heading2 = Style::new("Heading2", StyleType::Paragraph)
        .name("heading 2")
        .size(28)
        .bold()
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font))
        .line_spacing(LineSpacing::new().before(360).after(120));

    let heading3 = Style::new("Heading3", StyleType::Paragraph)
        .name("heading 3")
        .size(24)
        .bold()
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font))
        .line_spacing(LineSpacing::new().before(240).after(60));

    let chapter_num = Style::new("ChapterNum", StyleType::Paragraph)
        .name("Chapter Number")
        .size(24)
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font))
        .align(AlignmentType::Center)
        .line_spacing(LineSpacing::new().after(120));

    let title_style = Style::new("TitlePage", StyleType::Paragraph)
        .name("Title Page")
        .size(56)
        .bold()
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font))
        .align(AlignmentType::Center);

    let subtitle = Style::new("Subtitle", StyleType::Paragraph)
        .name("Subtitle")
        .size(28)
        .italic()
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font))
        .align(AlignmentType::Center);

    let author = Style::new("Author", StyleType::Paragraph)
        .name("Author")
        .size(28)
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font))
        .align(AlignmentType::Center);

    let copyright = Style::new("Copyright", StyleType::Paragraph)
        .name("Copyright")
        .size(18)
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font));

    // Footer with centered page number
    let page_footer = docx_rs::Footer::new().add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Center)
            .add_run(Run::new().add_field_char(docx_rs::FieldCharType::Begin, false))
            .add_run(Run::new().add_instr_text(docx_rs::InstrText::Unsupported("PAGE".to_string())))
            .add_run(Run::new().add_field_char(docx_rs::FieldCharType::Separate, false))
            .add_run(Run::new().add_text("1").size(20).fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font)))
            .add_run(Run::new().add_field_char(docx_rs::FieldCharType::End, false))
    );

    let first_footer = docx_rs::Footer::new().add_paragraph(Paragraph::new());

    let mut doc = Docx::new()
        .page_size(KDP_PAGE_W, KDP_PAGE_H)
        .page_margin(margin)
        .default_size(22)
        .default_line_spacing(LineSpacing::new().line(312).line_rule(LineSpacingType::Auto))
        .footer(page_footer)
        .first_footer(first_footer)
        .add_style(body_style)
        .add_style(body_indent)
        .add_style(heading1)
        .add_style(heading2)
        .add_style(heading3)
        .add_style(chapter_num)
        .add_style(title_style)
        .add_style(subtitle)
        .add_style(author)
        .add_style(copyright);

    // Technical book styles
    let code_block = Style::new("CodeBlock", StyleType::Paragraph)
        .name("Code Block")
        .size(18) // 9pt
        .fonts(docx_rs::RunFonts::new().ascii("Courier New").hi_ansi("Courier New"))
        .line_spacing(LineSpacing::new().line(240).line_rule(LineSpacingType::Auto));

    let callout_tip = Style::new("CalloutTip", StyleType::Paragraph)
        .name("Callout Tip")
        .size(20) // 10pt
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font))
        .indent(Some(432), None, None, None); // 0.3" left indent

    let callout_warning = Style::new("CalloutWarning", StyleType::Paragraph)
        .name("Callout Warning")
        .size(20)
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font))
        .indent(Some(432), None, None, None);

    let callout_note = Style::new("CalloutNote", StyleType::Paragraph)
        .name("Callout Note")
        .size(20)
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font))
        .indent(Some(432), None, None, None);

    let figure_caption = Style::new("FigureCaption", StyleType::Paragraph)
        .name("Figure Caption")
        .size(18) // 9pt
        .italic()
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font))
        .align(AlignmentType::Center);

    let pull_quote = Style::new("PullQuote", StyleType::Paragraph)
        .name("Pull Quote")
        .size(24) // 12pt
        .italic()
        .fonts(docx_rs::RunFonts::new().ascii(font).hi_ansi(font))
        .indent(Some(720), None, Some(720), None) // 0.5" both sides
        .line_spacing(LineSpacing::new().before(240).after(240));

    doc = doc
        .add_style(code_block)
        .add_style(callout_tip)
        .add_style(callout_warning)
        .add_style(callout_note)
        .add_style(figure_caption)
        .add_style(pull_quote);

    // Add linked TOC support
    doc = doc.add_table_of_contents(
        TableOfContents::new().heading_styles_range(1, 3).hyperlink()
    );

    if let Some(t) = title {
        doc = doc.custom_property("title", t);
    }

    doc
}

/// Insert a code block (monospace, preserves whitespace, each line as separate paragraph).
pub fn insert_code_block(
    docx: &mut Docx,
    index: usize,
    code: &str,
    _language: Option<&str>,
) -> Result<usize, DocxMcpError> {
    let count = count_body_children(docx);
    if index > count {
        return Err(DocxMcpError::IndexOutOfBounds {
            message: "Insert code block index out of bounds".into(),
            index,
            max: count,
        });
    }

    let lines: Vec<&str> = code.lines().collect();
    let num_lines = lines.len();

    // Insert lines in reverse so they end up in correct order at `index`
    for line in lines.into_iter().rev() {
        let run = Run::new()
            .add_text(line)
            .size(18)
            .fonts(RunFonts::new().ascii("Courier New").hi_ansi("Courier New"));
        let para = Paragraph::new()
            .style("CodeBlock")
            .add_run(run)
            .line_spacing(LineSpacing::new().line(240).line_rule(LineSpacingType::Auto));
        docx.document
            .children
            .insert(index, DocumentChild::Paragraph(Box::new(para)));
    }

    Ok(num_lines)
}

/// Insert a callout box (tip, warning, or note) with prefix label.
pub fn insert_callout(
    docx: &mut Docx,
    index: usize,
    callout_type: &str,
    text: &str,
) -> Result<usize, DocxMcpError> {
    let count = count_body_children(docx);
    if index > count {
        return Err(DocxMcpError::IndexOutOfBounds {
            message: "Insert callout index out of bounds".into(),
            index,
            max: count,
        });
    }

    let (prefix, style_name) = match callout_type {
        "warning" => ("⚠ WARNING: ", "CalloutWarning"),
        "note" => ("📝 NOTE: ", "CalloutNote"),
        _ => ("💡 TIP: ", "CalloutTip"),
    };

    let font = "Garamond";
    let prefix_run = Run::new()
        .add_text(prefix)
        .bold()
        .size(20)
        .fonts(RunFonts::new().ascii(font).hi_ansi(font));
    let text_run = Run::new()
        .add_text(text)
        .size(20)
        .fonts(RunFonts::new().ascii(font).hi_ansi(font));

    let para = Paragraph::new()
        .style(style_name)
        .add_run(prefix_run)
        .add_run(text_run)
        .indent(Some(432), None, None, None);

    docx.document
        .children
        .insert(index, DocumentChild::Paragraph(Box::new(para)));

    Ok(index)
}
