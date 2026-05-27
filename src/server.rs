use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router,
};

use crate::store::DocumentStore;
use crate::types::inputs::*;

/// The MCP server struct that holds the DocumentStore and registers all tool handlers.
#[derive(Clone)]
pub struct DocxMcpServer {
    store: Arc<Mutex<DocumentStore>>,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

impl DocxMcpServer {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(DocumentStore::new(100, Duration::from_secs(3600)))),
            tool_router: Self::tool_router(),
        }
    }
}

impl Default for DocxMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl DocxMcpServer {
    // ── Document lifecycle tools ────────────────────────────────────

    #[tool(name = "create_document", description = "Create a new DOCX document. Set format='kdp' for Amazon KDP 6x9 book formatting (Garamond font, proper margins, page numbers, chapter styles)")]
    async fn create_document(&self, Parameters(input): Parameters<CreateDocumentInput>) -> String {
        match crate::tools::document::create_document(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "open_document", description = "Open an existing .docx file from disk for reading and editing")]
    async fn open_document(&self, Parameters(input): Parameters<OpenDocumentInput>) -> String {
        match crate::tools::document::open_document(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "save_document", description = "Save a document to disk as a .docx file")]
    async fn save_document(&self, Parameters(input): Parameters<SaveDocumentInput>) -> String {
        match crate::tools::document::save_document(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "close_document", description = "Close a document and free its memory")]
    async fn close_document(&self, Parameters(input): Parameters<CloseDocumentInput>) -> String {
        match crate::tools::document::close_document(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    // ── Read tools ─────────────────────────────────────────────────

    #[tool(name = "describe_document", description = "Get a structural overview of a document including paragraph count, table count, and body children")]
    async fn describe_document(&self, Parameters(input): Parameters<DescribeDocumentInput>) -> String {
        match crate::tools::read::describe_document(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "read_paragraphs", description = "Read paragraphs with pagination (default offset=0, limit=50)")]
    async fn read_paragraphs(&self, Parameters(input): Parameters<ReadParagraphsInput>) -> String {
        match crate::tools::read::read_paragraphs(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "read_paragraph", description = "Read a single paragraph with full detail including runs, formatting, and numbering")]
    async fn read_paragraph(&self, Parameters(input): Parameters<ReadParagraphInput>) -> String {
        match crate::tools::read::read_paragraph(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "read_table", description = "Read a table's content as structured rows and cells")]
    async fn read_table(&self, Parameters(input): Parameters<ReadTableInput>) -> String {
        match crate::tools::read::read_table(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "search_text", description = "Search for text across all paragraphs in the document (supports exact, substring, and regex modes)")]
    async fn search_text(&self, Parameters(input): Parameters<SearchTextInput>) -> String {
        match crate::tools::read::search_text(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    // ── Write tools ─────────────────────────────────────────────────

    #[tool(name = "insert_paragraph", description = "Insert a new paragraph at a specific position in the document body")]
    async fn insert_paragraph(&self, Parameters(input): Parameters<InsertParagraphInput>) -> String {
        match crate::tools::write::insert_paragraph(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "replace_text", description = "Find and replace text across all paragraphs in the document")]
    async fn replace_text(&self, Parameters(input): Parameters<ReplaceTextInput>) -> String {
        match crate::tools::write::replace_text(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "delete_content", description = "Delete a paragraph, table, or specific run by index")]
    async fn delete_content(&self, Parameters(input): Parameters<DeleteContentInput>) -> String {
        match crate::tools::write::delete_content(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "insert_run", description = "Add a formatted text run to an existing paragraph")]
    async fn insert_run(&self, Parameters(input): Parameters<InsertRunInput>) -> String {
        match crate::tools::write::insert_run(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "update_paragraph_text", description = "Replace the entire text content of a paragraph with new text")]
    async fn update_paragraph_text(&self, Parameters(input): Parameters<UpdateParagraphTextInput>) -> String {
        match crate::tools::write::update_paragraph_text(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "batch_write", description = "Execute multiple write operations in a single call, stopping on first error")]
    async fn batch_write(&self, Parameters(input): Parameters<BatchWriteInput>) -> String {
        match crate::tools::write::batch_write(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    // ── Format tools ────────────────────────────────────────────────

    #[tool(name = "set_run_format", description = "Apply formatting (bold, italic, underline, font, size, color) to a run or range of runs")]
    async fn set_run_format(&self, Parameters(input): Parameters<SetRunFormatInput>) -> String {
        match crate::tools::format::set_run_format(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "set_paragraph_format", description = "Set paragraph-level formatting (alignment, spacing, indentation, heading level)")]
    async fn set_paragraph_format(&self, Parameters(input): Parameters<SetParagraphFormatInput>) -> String {
        match crate::tools::format::set_paragraph_format(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "apply_style", description = "Apply a named style to a paragraph or specific run")]
    async fn apply_style(&self, Parameters(input): Parameters<ApplyStyleInput>) -> String {
        match crate::tools::format::apply_style(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    // ── Table tools ─────────────────────────────────────────────────

    #[tool(name = "add_table", description = "Insert a table with specified rows and columns")]
    async fn add_table(&self, Parameters(input): Parameters<AddTableInput>) -> String {
        match crate::tools::tables::add_table(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "set_table_cell", description = "Write text to a specific table cell")]
    async fn set_table_cell(&self, Parameters(input): Parameters<SetTableCellInput>) -> String {
        match crate::tools::tables::set_table_cell(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "add_table_row", description = "Append a row to an existing table")]
    async fn add_table_row(&self, Parameters(input): Parameters<AddTableRowInput>) -> String {
        match crate::tools::tables::add_table_row(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "merge_table_cells", description = "Merge table cells horizontally or vertically")]
    async fn merge_table_cells(&self, Parameters(input): Parameters<MergeTableCellsInput>) -> String {
        match crate::tools::tables::merge_table_cells(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    // ── Image tool ──────────────────────────────────────────────────

    #[tool(name = "add_image", description = "Insert an image from a file into the document")]
    async fn add_image(&self, Parameters(input): Parameters<AddImageInput>) -> String {
        match crate::tools::images::add_image(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    // ── List tool ───────────────────────────────────────────────────

    #[tool(name = "add_list", description = "Create a bulleted or numbered list")]
    async fn add_list(&self, Parameters(input): Parameters<AddListInput>) -> String {
        match crate::tools::lists::add_list(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    // ── Section tools ───────────────────────────────────────────────

    #[tool(name = "add_section_break", description = "Insert a section break with optional page layout")]
    async fn add_section_break(&self, Parameters(input): Parameters<AddSectionBreakInput>) -> String {
        match crate::tools::sections::add_section_break(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "set_header_footer", description = "Set header or footer content for a section")]
    async fn set_header_footer(&self, Parameters(input): Parameters<SetHeaderFooterInput>) -> String {
        match crate::tools::sections::set_header_footer(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    // ── Export tools ────────────────────────────────────────────────

    #[tool(name = "to_plain_text", description = "Export document content as plain text")]
    async fn to_plain_text(&self, Parameters(input): Parameters<ExportInput>) -> String {
        match crate::tools::export::to_plain_text(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "to_markdown", description = "Export document content as Markdown")]
    async fn to_markdown(&self, Parameters(input): Parameters<ExportInput>) -> String {
        match crate::tools::export::to_markdown(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "to_html", description = "Export document content as HTML fragment")]
    async fn to_html(&self, Parameters(input): Parameters<ExportInput>) -> String {
        match crate::tools::export::to_html(&self.store, input).await {
            Ok(resp) => serde_json::to_string(&resp).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
            Err(e) => serde_json::to_string(&crate::types::responses::ToolResponse::<()>::error(e.to_string()))
                .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")),
        }
    }

    #[tool(name = "add_toc", description = "Insert a linked Table of Contents at the given position. Generates from Heading1-3 styles.")]
    async fn add_toc_at(&self, Parameters(input): Parameters<InsertTocInput>) -> String {
        let mut store = self.store.lock().await;
        match store.get_mut(&input.document_handle) {
            Ok(entry) => match crate::engine::insert_toc(&mut entry.data, input.index) {
                Ok(idx) => serde_json::json!({"success":true,"data":{"index":idx},"error":null}).to_string(),
                Err(e) => serde_json::json!({"success":false,"data":null,"error":e.to_string()}).to_string(),
            },
            Err(e) => serde_json::json!({"success":false,"data":null,"error":e.to_string()}).to_string(),
        }
    }

    #[tool(name = "insert_code_block", description = "Insert a monospace code block (Courier New 9pt, preserves whitespace). For KDP technical books.")]
    async fn insert_code_block(&self, Parameters(input): Parameters<InsertCodeBlockInput>) -> String {
        let mut store = self.store.lock().await;
        match store.get_mut(&input.document_handle) {
            Ok(entry) => match crate::engine::insert_code_block(&mut entry.data, input.index, &input.code, input.language.as_deref()) {
                Ok(lines) => serde_json::json!({"success":true,"data":{"lines_inserted":lines},"error":null}).to_string(),
                Err(e) => serde_json::json!({"success":false,"data":null,"error":e.to_string()}).to_string(),
            },
            Err(e) => serde_json::json!({"success":false,"data":null,"error":e.to_string()}).to_string(),
        }
    }

    #[tool(name = "insert_callout", description = "Insert a callout box (tip/warning/note) with icon prefix. For KDP technical books.")]
    async fn insert_callout(&self, Parameters(input): Parameters<InsertCalloutInput>) -> String {
        let mut store = self.store.lock().await;
        match store.get_mut(&input.document_handle) {
            Ok(entry) => match crate::engine::insert_callout(&mut entry.data, input.index, &input.callout_type, &input.text) {
                Ok(idx) => serde_json::json!({"success":true,"data":{"index":idx},"error":null}).to_string(),
                Err(e) => serde_json::json!({"success":false,"data":null,"error":e.to_string()}).to_string(),
            },
            Err(e) => serde_json::json!({"success":false,"data":null,"error":e.to_string()}).to_string(),
        }
    }
}

#[tool_handler]
impl ServerHandler for DocxMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("DOCX document manipulation MCP server. Create, read, edit, and export Word documents.")
    }
}
