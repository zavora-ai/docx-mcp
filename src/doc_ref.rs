use docx_rs::{DocumentChild, TableCell, TableChild, TableRowChild};

use crate::error::DocxMcpError;

/// Address a specific cell within a table.
pub struct TableCellRef {
    pub table: usize,
    pub row: usize,
    pub cell: usize,
}

/// Resolve a body child at the given index. Returns IndexOutOfBounds if invalid.
pub fn resolve_body_child(
    docx: &docx_rs::Docx,
    index: usize,
) -> Result<&DocumentChild, DocxMcpError> {
    let children = &docx.document.children;
    children.get(index).ok_or_else(|| DocxMcpError::IndexOutOfBounds {
        message: "Body child index out of bounds".into(),
        index,
        max: children.len(),
    })
}

/// Resolve a body child mutably.
pub fn resolve_body_child_mut(
    docx: &mut docx_rs::Docx,
    index: usize,
) -> Result<&mut DocumentChild, DocxMcpError> {
    let len = docx.document.children.len();
    docx.document
        .children
        .get_mut(index)
        .ok_or_else(|| DocxMcpError::IndexOutOfBounds {
            message: "Body child index out of bounds".into(),
            index,
            max: len,
        })
}

/// Navigate to a specific table cell. Returns IndexOutOfBounds with component detail.
pub fn resolve_table_cell<'a>(
    docx: &'a docx_rs::Docx,
    cell_ref: &TableCellRef,
) -> Result<&'a TableCell, DocxMcpError> {
    let child = resolve_body_child(docx, cell_ref.table)?;
    let table = match child {
        DocumentChild::Table(t) => t,
        _ => {
            return Err(DocxMcpError::InvalidInput {
                message: format!(
                    "Body child at index {} is not a table",
                    cell_ref.table
                ),
            });
        }
    };

    let row_count = table.rows.len();
    let table_row = table.rows.get(cell_ref.row).ok_or_else(|| {
        DocxMcpError::IndexOutOfBounds {
            message: format!("Table row index out of bounds (table {})", cell_ref.table),
            index: cell_ref.row,
            max: row_count,
        }
    })?;

    let row = match table_row {
        TableChild::TableRow(r) => r,
    };

    let cell_count = row.cells.len();
    let table_cell = row.cells.get(cell_ref.cell).ok_or_else(|| {
        DocxMcpError::IndexOutOfBounds {
            message: format!(
                "Table cell index out of bounds (table {}, row {})",
                cell_ref.table, cell_ref.row
            ),
            index: cell_ref.cell,
            max: cell_count,
        }
    })?;

    match table_cell {
        TableRowChild::TableCell(c) => Ok(c),
    }
}

/// Mutable version of resolve_table_cell.
pub fn resolve_table_cell_mut<'a>(
    docx: &'a mut docx_rs::Docx,
    cell_ref: &TableCellRef,
) -> Result<&'a mut TableCell, DocxMcpError> {
    let child = resolve_body_child_mut(docx, cell_ref.table)?;
    let table = match child {
        DocumentChild::Table(t) => t,
        _ => {
            return Err(DocxMcpError::InvalidInput {
                message: format!(
                    "Body child at index {} is not a table",
                    cell_ref.table
                ),
            });
        }
    };

    let row_count = table.rows.len();
    let table_row = table.rows.get_mut(cell_ref.row).ok_or_else(|| {
        DocxMcpError::IndexOutOfBounds {
            message: format!("Table row index out of bounds (table {})", cell_ref.table),
            index: cell_ref.row,
            max: row_count,
        }
    })?;

    let row = match table_row {
        TableChild::TableRow(r) => r,
    };

    let cell_count = row.cells.len();
    let table_cell = row.cells.get_mut(cell_ref.cell).ok_or_else(|| {
        DocxMcpError::IndexOutOfBounds {
            message: format!(
                "Table cell index out of bounds (table {}, row {})",
                cell_ref.table, cell_ref.row
            ),
            index: cell_ref.cell,
            max: cell_count,
        }
    })?;

    match table_cell {
        TableRowChild::TableCell(c) => Ok(c),
    }
}

/// Total number of body children (paragraphs + tables).
pub fn count_body_children(docx: &docx_rs::Docx) -> usize {
    docx.document.children.len()
}

/// Returns "paragraph" or "table" for the child at the given index.
pub fn body_child_type(
    docx: &docx_rs::Docx,
    index: usize,
) -> Result<&'static str, DocxMcpError> {
    let child = resolve_body_child(docx, index)?;
    match child {
        DocumentChild::Paragraph(_) => Ok("paragraph"),
        DocumentChild::Table(_) => Ok("table"),
        _ => Ok("other"),
    }
}
