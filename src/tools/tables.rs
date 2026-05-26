use std::sync::Arc;
use tokio::sync::Mutex;

use crate::engine;
use crate::error::DocxMcpError;
use crate::store::DocumentStore;
use crate::types::inputs::{AddTableInput, AddTableRowInput, MergeTableCellsInput, SetTableCellInput};
use crate::types::responses::{AddRowResult, ConfirmationResult, TableInsertResult, ToolResponse};

/// Insert a new table with the specified dimensions.
pub async fn add_table(
    store: &Arc<Mutex<DocumentStore>>,
    input: AddTableInput,
) -> Result<ToolResponse<TableInsertResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    let (index, rows, columns) =
        engine::add_table(&mut entry.data, input.rows, input.columns, input.position)?;
    Ok(ToolResponse::success(TableInsertResult {
        index,
        rows,
        columns,
    }))
}

/// Set the text content of a specific table cell.
pub async fn set_table_cell(
    store: &Arc<Mutex<DocumentStore>>,
    input: SetTableCellInput,
) -> Result<ToolResponse<ConfirmationResult>, DocxMcpError> {
    let cell_ref = crate::doc_ref::TableCellRef {
        table: input.table_index,
        row: input.row_index,
        cell: input.cell_index,
    };

    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    engine::set_table_cell(&mut entry.data, &cell_ref, &input.text)?;
    Ok(ToolResponse::success(ConfirmationResult {
        message: format!(
            "Cell ({}, {}) set in table at index {}",
            input.row_index, input.cell_index, input.table_index
        ),
    }))
}

/// Append a new row to an existing table.
pub async fn add_table_row(
    store: &Arc<Mutex<DocumentStore>>,
    input: AddTableRowInput,
) -> Result<ToolResponse<AddRowResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    let row_index =
        engine::add_table_row(&mut entry.data, input.table_index, input.cell_texts)?;
    Ok(ToolResponse::success(AddRowResult { row_index }))
}

/// Merge table cells horizontally or vertically.
pub async fn merge_table_cells(
    store: &Arc<Mutex<DocumentStore>>,
    input: MergeTableCellsInput,
) -> Result<ToolResponse<ConfirmationResult>, DocxMcpError> {
    let mut store = store.lock().await;
    let entry = store.get_mut(&input.document_handle)?;
    engine::merge_table_cells(
        &mut entry.data,
        input.table_index,
        (input.start_row, input.start_cell),
        (input.end_row, input.end_cell),
        input.direction,
    )?;
    Ok(ToolResponse::success(ConfirmationResult {
        message: format!(
            "Cells merged in table at index {}",
            input.table_index
        ),
    }))
}
