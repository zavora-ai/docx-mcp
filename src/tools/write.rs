use std::sync::Arc;
use tokio::sync::Mutex;

use crate::engine;
use crate::error::DocxMcpError;
use crate::store::DocumentStore;
use crate::types::inputs::{
    BatchWriteInput, DeleteContentInput, InsertParagraphInput, InsertRunInput, ReplaceTextInput,
    UpdateParagraphTextInput,
};
use crate::types::responses::{
    BatchResult, ConfirmationResult, DeleteResult, InsertResult, ReplaceResult, ToolResponse,
};

/// Insert a new paragraph at the given body index.
pub async fn insert_paragraph(
    store: &Arc<Mutex<DocumentStore>>,
    input: InsertParagraphInput,
) -> Result<ToolResponse<InsertResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    let index = engine::insert_paragraph(
        &mut entry.data,
        input.index,
        input.text.as_deref(),
        input.heading_level,
        input.style.as_deref(),
        input.page_break_before.unwrap_or(false),
    )?;
    Ok(ToolResponse::success(InsertResult {
        index,
        message: format!("Paragraph inserted at index {index}"),
    }))
}

/// Replace occurrences of search text with replacement text across all paragraphs.
pub async fn replace_text(
    store: &Arc<Mutex<DocumentStore>>,
    input: ReplaceTextInput,
) -> Result<ToolResponse<ReplaceResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    let count = engine::replace_text(
        &mut entry.data,
        &input.search,
        &input.replacement,
        input.replace_first.unwrap_or(false),
    );
    Ok(ToolResponse::success(ReplaceResult {
        replacements: count,
    }))
}


/// Delete a body child or a specific run within a paragraph.
pub async fn delete_content(
    store: &Arc<Mutex<DocumentStore>>,
    input: DeleteContentInput,
) -> Result<ToolResponse<DeleteResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    let updated_count = engine::delete_content(&mut entry.data, input.index, input.run_index)?;
    let message = match input.run_index {
        Some(ri) => format!("Run {ri} deleted from paragraph at index {}", input.index),
        None => format!("Body child at index {} deleted", input.index),
    };
    Ok(ToolResponse::success(DeleteResult {
        body_children_count: updated_count,
        message,
    }))
}

/// Add a formatted run to a paragraph.
pub async fn insert_run(
    store: &Arc<Mutex<DocumentStore>>,
    input: InsertRunInput,
) -> Result<ToolResponse<ConfirmationResult>, DocxMcpError> {
    let format = if input.bold.is_some()
        || input.italic.is_some()
        || input.underline.is_some()
        || input.font.is_some()
        || input.size.is_some()
        || input.color.is_some()
    {
        Some(engine::RunFormat {
            bold: input.bold,
            italic: input.italic,
            underline: input.underline,
            font: input.font,
            size: input.size,
            color: input.color,
        })
    } else {
        None
    };

    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    engine::insert_run(&mut entry.data, input.paragraph_index, &input.text, format)?;
    Ok(ToolResponse::success(ConfirmationResult {
        message: format!(
            "Run inserted into paragraph at index {}",
            input.paragraph_index
        ),
    }))
}

/// Clear all runs in a paragraph and replace with a single run containing the given text.
pub async fn update_paragraph_text(
    store: &Arc<Mutex<DocumentStore>>,
    input: UpdateParagraphTextInput,
) -> Result<ToolResponse<ConfirmationResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    engine::update_paragraph_text(&mut entry.data, input.paragraph_index, &input.text)?;
    Ok(ToolResponse::success(ConfirmationResult {
        message: format!(
            "Paragraph text updated at index {}",
            input.paragraph_index
        ),
    }))
}

/// Execute a batch of write operations sequentially, stopping on first error.
pub async fn batch_write(
    store: &Arc<Mutex<DocumentStore>>,
    input: BatchWriteInput,
) -> Result<ToolResponse<BatchResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    let completed = engine::batch_write(&mut entry.data, &input.operations)?;
    Ok(ToolResponse::success(BatchResult {
        operations_completed: completed,
        message: format!("{completed} operation(s) completed successfully"),
    }))
}
