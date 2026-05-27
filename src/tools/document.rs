use std::sync::Arc;
use tokio::sync::Mutex;

use crate::engine;
use crate::error::DocxMcpError;
use crate::store::DocumentStore;
use crate::types::inputs::{CloseDocumentInput, CreateDocumentInput, OpenDocumentInput, SaveDocumentInput};
use crate::types::responses::{ConfirmationResult, DocumentInfo, SaveResult, ToolResponse};

/// Create a new empty document, store it, and return the handle.
pub async fn create_document(
    store: &Arc<Mutex<DocumentStore>>,
    input: CreateDocumentInput,
) -> Result<ToolResponse<DocumentInfo>, DocxMcpError> {
    let doc = match input.format.as_deref() {
        Some("kdp") => engine::create_kdp_document(input.title.as_deref()),
        _ => engine::create_document(input.title.as_deref()),
    };
    let mut store = store.lock().await;
    let handle = store.insert(doc, None);
    Ok(ToolResponse::success(DocumentInfo {
        handle,
        message: "Document created".into(),
    }))
}

/// Read a .docx file from disk, parse it, store it, and return the handle.
pub async fn open_document(
    store: &Arc<Mutex<DocumentStore>>,
    input: OpenDocumentInput,
) -> Result<ToolResponse<DocumentInfo>, DocxMcpError> {
    let bytes = tokio::fs::read(&input.file_path).await?;
    let doc = engine::open_document(&bytes)?;
    let mut store = store.lock().await;
    let handle = store.insert(doc, Some(input.file_path));
    Ok(ToolResponse::success(DocumentInfo {
        handle,
        message: "Document opened".into(),
    }))
}

/// Serialize a document to bytes and write to disk.
/// Uses output_path if provided, otherwise falls back to the original file path.
pub async fn save_document(
    store: &Arc<Mutex<DocumentStore>>,
    input: SaveDocumentInput,
) -> Result<ToolResponse<SaveResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get(&input.document_handle)?;

    let path = input
        .output_path
        .or_else(|| entry.file_path.clone())
        .ok_or_else(|| DocxMcpError::InvalidInput {
            message: "No output path provided and document has no original file path".into(),
        })?;

    let bytes = engine::save_document(&entry.data)?;
    drop(store);

    tokio::fs::write(&path, bytes).await?;

    Ok(ToolResponse::success(SaveResult {
        path,
        message: "Document saved".into(),
    }))
}

/// Remove a document from the store.
pub async fn close_document(
    store: &Arc<Mutex<DocumentStore>>,
    input: CloseDocumentInput,
) -> Result<ToolResponse<ConfirmationResult>, DocxMcpError> {
    let mut store = store.lock().await;
    store.remove(&input.document_handle)?;
    Ok(ToolResponse::success(ConfirmationResult {
        message: format!("Document {} closed", input.document_handle),
    }))
}
