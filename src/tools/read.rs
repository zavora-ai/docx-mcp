use std::sync::Arc;
use tokio::sync::Mutex;

use crate::engine;
use crate::error::DocxMcpError;
use crate::store::DocumentStore;
use crate::types::enums::SearchMode;
use crate::types::inputs::{
    DescribeDocumentInput, ReadParagraphInput, ReadParagraphsInput, ReadTableInput, SearchTextInput,
};
use crate::types::responses::{
    DocumentDescription, PaginatedParagraphs, ParagraphDetail, SearchResult, TableData,
    ToolResponse,
};

/// Get a structural overview of a document.
pub async fn describe_document(
    store: &Arc<Mutex<DocumentStore>>,
    input: DescribeDocumentInput,
) -> Result<ToolResponse<DocumentDescription>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get(&input.document_handle)?;
    let description = engine::describe_document(&entry.data);
    Ok(ToolResponse::success(description))
}

/// Read paragraphs with pagination (default offset=0, limit=50).
pub async fn read_paragraphs(
    store: &Arc<Mutex<DocumentStore>>,
    input: ReadParagraphsInput,
) -> Result<ToolResponse<PaginatedParagraphs>, DocxMcpError> {
    let offset = input.offset.unwrap_or(0);
    let limit = input.limit.unwrap_or(50);
    let mut store = store.lock().await;
    let entry = store.get(&input.document_handle)?;
    let paginated = engine::read_paragraphs(&entry.data, offset, limit);
    Ok(ToolResponse::success(paginated))
}

/// Read a single paragraph with full detail.
pub async fn read_paragraph(
    store: &Arc<Mutex<DocumentStore>>,
    input: ReadParagraphInput,
) -> Result<ToolResponse<ParagraphDetail>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get(&input.document_handle)?;
    let detail = engine::read_paragraph(&entry.data, input.paragraph_index)?;
    Ok(ToolResponse::success(detail))
}

/// Read a table's content as structured rows and cells.
pub async fn read_table(
    store: &Arc<Mutex<DocumentStore>>,
    input: ReadTableInput,
) -> Result<ToolResponse<TableData>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get(&input.document_handle)?;
    let table_data = engine::read_table(&entry.data, input.table_index)?;
    Ok(ToolResponse::success(table_data))
}

/// Search text across all paragraphs (default mode=Substring).
pub async fn search_text(
    store: &Arc<Mutex<DocumentStore>>,
    input: SearchTextInput,
) -> Result<ToolResponse<Vec<SearchResult>>, DocxMcpError> {
    let mode = input.mode.unwrap_or(SearchMode::Substring);
    let mut store = store.lock().await;
    let entry = store.get(&input.document_handle)?;
    let results = engine::search_text(&entry.data, &input.query, mode)?;
    Ok(ToolResponse::success(results))
}
