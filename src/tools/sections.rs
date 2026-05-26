use std::sync::Arc;
use tokio::sync::Mutex;

use crate::engine;
use crate::error::DocxMcpError;
use crate::store::DocumentStore;
use crate::types::inputs::{AddSectionBreakInput, SetHeaderFooterInput};
use crate::types::responses::{ConfirmationResult, ToolResponse};

/// Insert a section break with optional page layout configuration.
pub async fn add_section_break(
    store: &Arc<Mutex<DocumentStore>>,
    input: AddSectionBreakInput,
) -> Result<ToolResponse<ConfirmationResult>, DocxMcpError> {
    let page_size = match (input.page_width, input.page_height) {
        (Some(w), Some(h)) => Some((w, h)),
        _ => None,
    };
    let margins = if input.margin_top.is_some()
        || input.margin_bottom.is_some()
        || input.margin_left.is_some()
        || input.margin_right.is_some()
    {
        Some((
            input.margin_top,
            input.margin_bottom,
            input.margin_left,
            input.margin_right,
        ))
    } else {
        None
    };
    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    engine::add_section_break(&mut entry.data, input.break_type, page_size, margins)?;
    Ok(ToolResponse::success(ConfirmationResult {
        message: "Section break added".into(),
    }))
}

/// Set header or footer content on the document.
pub async fn set_header_footer(
    store: &Arc<Mutex<DocumentStore>>,
    input: SetHeaderFooterInput,
) -> Result<ToolResponse<ConfirmationResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    engine::set_header_footer(
        &mut entry.data,
        input.hf_type,
        &input.content,
        input.section_index,
    )?;
    Ok(ToolResponse::success(ConfirmationResult {
        message: "Header/footer set".into(),
    }))
}
