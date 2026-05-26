use std::sync::Arc;
use tokio::sync::Mutex;

use crate::engine;
use crate::error::DocxMcpError;
use crate::store::DocumentStore;
use crate::types::inputs::{ApplyStyleInput, SetParagraphFormatInput, SetRunFormatInput};
use crate::types::responses::{ConfirmationResult, ToolResponse};

/// Apply formatting (bold, italic, underline, font, size, color) to a run or range of runs.
pub async fn set_run_format(
    store: &Arc<Mutex<DocumentStore>>,
    input: SetRunFormatInput,
) -> Result<ToolResponse<ConfirmationResult>, DocxMcpError> {
    let format = engine::RunFormat {
        bold: input.bold,
        italic: input.italic,
        underline: input.underline,
        font: input.font,
        size: input.size,
        color: input.color,
    };

    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    engine::set_run_format(
        &mut entry.data,
        input.paragraph_index,
        input.run_index,
        input.run_end,
        format,
    )?;

    let msg = match input.run_end {
        Some(end) => format!(
            "Runs {}-{} formatted in paragraph at index {}",
            input.run_index, end, input.paragraph_index
        ),
        None => format!(
            "Run {} formatted in paragraph at index {}",
            input.run_index, input.paragraph_index
        ),
    };

    Ok(ToolResponse::success(ConfirmationResult { message: msg }))
}

/// Apply paragraph-level formatting (alignment, spacing, indentation, heading level).
pub async fn set_paragraph_format(
    store: &Arc<Mutex<DocumentStore>>,
    input: SetParagraphFormatInput,
) -> Result<ToolResponse<ConfirmationResult>, DocxMcpError> {
    let format = engine::ParagraphFormat {
        alignment: input.alignment,
        line_spacing: input.line_spacing,
        space_before: input.space_before,
        space_after: input.space_after,
        indent_left: input.indent_left,
        indent_right: input.indent_right,
        indent_first_line: input.indent_first_line,
        heading_level: input.heading_level,
    };

    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    engine::set_paragraph_format(&mut entry.data, input.paragraph_index, format)?;

    Ok(ToolResponse::success(ConfirmationResult {
        message: format!(
            "Paragraph format updated at index {}",
            input.paragraph_index
        ),
    }))
}

/// Apply a named style to a paragraph or a specific run within it.
pub async fn apply_style(
    store: &Arc<Mutex<DocumentStore>>,
    input: ApplyStyleInput,
) -> Result<ToolResponse<ConfirmationResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    engine::apply_style(
        &mut entry.data,
        input.paragraph_index,
        input.run_index,
        &input.style_name,
    )?;

    let msg = match input.run_index {
        Some(ri) => format!(
            "Style '{}' applied to run {} in paragraph at index {}",
            input.style_name, ri, input.paragraph_index
        ),
        None => format!(
            "Style '{}' applied to paragraph at index {}",
            input.style_name, input.paragraph_index
        ),
    };

    Ok(ToolResponse::success(ConfirmationResult { message: msg }))
}
