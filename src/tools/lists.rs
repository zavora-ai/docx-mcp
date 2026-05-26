use std::sync::Arc;
use tokio::sync::Mutex;

use crate::engine;
use crate::error::DocxMcpError;
use crate::store::DocumentStore;
use crate::types::inputs::AddListInput;
use crate::types::responses::{ListInsertResult, ToolResponse};

/// Insert a bulleted or numbered list into the document.
pub async fn add_list(
    store: &Arc<Mutex<DocumentStore>>,
    input: AddListInput,
) -> Result<ToolResponse<ListInsertResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    let (start_index, end_index) =
        engine::add_list(&mut entry.data, input.list_type, &input.items, input.position)?;
    Ok(ToolResponse::success(ListInsertResult {
        start_index,
        end_index,
    }))
}
