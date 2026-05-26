use std::sync::Arc;
use tokio::sync::Mutex;

use crate::engine;
use crate::error::DocxMcpError;
use crate::store::DocumentStore;
use crate::types::inputs::ExportInput;
use crate::types::responses::{ExportResult, ToolResponse};

/// Export document content as plain text.
pub async fn to_plain_text(
    store: &Arc<Mutex<DocumentStore>>,
    input: ExportInput,
) -> Result<ToolResponse<ExportResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get(&input.document_handle)?;
    let content = engine::to_plain_text(&entry.data);
    Ok(ToolResponse::success(ExportResult { content }))
}

/// Export document content as Markdown.
pub async fn to_markdown(
    store: &Arc<Mutex<DocumentStore>>,
    input: ExportInput,
) -> Result<ToolResponse<ExportResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get(&input.document_handle)?;
    let content = engine::to_markdown(&entry.data);
    Ok(ToolResponse::success(ExportResult { content }))
}

/// Export document content as HTML.
pub async fn to_html(
    store: &Arc<Mutex<DocumentStore>>,
    input: ExportInput,
) -> Result<ToolResponse<ExportResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get(&input.document_handle)?;
    let content = engine::to_html(&entry.data);
    Ok(ToolResponse::success(ExportResult { content }))
}
