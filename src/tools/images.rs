use std::sync::Arc;
use tokio::sync::Mutex;

use crate::engine;
use crate::error::DocxMcpError;
use crate::store::DocumentStore;
use crate::types::enums::ImagePlacement;
use crate::types::inputs::AddImageInput;
use crate::types::responses::{InsertResult, ToolResponse};

/// Insert an image into the document from a file on disk.
pub async fn add_image(
    store: &Arc<Mutex<DocumentStore>>,
    input: AddImageInput,
) -> Result<ToolResponse<InsertResult>, DocxMcpError> {
    let image_bytes = tokio::fs::read(&input.image_path).await?;
    let placement = input.placement.unwrap_or(ImagePlacement::Inline);
    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    let index = engine::add_image(
        &mut entry.data,
        &image_bytes,
        placement,
        input.width,
        input.height,
        input.position,
    )?;
    Ok(ToolResponse::success(InsertResult {
        index,
        message: format!("Image inserted at index {index}"),
    }))
}
