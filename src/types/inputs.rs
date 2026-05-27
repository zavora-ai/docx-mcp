use schemars::JsonSchema;
use serde::Deserialize;

use super::enums::*;

// ── Document lifecycle inputs ──────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct CreateDocumentInput {
    pub title: Option<String>,
    /// Optional format preset: "kdp" for Amazon KDP 6x9 book formatting (Garamond, proper margins, page numbers, heading styles)
    pub format: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct OpenDocumentInput {
    pub file_path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct SaveDocumentInput {
    pub document_handle: String,
    pub output_path: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CloseDocumentInput {
    pub document_handle: String,
}

// ── Read inputs ────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct DescribeDocumentInput {
    pub document_handle: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ReadParagraphsInput {
    pub document_handle: String,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ReadParagraphInput {
    pub document_handle: String,
    pub paragraph_index: usize,
}

#[derive(Deserialize, JsonSchema)]
pub struct ReadTableInput {
    pub document_handle: String,
    pub table_index: usize,
}

#[derive(Deserialize, JsonSchema)]
pub struct SearchTextInput {
    pub document_handle: String,
    pub query: String,
    pub mode: Option<SearchMode>,
}

// ── Write inputs ───────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct InsertParagraphInput {
    pub document_handle: String,
    pub index: usize,
    pub text: Option<String>,
    pub heading_level: Option<HeadingLevel>,
    pub style: Option<String>,
    /// If true, starts a new page before this paragraph
    pub page_break_before: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ReplaceTextInput {
    pub document_handle: String,
    pub search: String,
    pub replacement: String,
    pub replace_first: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct DeleteContentInput {
    pub document_handle: String,
    pub index: usize,
    pub run_index: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct InsertRunInput {
    pub document_handle: String,
    pub paragraph_index: usize,
    pub text: String,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub font: Option<String>,
    pub size: Option<usize>,
    pub color: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateParagraphTextInput {
    pub document_handle: String,
    pub paragraph_index: usize,
    pub text: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct BatchWriteInput {
    pub document_handle: String,
    pub operations: Vec<BatchOperation>,
}

#[derive(Deserialize, JsonSchema)]
pub struct BatchOperation {
    pub operation_type: BatchOperationType,
    pub params: serde_json::Value,
}

// ── Format inputs ──────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct SetRunFormatInput {
    pub document_handle: String,
    pub paragraph_index: usize,
    pub run_index: usize,
    pub run_end: Option<usize>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub font: Option<String>,
    pub size: Option<usize>,
    pub color: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SetParagraphFormatInput {
    pub document_handle: String,
    pub paragraph_index: usize,
    pub alignment: Option<Alignment>,
    pub line_spacing: Option<f32>,
    pub space_before: Option<u32>,
    pub space_after: Option<u32>,
    pub indent_left: Option<u32>,
    pub indent_right: Option<u32>,
    pub indent_first_line: Option<u32>,
    pub heading_level: Option<HeadingLevel>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ApplyStyleInput {
    pub document_handle: String,
    pub paragraph_index: usize,
    pub run_index: Option<usize>,
    pub style_name: String,
}

// ── Table inputs ───────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct AddTableInput {
    pub document_handle: String,
    pub rows: usize,
    pub columns: usize,
    pub position: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SetTableCellInput {
    pub document_handle: String,
    pub table_index: usize,
    pub row_index: usize,
    pub cell_index: usize,
    pub text: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct AddTableRowInput {
    pub document_handle: String,
    pub table_index: usize,
    pub cell_texts: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
pub struct MergeTableCellsInput {
    pub document_handle: String,
    pub table_index: usize,
    pub start_row: usize,
    pub start_cell: usize,
    pub end_row: usize,
    pub end_cell: usize,
    pub direction: MergeDirection,
}

// ── Structure inputs ───────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct AddImageInput {
    pub document_handle: String,
    pub image_path: String,
    pub placement: Option<ImagePlacement>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub position: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListItem {
    pub text: String,
    pub level: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AddListInput {
    pub document_handle: String,
    pub list_type: ListType,
    pub items: Vec<ListItem>,
    pub position: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AddSectionBreakInput {
    pub document_handle: String,
    pub break_type: SectionBreakType,
    pub page_width: Option<u32>,
    pub page_height: Option<u32>,
    pub orientation: Option<String>,
    pub margin_top: Option<u32>,
    pub margin_bottom: Option<u32>,
    pub margin_left: Option<u32>,
    pub margin_right: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SetHeaderFooterInput {
    pub document_handle: String,
    pub hf_type: HeaderFooterType,
    pub content: String,
    pub section_index: Option<usize>,
}

// ── Export inputs ──────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct ExportInput {
    pub document_handle: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct InsertCodeBlockInput {
    pub document_handle: String,
    pub index: usize,
    pub code: String,
    /// Optional language hint (e.g., "rust", "python")
    pub language: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct InsertCalloutInput {
    pub document_handle: String,
    pub index: usize,
    /// "tip", "warning", or "note"
    pub callout_type: String,
    pub text: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct InsertTocInput {
    pub document_handle: String,
    pub index: usize,
}
