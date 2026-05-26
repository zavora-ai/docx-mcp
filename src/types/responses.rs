use serde::Serialize;

/// Universal response envelope for all tools.
#[derive(Serialize)]
pub struct ToolResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ToolResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// Document info returned by create/open.
#[derive(Serialize)]
pub struct DocumentInfo {
    pub handle: String,
    pub message: String,
}

/// Describe document response.
#[derive(Serialize)]
pub struct DocumentDescription {
    pub paragraph_count: usize,
    pub table_count: usize,
    pub body_children: Vec<BodyChildInfo>,
    pub section_count: usize,
    pub has_headers: bool,
    pub has_footers: bool,
}

#[derive(Serialize)]
pub struct BodyChildInfo {
    pub index: usize,
    pub child_type: String,
    pub heading_level: Option<String>,
    pub style_name: Option<String>,
}

/// Read paragraphs response with pagination.
#[derive(Serialize)]
pub struct PaginatedParagraphs {
    pub paragraphs: Vec<ParagraphSummary>,
    pub total_count: usize,
    pub offset: usize,
    pub returned: usize,
}

#[derive(Serialize)]
pub struct ParagraphSummary {
    pub index: usize,
    pub text: String,
    pub style: Option<String>,
    pub heading_level: Option<String>,
}

/// Read paragraph detail response.
#[derive(Serialize)]
pub struct ParagraphDetail {
    pub index: usize,
    pub text: String,
    pub runs: Vec<RunDetail>,
    pub style: Option<String>,
    pub heading_level: Option<String>,
    pub numbering: Option<NumberingInfo>,
    pub alignment: Option<String>,
}

#[derive(Serialize)]
pub struct RunDetail {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub font: Option<String>,
    pub size: Option<usize>,
    pub color: Option<String>,
}

#[derive(Serialize)]
pub struct NumberingInfo {
    pub level: usize,
    pub num_id: usize,
}

/// Table data response.
#[derive(Serialize)]
pub struct TableData {
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
    pub column_count: usize,
}

/// Search results.
#[derive(Serialize)]
pub struct SearchResult {
    pub index: usize,
    pub matched_text: String,
    pub paragraph_text: String,
}

/// Write operation responses.
#[derive(Serialize)]
pub struct InsertResult {
    pub index: usize,
    pub message: String,
}

#[derive(Serialize)]
pub struct ReplaceResult {
    pub replacements: usize,
}

#[derive(Serialize)]
pub struct DeleteResult {
    pub body_children_count: usize,
    pub message: String,
}

#[derive(Serialize)]
pub struct BatchResult {
    pub operations_completed: usize,
    pub message: String,
}

#[derive(Serialize)]
pub struct TableInsertResult {
    pub index: usize,
    pub rows: usize,
    pub columns: usize,
}

#[derive(Serialize)]
pub struct AddRowResult {
    pub row_index: usize,
}

#[derive(Serialize)]
pub struct ListInsertResult {
    pub start_index: usize,
    pub end_index: usize,
}

#[derive(Serialize)]
pub struct ExportResult {
    pub content: String,
}

#[derive(Serialize)]
pub struct SaveResult {
    pub path: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct ConfirmationResult {
    pub message: String,
}
