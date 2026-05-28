//! MCP server with tool routing.

use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use crate::engine::{self, SharedStore};

// ── Input types ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateInput {
    pub title: Option<String>,
    /// "kdp:technical", "kdp:novel", or omit for blank
    pub format: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HandleInput { pub document_handle: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OpenInput { pub file_path: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SaveInput { pub document_handle: String, pub output_path: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InsertParaInput {
    pub document_handle: String,
    pub index: usize,
    pub text: String,
    /// "Heading1", "Heading2", "Heading3", "BodyText", "BodyTextIndent", "ChapterNum", "TitlePage", "Subtitle", "Author", "Copyright"
    pub style: Option<String>,
    pub page_break_before: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CodeBlockInput {
    pub document_handle: String,
    pub index: usize,
    pub code: String,
    /// "rust", "python", "bash", "json", etc.
    pub language: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CalloutInput {
    pub document_handle: String,
    pub index: usize,
    /// "tip", "warning", or "note"
    pub callout_type: String,
    pub text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TableInput {
    pub document_handle: String,
    pub index: usize,
    pub rows: usize,
    pub cols: usize,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CellInput {
    pub document_handle: String,
    pub row: usize,
    pub col: usize,
    pub text: String,
    /// Which table (0-based index among all tables in document)
    pub table_index: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ImageInput {
    pub document_handle: String,
    pub image_path: String,
    /// Width in inches
    pub width: Option<f64>,
    /// Height in inches
    pub height: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TocInput {
    pub document_handle: String,
    pub index: usize,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SceneBreakInput {
    pub document_handle: String,
    pub index: usize,
    /// "asterisks", "diamond", or "blank"
    pub style: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HeaderFooterInput {
    pub document_handle: String,
    pub header: Option<String>,
    pub footer: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExportInput { pub document_handle: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReadParasInput { pub document_handle: String, pub offset: Option<usize>, pub limit: Option<usize> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReadParaInput { pub document_handle: String, pub index: usize }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReadTableInput { pub document_handle: String, pub table_index: Option<usize> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchInput { pub document_handle: String, pub query: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReplaceInput { pub document_handle: String, pub find: String, pub replace: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteInput { pub document_handle: String, pub index: usize }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateParaInput { pub document_handle: String, pub index: usize, pub text: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RunFormatInput { pub document_handle: String, pub paragraph_index: usize, pub text: String, pub bold: Option<bool>, pub italic: Option<bool>, pub underline: Option<bool>, pub font: Option<String>, pub size: Option<f64>, pub color: Option<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ParaFormatInput { pub document_handle: String, pub index: usize, pub alignment: Option<String>, pub space_before: Option<f64>, pub space_after: Option<f64>, pub line_spacing: Option<f64> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListInput { pub document_handle: String, pub items: Vec<String>, pub list_type: Option<String>, pub index: Option<usize> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TableWithDataInput { pub document_handle: String, pub index: usize, pub headers: Vec<String>, pub rows: Vec<Vec<String>> }

// ── Server ───────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct DocxServer {
    store: SharedStore,
}

impl DocxServer {
    pub fn new() -> Self {
        Self { store: engine::new_store() }
    }
}

macro_rules! with_doc {
    ($store:expr, $handle:expr, $body:expr) => {{
        let mut store = $store.lock().await;
        match store.get_mut(&$handle) {
            Some(doc) => {
                let result = $body(doc);
                result
            }
            None => serde_json::json!({"error": "Document not found"}).to_string(),
        }
    }};
}

#[tool_router(server_handler)]
impl DocxServer {
    #[tool(description = "Create a new DOCX document. format: 'kdp:technical' for 6x9 tech book, 'kdp:novel' for 5.25x8 fiction, or omit for blank.")]
    async fn create_document(&self, Parameters(input): Parameters<CreateInput>) -> String {
        let mut doc = rdocx::Document::new();
        match input.format.as_deref() {
            Some("kdp:technical" | "kdp") => engine::create_kdp_technical(&mut doc),
            Some("kdp:novel") => engine::create_kdp_novel(&mut doc),
            _ => {}
        }
        let mut store = self.store.lock().await;
        let handle = store.insert(doc);
        serde_json::json!({"handle": handle}).to_string()
    }

    #[tool(description = "Open an existing .docx file from disk")]
    async fn open_document(&self, Parameters(input): Parameters<OpenInput>) -> String {
        match rdocx::Document::open(&input.file_path) {
            Ok(doc) => {
                let mut store = self.store.lock().await;
                let handle = store.insert(doc);
                serde_json::json!({"handle": handle}).to_string()
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Save document to disk as .docx")]
    async fn save_document(&self, Parameters(input): Parameters<SaveInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            match doc.save(&input.output_path) {
                Ok(_) => serde_json::json!({"saved": input.output_path}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        })
    }

    #[tool(description = "Close a document and free memory")]
    async fn close_document(&self, Parameters(input): Parameters<HandleInput>) -> String {
        let mut store = self.store.lock().await;
        if store.remove(&input.document_handle) {
            serde_json::json!({"closed": true}).to_string()
        } else {
            serde_json::json!({"error": "Not found"}).to_string()
        }
    }

    #[tool(description = "Get document info: paragraph count, table count, word count")]
    async fn describe_document(&self, Parameters(input): Parameters<HandleInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            serde_json::json!({
                "paragraphs": doc.paragraph_count(),
                "tables": doc.table_count(),
                "content_elements": doc.content_count(),
                "word_count": doc.word_count(),
            }).to_string()
        })
    }

    #[tool(description = "Insert a paragraph with optional style and page break. Styles: Heading1, Heading2, Heading3, BodyText, BodyTextIndent, ChapterNum, TitlePage, Subtitle, Author, Copyright")]
    async fn insert_paragraph(&self, Parameters(input): Parameters<InsertParaInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let mut para = doc.insert_paragraph(input.index, "");

            if input.page_break_before.unwrap_or(false) {
                para = para.page_break_before(true);
            }

            // Apply style-specific formatting
            let style = input.style.as_deref();
            match style {
                Some("Heading1") => {
                    para = para.alignment(rdocx::Alignment::Center)
                        .space_before(rdocx::Length::pt(24.0))
                        .space_after(rdocx::Length::pt(12.0))
                        .keep_with_next(true)
                        .outline_level(0);
                    para.add_run(&input.text).font("Garamond").size(24.0).bold(true);
                }
                Some("Heading2") => {
                    para = para.space_before(rdocx::Length::pt(18.0))
                        .space_after(rdocx::Length::pt(6.0))
                        .keep_with_next(true)
                        .outline_level(1);
                    para.add_run(&input.text).font("Garamond").size(14.0).bold(true);
                }
                Some("Heading3") => {
                    para = para.space_before(rdocx::Length::pt(12.0))
                        .space_after(rdocx::Length::pt(4.0))
                        .keep_with_next(true)
                        .outline_level(2);
                    para.add_run(&input.text).font("Garamond").size(12.0).bold(true);
                }
                Some("BodyTextIndent") => {
                    para = para.first_line_indent(rdocx::Length::inches(0.3))
                        .line_spacing_multiple(1.3);
                    para.add_run(&input.text).font("Garamond").size(11.0);
                }
                Some("ChapterNum") => {
                    para = para.alignment(rdocx::Alignment::Center)
                        .space_after(rdocx::Length::pt(6.0));
                    para.add_run(&input.text).font("Garamond").size(12.0).small_caps(true);
                }
                Some("TitlePage") => {
                    para = para.alignment(rdocx::Alignment::Center);
                    para.add_run(&input.text).font("Garamond").size(28.0).bold(true);
                }
                Some("Subtitle") => {
                    para = para.alignment(rdocx::Alignment::Center);
                    para.add_run(&input.text).font("Garamond").size(14.0).italic(true);
                }
                Some("Author") => {
                    para = para.alignment(rdocx::Alignment::Center);
                    para.add_run(&input.text).font("Garamond").size(14.0);
                }
                Some("Copyright") => {
                    para.add_run(&input.text).font("Garamond").size(9.0);
                }
                _ => {
                    // BodyText (default)
                    para = para.line_spacing_multiple(1.3);
                    para.add_run(&input.text).font("Garamond").size(11.0);
                }
            }

            serde_json::json!({"index": input.index}).to_string()
        })
    }

    #[tool(description = "Insert a syntax-highlighted code block with gray background (Courier New 9pt). Supports 'rust' highlighting.")]
    async fn insert_code_block(&self, Parameters(input): Parameters<CodeBlockInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let lines = engine::insert_code_block(doc, input.index, &input.code, input.language.as_deref());
            serde_json::json!({"lines_inserted": lines}).to_string()
        })
    }

    #[tool(description = "Insert a callout box with colored background and border (tip=green, warning=orange, note=blue)")]
    async fn insert_callout(&self, Parameters(input): Parameters<CalloutInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            engine::insert_callout(doc, input.index, &input.callout_type, &input.text);
            serde_json::json!({"inserted": true}).to_string()
        })
    }

    #[tool(description = "Insert a table at the given position")]
    async fn add_table(&self, Parameters(input): Parameters<TableInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            doc.insert_table(input.index, input.rows, input.cols);
            serde_json::json!({"index": input.index, "rows": input.rows, "cols": input.cols}).to_string()
        })
    }

    #[tool(description = "Set text in a table cell. Note: only works on the most recently inserted table in the current session.")]
    async fn set_table_cell(&self, Parameters(input): Parameters<CellInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            // rdocx 0.1.2 doesn't expose table_mut; we use content index to find and access tables
            // For now, return guidance on using add_table with cell data
            serde_json::json!({"error": "LIMITATION", "detail": "rdocx 0.1.2 does not support mutating existing tables. Use add_table to create tables, then populate cells at creation time via insert_table_with_data tool."}).to_string()
        })
    }

    #[tool(description = "Add an image from file path")]
    async fn add_image(&self, Parameters(input): Parameters<ImageInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let img_data = match std::fs::read(&input.image_path) {
                Ok(d) => d,
                Err(e) => return serde_json::json!({"error": e.to_string()}).to_string(),
            };
            let ext = input.image_path.rsplit('.').next().unwrap_or("png");
            let filename = format!("image.{}", ext);
            let w = rdocx::Length::inches(input.width.unwrap_or(4.0));
            let h = rdocx::Length::inches(input.height.unwrap_or(3.0));
            doc.add_picture(&img_data, &filename, w, h);
            serde_json::json!({"added": true}).to_string()
        })
    }

    #[tool(description = "Insert a linked Table of Contents at the given position")]
    async fn add_toc(&self, Parameters(input): Parameters<TocInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            doc.insert_toc(input.index, 3);
            serde_json::json!({"index": input.index}).to_string()
        })
    }

    #[tool(description = "Insert a scene break (for novels). style: 'asterisks', 'diamond', or 'blank'")]
    async fn insert_scene_break(&self, Parameters(input): Parameters<SceneBreakInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            engine::insert_scene_break(doc, input.index, input.style.as_deref().unwrap_or("asterisks"));
            serde_json::json!({"inserted": true}).to_string()
        })
    }

    #[tool(description = "Set header and/or footer text. Use {{PAGE}} for page numbers.")]
    async fn set_header_footer(&self, Parameters(input): Parameters<HeaderFooterInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            if let Some(h) = &input.header { doc.set_header(h); }
            if let Some(f) = &input.footer { doc.set_footer(f); }
            serde_json::json!({"set": true}).to_string()
        })
    }

    #[tool(description = "Export document as plain text")]
    async fn to_plain_text(&self, Parameters(input): Parameters<ExportInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let mut text = String::new();
            for para in doc.paragraphs() {
                text.push_str(&para.text());
                text.push('\n');
            }
            serde_json::json!({"text": text}).to_string()
        })
    }

    #[tool(description = "Export document as Markdown. Headings become #/##/###, paragraphs become text blocks.")]
    async fn to_markdown(&self, Parameters(input): Parameters<ExportInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let mut md = String::new();
            for para in doc.paragraphs() {
                let text = para.text();
                let level = para.style_id().map(|s| s.to_string());
                match level {
                    Some(ref s) if s.contains("Heading1") || s.contains("heading 1") => { md.push_str(&format!("# {}\n\n", text)); }
                    Some(ref s) if s.contains("Heading2") || s.contains("heading 2") => { md.push_str(&format!("## {}\n\n", text)); }
                    Some(ref s) if s.contains("Heading3") || s.contains("heading 3") => { md.push_str(&format!("### {}\n\n", text)); }
                    _ => { if !text.is_empty() { md.push_str(&text); md.push_str("\n\n"); } }
                }
            }
            serde_json::json!({"markdown": md}).to_string()
        })
    }

    #[tool(description = "Export document as HTML fragment.")]
    async fn to_html(&self, Parameters(input): Parameters<ExportInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let mut html = String::new();
            for para in doc.paragraphs() {
                let text = para.text();
                let level = para.style_id().map(|s| s.to_string());
                match level {
                    Some(ref s) if s.contains("Heading1") || s.contains("heading 1") => { html.push_str(&format!("<h1>{}</h1>\n", text)); }
                    Some(ref s) if s.contains("Heading2") || s.contains("heading 2") => { html.push_str(&format!("<h2>{}</h2>\n", text)); }
                    Some(ref s) if s.contains("Heading3") || s.contains("heading 3") => { html.push_str(&format!("<h3>{}</h3>\n", text)); }
                    _ => { if !text.is_empty() { html.push_str(&format!("<p>{}</p>\n", text)); } }
                }
            }
            serde_json::json!({"html": html}).to_string()
        })
    }

    #[tool(description = "Read paragraphs with pagination. Returns text and index for each paragraph.")]
    async fn read_paragraphs(&self, Parameters(input): Parameters<ReadParasInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let offset = input.offset.unwrap_or(0);
            let limit = input.limit.unwrap_or(20);
            let paras = doc.paragraphs();
            let total = paras.len();
            let items: Vec<serde_json::Value> = paras.iter().enumerate().skip(offset).take(limit).map(|(i, p)| {
                serde_json::json!({"index": i, "text": p.text(), "outline_level": p.style_id()})
            }).collect();
            serde_json::json!({"total": total, "offset": offset, "count": items.len(), "paragraphs": items}).to_string()
        })
    }

    #[tool(description = "Read a single paragraph with full detail (text, formatting info).")]
    async fn read_paragraph(&self, Parameters(input): Parameters<ReadParaInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let paras = doc.paragraphs();
            if input.index >= paras.len() { return serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(); }
            let p = &paras[input.index];
            serde_json::json!({"index": input.index, "text": p.text(), "outline_level": p.style_id()}).to_string()
        })
    }

    #[tool(description = "Read table content as structured rows and cells.")]
    async fn read_table(&self, Parameters(input): Parameters<ReadTableInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let tables = doc.tables();
            let ti = input.table_index.unwrap_or(0);
            if ti >= tables.len() { return serde_json::json!({"error": "TABLE_NOT_FOUND"}).to_string(); }
            let table = &tables[ti];
            let mut rows = Vec::new();
            for r in 0..table.row_count() {
                if let Some(row) = table.row(r) {
                    let cells: Vec<String> = (0..row.cell_count()).filter_map(|c| row.cell(c).map(|cell| cell.text())).collect();
                    rows.push(cells);
                }
            }
            serde_json::json!({"table_index": ti, "rows": table.row_count(), "columns": table.column_count(), "data": rows}).to_string()
        })
    }

    #[tool(description = "Search text across all paragraphs. Returns matching paragraph indices and text.")]
    async fn search_text(&self, Parameters(input): Parameters<SearchInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let query_lower = input.query.to_lowercase();
            let results: Vec<serde_json::Value> = doc.paragraphs().iter().enumerate().filter(|(_, p)| p.text().to_lowercase().contains(&query_lower)).map(|(i, p)| serde_json::json!({"index": i, "text": p.text()})).collect();
            serde_json::json!({"query": input.query, "matches": results.len(), "results": results}).to_string()
        })
    }

    #[tool(description = "Find and replace text across all paragraphs. Returns count of replacements made.")]
    async fn replace_text(&self, Parameters(input): Parameters<ReplaceInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let count = doc.paragraph_count();
            let mut replacements = 0;
            let mut i = 0;
            while i < doc.paragraph_count() {
                let text = doc.paragraphs()[i].text();
                if text.contains(&input.find) {
                    let new_text = text.replace(&input.find, &input.replace);
                    doc.remove_content(i);
                    doc.insert_paragraph(i, &new_text);
                    replacements += 1;
                }
                i += 1;
            }
            serde_json::json!({"find": input.find, "replace": input.replace, "replacements": replacements}).to_string()
        })
    }

    #[tool(description = "Delete content element (paragraph or table) by index.")]
    async fn delete_content(&self, Parameters(input): Parameters<DeleteInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            if doc.remove_content(input.index) {
                serde_json::json!({"deleted": true, "index": input.index}).to_string()
            } else {
                serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string()
            }
        })
    }

    #[tool(description = "Update paragraph text at given index (replaces entire paragraph text).")]
    async fn update_paragraph_text(&self, Parameters(input): Parameters<UpdateParaInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            if input.index >= doc.paragraph_count() { return serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(); }
            doc.remove_content(input.index);
            doc.insert_paragraph(input.index, &input.text);
            serde_json::json!({"updated": true, "index": input.index}).to_string()
        })
    }

    #[tool(description = "Add a formatted text run to a paragraph. Supports bold, italic, underline, font, size, color.")]
    async fn insert_run(&self, Parameters(input): Parameters<RunFormatInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            match doc.paragraph_mut(input.paragraph_index) {
                Some(mut para) => {
                    let mut run = para.add_run(&input.text);
                    if let Some(f) = &input.font { run = run.font(f); }
                    if let Some(s) = input.size { run = run.size(s); }
                    if input.bold.unwrap_or(false) { run = run.bold(true); }
                    if input.italic.unwrap_or(false) { run = run.italic(true); }
                    if input.underline.unwrap_or(false) { run = run.underline(true); }
                    if let Some(c) = &input.color { run.color(c); }
                    serde_json::json!({"added": true, "paragraph_index": input.paragraph_index}).to_string()
                }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Set paragraph formatting: alignment (left, center, right, justify), spacing, line spacing.")]
    async fn set_paragraph_format(&self, Parameters(input): Parameters<ParaFormatInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            match doc.paragraph_mut(input.index) {
                Some(mut para) => {
                    if let Some(align) = &input.alignment {
                        para = match align.as_str() {
                            "center" => para.alignment(rdocx::Alignment::Center),
                            "right" => para.alignment(rdocx::Alignment::Right),
                            "justify" => para.alignment(rdocx::Alignment::Justify),
                            _ => para.alignment(rdocx::Alignment::Left),
                        };
                    }
                    if let Some(sb) = input.space_before { para = para.space_before(rdocx::Length::pt(sb)); }
                    if let Some(sa) = input.space_after { para = para.space_after(rdocx::Length::pt(sa)); }
                    if let Some(ls) = input.line_spacing { para.line_spacing_multiple(ls); }
                    serde_json::json!({"formatted": true, "index": input.index}).to_string()
                }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Create a bulleted or numbered list. list_type: 'bullet' or 'numbered'.")]
    async fn add_list(&self, Parameters(input): Parameters<ListInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let is_numbered = input.list_type.as_deref() == Some("numbered");
            for item in &input.items {
                if is_numbered { doc.add_numbered_list_item(item, 0); }
                else { doc.add_bullet_list_item(item, 0); }
            }
            serde_json::json!({"items_added": input.items.len(), "type": if is_numbered { "numbered" } else { "bullet" }}).to_string()
        })
    }

    #[tool(description = "Insert a table with headers and pre-populated row data. This is the recommended way to create tables with content.")]
    async fn insert_table_with_data(&self, Parameters(input): Parameters<TableWithDataInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut rdocx::Document| {
            let cols = input.headers.len();
            let rows = input.rows.len() + 1; // +1 for header
            let mut table = doc.insert_table(input.index, rows, cols);
            // Set headers
            for (c, header) in input.headers.iter().enumerate() {
                if let Some(mut cell) = table.cell(0, c) { cell.set_text(header); }
            }
            // Set data rows
            for (r, row) in input.rows.iter().enumerate() {
                for (c, val) in row.iter().enumerate() {
                    if let Some(mut cell) = table.cell(r + 1, c) { cell.set_text(val); }
                }
            }
            serde_json::json!({"index": input.index, "rows": rows, "cols": cols}).to_string()
        })
    }
}
