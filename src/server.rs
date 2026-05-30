//! MCP server with tool routing.

use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use crate::engine::{self, SharedStore};

// ── Input types ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateInput {
    pub title: Option<String>,
    /// "kdp:technical", "kdp:novel", "kdp:cookbook", "kdp:children", "kdp:interior_design", "kdp:encyclopedia", "kdp:manga", or omit for blank
    pub format: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NovelInput {
    pub title: String,
    pub author: String,
    /// Trim width in inches (e.g. 5.0, 5.25, 5.5, 6.0). Default 5.25.
    pub trim_width: Option<f64>,
    /// Trim height in inches (e.g. 8.0, 8.5, 9.0). Default 8.0.
    pub trim_height: Option<f64>,
    /// Body/heading font family. Default "Garamond".
    pub font: Option<String>,
    /// Body text size in points. Default 11.5.
    pub body_pt: Option<f64>,
    /// Line spacing multiple. Default 1.3.
    pub line_spacing: Option<f64>,
    /// Justify body text. Default true.
    pub justified: Option<bool>,
    /// Show running header (author / title). Default true.
    pub running_header: Option<bool>,
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
    /// Clockwise rotation in degrees
    pub rotation: Option<f64>,
    /// Flip horizontally
    pub flip_h: Option<bool>,
    /// Flip vertically
    pub flip_v: Option<bool>,
    /// Crop percentages [left, top, right, bottom] (e.g. 10 = crop 10%)
    pub crop: Option<[f64; 4]>,
    /// Border color hex (e.g. "000000")
    pub border_color: Option<String>,
    /// Border width in points (default 1.0 when border_color set)
    pub border_width: Option<f64>,
    /// Drop shadow color hex (e.g. "808080")
    pub shadow_color: Option<String>,
    /// Accessibility alt-text title
    pub alt_text: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PageBackgroundInput {
    pub document_handle: String,
    /// Body index where the background anchors (place after a page break so it
    /// covers that page). Use document content_count for the current end.
    pub index: usize,
    pub image_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TocInput {
    pub document_handle: String,
    pub index: usize,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ChartSeriesInput {
    pub name: String,
    pub values: Vec<f64>,
}

/// Build a DataLabels config from chart input, starting from the per-kind
/// default and applying any caller overrides. Returns None only if the caller
/// supplied nothing (so the library default applies).
fn build_labels(input: &ChartInput, kind: zavora_docx::ChartKind) -> Option<zavora_docx::DataLabels> {
    let any = input.label_position.is_some()
        || input.label_show_value.is_some()
        || input.label_show_category.is_some()
        || input.label_show_percent.is_some()
        || input.label_color.is_some();
    if !any {
        return None;
    }
    let mut l = zavora_docx::Chart::default_labels(kind);
    if let Some(ref p) = input.label_position {
        l.position = match p.as_str() {
            "outEnd" => Some(zavora_docx::LabelPosition::OutsideEnd),
            "inEnd" => Some(zavora_docx::LabelPosition::InsideEnd),
            "ctr" => Some(zavora_docx::LabelPosition::Center),
            "inBase" => Some(zavora_docx::LabelPosition::InsideBase),
            "bestFit" => Some(zavora_docx::LabelPosition::BestFit),
            _ => l.position,
        };
    }
    if let Some(v) = input.label_show_value { l.show_value = v; }
    if let Some(v) = input.label_show_category { l.show_category = v; }
    if let Some(v) = input.label_show_percent { l.show_percent = v; }
    if input.label_color.is_some() { l.color = input.label_color.clone(); }
    Some(l)
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ChartInput {
    pub document_handle: String,
    /// Chart type: "bar", "column", "line", "pie", "area".
    pub kind: String,
    pub categories: Vec<String>,
    pub series: Vec<ChartSeriesInput>,
    pub title: Option<String>,
    pub width_inches: Option<f64>,
    pub height_inches: Option<f64>,
    /// Data-label position: "outEnd", "inEnd", "ctr", "inBase", "bestFit".
    pub label_position: Option<String>,
    /// Show value on labels.
    pub label_show_value: Option<bool>,
    /// Show category name on labels.
    pub label_show_category: Option<bool>,
    /// Show percentage on labels (pie).
    pub label_show_percent: Option<bool>,
    /// Fixed label text color (hex, e.g. "FFFFFF"). Omit for theme default.
    pub label_color: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TextBoxInput {
    pub document_handle: String,
    /// Lines of text inside the box.
    pub lines: Vec<String>,
    pub width_inches: Option<f64>,
    pub height_inches: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ShapeInput {
    pub document_handle: String,
    /// Preset geometry: "rect", "ellipse", "roundRect", "rightArrow", "star5", etc.
    pub geometry: String,
    pub width_inches: Option<f64>,
    pub height_inches: Option<f64>,
    /// Solid fill color hex (e.g. "FFCC00").
    pub fill_color: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EquationInput {
    pub document_handle: String,
    /// LaTeX-subset equation, e.g. "\\frac{a}{b}^2", "\\sum_{i=1}^{n} i", "\\sqrt{x}".
    pub latex: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ContentControlInput {
    pub document_handle: String,
    /// Control kind: "text", "rich_text", "dropdown", "combo", "date", "checkbox".
    pub kind: String,
    /// Tag identifying the control.
    pub tag: String,
    /// Display/placeholder text shown inside the control.
    pub placeholder: Option<String>,
    /// Options as [display, value] pairs (for dropdown/combo).
    pub options: Option<Vec<[String; 2]>>,
    /// Date display format (for date), e.g. "yyyy-MM-dd".
    pub date_format: Option<String>,
    /// Initial checked state (for checkbox).
    pub checked: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DocumentSettingsInput {
    pub document_handle: String,
    /// Default automatic tab stop width, in inches.
    pub default_tab_stop_inches: Option<f64>,
    /// Mirror inside/outside margins for double-sided printing.
    pub mirror_margins: Option<bool>,
    /// Enable the track-changes (revisions) flag.
    pub track_changes: Option<bool>,
    /// Open-zoom level as a percentage (e.g. 100, 150).
    pub zoom_percent: Option<u32>,
    /// Default proofing/theme language (e.g. "en-US", "fr-FR").
    pub language: Option<String>,
    /// Force Word to recalculate fields (TOC, PAGEREF) on open.
    pub update_fields: Option<bool>,
    /// Enable automatic hyphenation.
    pub auto_hyphenation: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SceneBreakInput {
    pub document_handle: String,
    pub index: usize,
    /// "asterisks", "diamond", or "blank"
    pub style: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ChapterOpeningInput {
    pub document_handle: String,
    pub index: usize,
    /// Full text of the chapter's first paragraph.
    pub text: String,
    /// Font family for the drop cap + lead-in. Default "Garamond".
    pub font: Option<String>,
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
pub struct SavePdfInput { pub document_handle: String, pub output_path: String }

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
pub struct FieldInput { pub document_handle: String, pub paragraph_index: usize, pub instruction: String, pub cached: Option<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ParaFormatInput { pub document_handle: String, pub index: usize, pub alignment: Option<String>, pub space_before: Option<f64>, pub space_after: Option<f64>, pub line_spacing: Option<f64> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListInput { pub document_handle: String, pub items: Vec<String>, pub list_type: Option<String>, pub index: Option<usize> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TableWithDataInput { pub document_handle: String, pub index: usize, pub headers: Vec<String>, pub rows: Vec<Vec<String>> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MergeCellsInput {
    pub document_handle: String,
    pub table_index: Option<usize>,
    /// "horizontal" merges columns via grid_span, "vertical" merges rows via v_merge
    pub direction: String,
    pub start_row: usize,
    pub start_col: usize,
    /// Number of cells to merge (columns for horizontal, rows for vertical)
    pub span: usize,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddRowInput {
    pub document_handle: String,
    pub table_index: Option<usize>,
    /// Cell values for the new row
    pub cells: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PageLayoutInput {
    pub document_handle: String,
    /// Page width in inches (e.g. 8.5 for US Letter)
    pub width: Option<f64>,
    /// Page height in inches (e.g. 11 for US Letter)
    pub height: Option<f64>,
    /// "portrait" or "landscape"
    pub orientation: Option<String>,
    /// Margins in inches: [top, right, bottom, left]
    pub margins: Option<Vec<f64>>,
    /// Number of text columns
    pub columns: Option<u32>,
    /// Gutter width in inches (extra inside margin for binding)
    pub gutter: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SetMetadataInput {
    pub document_handle: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    /// Company name (docProps/app.xml).
    pub company: Option<String>,
    /// Authoring application name (docProps/app.xml).
    pub application: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MergeDocumentsInput {
    pub document_handle: String,
    /// Path to the document to append
    pub other_path: String,
    /// Section break type between documents: "nextPage", "continuous", "evenPage", "oddPage"
    pub break_type: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RenderPageInput {
    pub document_handle: String,
    /// 0-based page index to render
    pub page_index: usize,
    /// DPI for rendering (default 150)
    pub dpi: Option<f64>,
    /// Output file path for the PNG
    pub output_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RegexReplaceInput {
    pub document_handle: String,
    /// Regex pattern to match
    pub pattern: String,
    /// Replacement string (supports $1, $2 capture groups)
    pub replacement: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FormatTableInput {
    pub document_handle: String,
    pub table_index: Option<usize>,
    /// Table width as percentage (0-100) or omit for auto
    pub width_pct: Option<f64>,
    /// Table alignment: "left", "center", "right"
    pub alignment: Option<String>,
    /// Border style: "single", "double", "dashed", "dotted", "none"
    pub border_style: Option<String>,
    /// Border color (hex, e.g. "000000")
    pub border_color: Option<String>,
    /// Border size in eighths of a point
    pub border_size: Option<u32>,
    /// Cell margins in points: [top, right, bottom, left]
    pub cell_margins: Option<Vec<f64>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FormatCellInput {
    pub document_handle: String,
    pub table_index: Option<usize>,
    pub row: usize,
    pub col: usize,
    /// Background shading color (hex, e.g. "E2EFDA")
    pub shading: Option<String>,
    /// Vertical alignment: "top", "center", "bottom"
    pub vertical_alignment: Option<String>,
    /// Cell width in inches
    pub width: Option<f64>,
    /// Prevent text wrapping
    pub no_wrap: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SectionBreakInput {
    pub document_handle: String,
    pub index: usize,
    /// Break type: "nextPage", "continuous", "evenPage", "oddPage"
    pub break_type: Option<String>,
    /// Optional page width in inches for the new section
    pub page_width: Option<f64>,
    /// Optional page height in inches for the new section
    pub page_height: Option<f64>,
    /// Optional orientation for the new section: "portrait" or "landscape"
    pub orientation: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FootnoteInput { pub document_handle: String, pub text: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FootnoteRefInput { pub document_handle: String, pub paragraph_index: usize, pub footnote_id: i32 }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HyperlinkInput { pub document_handle: String, pub paragraph_index: usize, pub url: String, pub text: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BookmarkInput { pub document_handle: String, pub paragraph_index: usize, pub id: u32, pub name: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CommentInput { pub document_handle: String, pub id: u32, pub author: String, pub text: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CommentRangeInput { pub document_handle: String, pub paragraph_index: usize, pub comment_id: u32, pub commented_text: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CommentReplyInput { pub document_handle: String, pub id: u32, pub parent_id: u32, pub author: String, pub text: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ResolveCommentInput { pub document_handle: String, pub id: u32 }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WatermarkInput { pub document_handle: String, pub text: String, /// Hex color e.g. "C0C0C0"
    pub color: Option<String>, /// Rotation in degrees (default -45)
    pub rotation: Option<i32> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TrackedInsertInput { pub document_handle: String, pub paragraph_index: usize, pub text: String, pub author: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TrackedDeleteInput { pub document_handle: String, pub paragraph_index: usize, pub text: String, pub author: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TextFieldInput { pub document_handle: String, pub paragraph_index: usize, pub name: String, pub default_value: Option<String>, pub label: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CheckboxInput { pub document_handle: String, pub paragraph_index: usize, pub name: String, pub checked: Option<bool>, pub label: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DropdownInput { pub document_handle: String, pub paragraph_index: usize, pub name: String, pub options: Vec<String>, pub selected: Option<usize>, pub label: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ProtectInput { pub document_handle: String, /// "readonly", "forms", "comments", "trackedChanges"
    pub protection_type: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DropCapInput { pub document_handle: String, pub paragraph_index: usize, /// Number of lines to span (2-4)
    pub lines: u32 }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TextEffectInput { pub document_handle: String, pub paragraph_index: usize, pub text: String,
    /// "shadow", "glow", "outline", "reflection"
    pub effect: String,
    /// Color hex for shadow/glow/outline (e.g. "4472C4")
    pub color: Option<String>,
    /// Size in points (blur for shadow, radius for glow, width for outline)
    pub size: Option<f64> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ThemeColorRunInput { pub document_handle: String, pub paragraph_index: usize, pub text: String,
    /// Theme color: "accent1"-"accent6", "dk1", "dk2", "lt1", "lt2", "hlink"
    pub theme_color: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BandedTableInput { pub document_handle: String, pub table_index: Option<usize>,
    /// Hex color for alternating rows (e.g. "D9E2F3")
    pub band_color: String,
    /// Header background color
    pub header_bg: Option<String>,
    /// Header text color
    pub header_text: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LineNumberingInput { pub document_handle: String,
    /// Show every Nth line number (1=every line, 5=every 5th)
    pub count_by: Option<u32>,
    /// "continuous", "newPage", or "newSection"
    pub restart: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CustomListInput { pub document_handle: String, pub items: Vec<String>,
    /// "decimal", "upperRoman", "lowerRoman", "upperLetter", "lowerLetter", "bullet"
    pub format: String,
    /// Custom bullet character (only for format="bullet"), e.g. "★", "→", "◆"
    pub bullet_char: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ThemeInput { pub document_handle: String,
    /// Major font (headings)
    pub major_font: String,
    /// Minor font (body)
    pub minor_font: String,
    /// Accent colors as hex: [accent1, accent2, accent3, accent4, accent5, accent6]
    pub accent_colors: Option<Vec<String>> }

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
    #[tool(description = "Create a new DOCX document. format: 'kdp:technical' (6x9), 'kdp:novel' (5.25x8), 'kdp:cookbook' (8x10), 'kdp:children' (8.5x8.5 square), 'kdp:interior_design' (8.5x11), 'kdp:encyclopedia' (8.5x11 2-col), 'kdp:manga' (5x7.5), or omit for blank.")]
    async fn create_document(&self, Parameters(input): Parameters<CreateInput>) -> String {
        let mut doc = zavora_docx::Document::new();
        match input.format.as_deref() {
            Some("kdp:technical" | "kdp") => engine::create_kdp_technical(&mut doc),
            Some("kdp:novel") => engine::create_kdp_novel(&mut doc),
            Some("kdp:cookbook") => engine::create_kdp_cookbook(&mut doc),
            Some("kdp:children") => engine::create_kdp_children(&mut doc),
            Some("kdp:interior_design") => engine::create_kdp_interior_design(&mut doc),
            Some("kdp:encyclopedia") => engine::create_kdp_encyclopedia(&mut doc),
            Some("kdp:manga") => engine::create_kdp_manga(&mut doc),
            _ => {}
        }
        let mut store = self.store.lock().await;
        let handle = store.insert(doc);
        serde_json::json!({"handle": handle}).to_string()
    }

    #[tool(description = "Create a professionally-styled novel with author-specific settings (trim size, font, spacing). Sets justified body text, widow control, chapter-opener heading style, and a running header. Use insert_paragraph with style=Heading1 for chapter titles (drives the TOC), TitlePage/Subtitle/Author for the title page, and BodyText/BodyTextIndent for prose.")]
    async fn create_novel(&self, Parameters(input): Parameters<NovelInput>) -> String {
        let mut doc = zavora_docx::Document::new();
        let cfg = engine::NovelConfig {
            title: input.title,
            author: input.author,
            trim: (input.trim_width.unwrap_or(5.25), input.trim_height.unwrap_or(8.0)),
            font: input.font.unwrap_or_else(|| "Garamond".into()),
            body_pt: input.body_pt.unwrap_or(11.5),
            line_spacing: input.line_spacing.unwrap_or(1.3),
            justified: input.justified.unwrap_or(true),
            running_header: input.running_header.unwrap_or(true),
        };
        engine::create_novel(&mut doc, &cfg);
        let mut store = self.store.lock().await;
        let handle = store.insert(doc);
        serde_json::json!({"handle": handle}).to_string()
    }

    #[tool(description = "Open an existing .docx file from disk")]
    async fn open_document(&self, Parameters(input): Parameters<OpenInput>) -> String {
        match zavora_docx::Document::open(&input.file_path) {
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
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
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
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
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
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let mut para = doc.insert_paragraph(input.index, "");

            if input.page_break_before.unwrap_or(false) {
                para = para.page_break_before(true);
            }

            // Apply style. Headings/Title/Subtitle use NAMED paragraph styles
            // (via para.style) so the TOC scanner detects them and Word shows
            // them in the Styles gallery + navigation pane. Other cases keep
            // direct formatting since no named style ships for them.
            let style = input.style.as_deref();
            match style {
                Some("Heading1") => {
                    para = para.style("Heading1");
                    para.add_run(&input.text);
                }
                Some("Heading2") => {
                    para = para.style("Heading2");
                    para.add_run(&input.text);
                }
                Some("Heading3") => {
                    para = para.style("Heading3");
                    para.add_run(&input.text);
                }
                Some("TitlePage") => {
                    para = para.style("Title").alignment(zavora_docx::Alignment::Center);
                    para.add_run(&input.text);
                }
                Some("Subtitle") => {
                    para = para.style("Subtitle").alignment(zavora_docx::Alignment::Center);
                    para.add_run(&input.text);
                }
                Some("BodyTextIndent") => {
                    // Inherit Normal (font/size/justify/spacing from the template);
                    // only add the first-line indent that distinguishes continuation paras.
                    para = para.first_line_indent(zavora_docx::Length::inches(0.25));
                    para.add_run(&input.text);
                }
                Some("ChapterNum") => {
                    para = para.alignment(zavora_docx::Alignment::Center)
                        .space_after(zavora_docx::Length::pt(6.0));
                    para.add_run(&input.text).small_caps(true);
                }
                Some("Author") => {
                    para = para.alignment(zavora_docx::Alignment::Center);
                    para.add_run(&input.text);
                }
                Some("Copyright") => {
                    para.add_run(&input.text).size(9.0);
                }
                _ => {
                    // BodyText (default): inherit the document's Normal style so the
                    // template/author config controls font, size, spacing, justification.
                    para.add_run(&input.text);
                }
            }

            serde_json::json!({"index": input.index}).to_string()
        })
    }

    #[tool(description = "Insert a syntax-highlighted code block with gray background (Courier New 9pt). Supports 'rust' highlighting.")]
    async fn insert_code_block(&self, Parameters(input): Parameters<CodeBlockInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let lines = engine::insert_code_block(doc, input.index, &input.code, input.language.as_deref());
            serde_json::json!({"lines_inserted": lines}).to_string()
        })
    }

    #[tool(description = "Insert a callout box with colored background and border (tip=green, warning=orange, note=blue)")]
    async fn insert_callout(&self, Parameters(input): Parameters<CalloutInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            engine::insert_callout(doc, input.index, &input.callout_type, &input.text);
            serde_json::json!({"inserted": true}).to_string()
        })
    }

    #[tool(description = "Insert a table at the given position")]
    async fn add_table(&self, Parameters(input): Parameters<TableInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            doc.insert_table(input.index, input.rows, input.cols);
            serde_json::json!({"index": input.index, "rows": input.rows, "cols": input.cols}).to_string()
        })
    }

    #[tool(description = "Set text in a table cell by table_index (0-based), row, and col.")]
    async fn set_table_cell(&self, Parameters(input): Parameters<CellInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let ti = input.table_index.unwrap_or(0);
            match doc.table_mut(ti) {
                Some(mut table) => match table.cell(input.row, input.col) {
                    Some(mut cell) => { cell.set_text(&input.text); serde_json::json!({"set": true, "table": ti, "row": input.row, "col": input.col}).to_string() }
                    None => serde_json::json!({"error": "CELL_NOT_FOUND"}).to_string(),
                },
                None => serde_json::json!({"error": "TABLE_NOT_FOUND"}).to_string(),
            }
        })
    }

    #[tool(description = "Add an image from file path")]
    async fn add_image(&self, Parameters(input): Parameters<ImageInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let img_data = match std::fs::read(&input.image_path) {
                Ok(d) => d,
                Err(e) => return serde_json::json!({"error": e.to_string()}).to_string(),
            };
            let ext = input.image_path.rsplit('.').next().unwrap_or("png");
            let filename = format!("image.{}", ext);
            let w = zavora_docx::Length::inches(input.width.unwrap_or(4.0));
            let h = zavora_docx::Length::inches(input.height.unwrap_or(3.0));
            let has_feature = input.rotation.is_some() || input.flip_h.is_some()
                || input.flip_v.is_some() || input.crop.is_some()
                || input.border_color.is_some() || input.shadow_color.is_some()
                || input.alt_text.is_some();
            if has_feature {
                let props = zavora_docx::PicProps {
                    rotation: input.rotation.map(|d| (d * 60_000.0) as i32),
                    flip_h: input.flip_h.unwrap_or(false),
                    flip_v: input.flip_v.unwrap_or(false),
                    crop: input.crop.map(|c| [
                        (c[0] * 1000.0) as i32, (c[1] * 1000.0) as i32,
                        (c[2] * 1000.0) as i32, (c[3] * 1000.0) as i32,
                    ]),
                    border: input.border_color.map(|c|
                        (c, (input.border_width.unwrap_or(1.0) * 12700.0) as i64)),
                    shadow: input.shadow_color,
                    title: input.alt_text,
                };
                doc.add_picture_with(&img_data, &filename, w, h, props);
            } else {
                doc.add_picture(&img_data, &filename, w, h);
            }
            serde_json::json!({"added": true}).to_string()
        })
    }

    #[tool(description = "Add a full-bleed page background image anchored to the page at `index` (place a page-break paragraph there first so it covers that page). Text added after flows on top. Use for children's book spreads and full-page interior plates. Returns the anchor index.")]
    async fn add_page_background(&self, Parameters(input): Parameters<PageBackgroundInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let img_data = match std::fs::read(&input.image_path) {
                Ok(d) => d,
                Err(e) => return serde_json::json!({"error": e.to_string()}).to_string(),
            };
            let ext = input.image_path.rsplit('.').next().unwrap_or("png");
            let at = doc.add_page_background_at(input.index, &img_data, &format!("bg.{ext}"));
            serde_json::json!({"anchored_at": at}).to_string()
        })
    }

    #[tool(description = "Set document-level settings in settings.xml: default tab stop (inches), mirror margins, track changes, open zoom percent, proofing language (e.g. en-US), update fields on open, and auto hyphenation. Only provided fields are changed; existing settings are preserved.")]
    async fn set_document_settings(&self, Parameters(input): Parameters<DocumentSettingsInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            if let Some(t) = input.default_tab_stop_inches {
                doc.set_default_tab_stop(zavora_docx::Length::inches(t));
            }
            if let Some(v) = input.mirror_margins {
                doc.set_mirror_margins(v);
            }
            if let Some(v) = input.track_changes {
                doc.set_track_changes(v);
            }
            if let Some(z) = input.zoom_percent {
                doc.set_zoom(z);
            }
            if let Some(ref l) = input.language {
                doc.set_document_language(l);
            }
            if let Some(v) = input.update_fields {
                if v { doc.set_update_fields(); }
            }
            if let Some(v) = input.auto_hyphenation {
                doc.set_auto_hyphenation(v);
            }
            serde_json::json!({"updated": true}).to_string()
        })
    }

    #[tool(description = "Add a content control (structured document tag) to the end of the document: text, rich_text, dropdown, combo, date, or checkbox. Provide a tag to identify it; placeholder is the display text. dropdown/combo take options as [display,value] pairs; date takes date_format (e.g. yyyy-MM-dd); checkbox takes checked.")]
    async fn add_content_control(&self, Parameters(input): Parameters<ContentControlInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let opts = || input.options.clone().unwrap_or_default()
                .into_iter().map(|o| (o[0].clone(), o[1].clone())).collect::<Vec<_>>();
            let kind = match input.kind.as_str() {
                "text" => zavora_docx::SdtKind::Text,
                "rich_text" => zavora_docx::SdtKind::RichText,
                "dropdown" => zavora_docx::SdtKind::DropDown(opts()),
                "combo" => zavora_docx::SdtKind::ComboBox(opts()),
                "date" => zavora_docx::SdtKind::Date(
                    input.date_format.clone().unwrap_or_else(|| "yyyy-MM-dd".to_string())),
                "checkbox" => zavora_docx::SdtKind::Checkbox(input.checked.unwrap_or(false)),
                other => return serde_json::json!({"error": format!("unknown kind: {other}")}).to_string(),
            };
            doc.add_content_control(kind, &input.tag, input.placeholder.as_deref());
            serde_json::json!({"added": true}).to_string()
        })
    }

    #[tool(description = "Add a block-level mathematical equation from a LaTeX-subset string. Supports \\frac{}{}, ^ and _ (superscript/subscript), \\sqrt{} and \\sqrt[n]{}, \\sum/\\int/\\prod with _/^ limits, \\left(...\\right), functions (\\sin, \\cos, \\log, \\ln, \\lim), Greek letters (\\alpha, \\beta, \\pi, ...), and operators (\\cdot, \\times, \\pm, \\leq, \\geq, \\infty, \\to). Renders as a real editable Word equation (OMML).")]
    async fn add_equation(&self, Parameters(input): Parameters<EquationInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            doc.add_equation_latex(&input.latex);
            serde_json::json!({"added": true}).to_string()
        })
    }

    #[tool(description = "Add a rectangular text box (with a border) containing the given lines of text. Width/height default to 3x1.5 inches.")]
    async fn add_text_box(&self, Parameters(input): Parameters<TextBoxInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let w = zavora_docx::Length::inches(input.width_inches.unwrap_or(3.0));
            let h = zavora_docx::Length::inches(input.height_inches.unwrap_or(1.5));
            doc.add_text_box(w, h, input.lines.clone());
            serde_json::json!({"added": true}).to_string()
        })
    }

    #[tool(description = "Add a preset shape (DrawingML). geometry is a preset name: rect, roundRect, ellipse, triangle, diamond, rightArrow, leftArrow, star5, hexagon, etc. Optional solid fill_color (hex). Width/height default to 2x2 inches.")]
    async fn add_shape(&self, Parameters(input): Parameters<ShapeInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let w = zavora_docx::Length::inches(input.width_inches.unwrap_or(2.0));
            let h = zavora_docx::Length::inches(input.height_inches.unwrap_or(2.0));
            doc.add_shape(w, h, &input.geometry, input.fill_color.as_deref());
            serde_json::json!({"added": true}).to_string()
        })
    }

    #[tool(description = "Add a native, editable chart (bar, column, line, pie, or area). Provide categories (x-axis labels) and one or more series, each with a name and a value per category. Data labels are configurable: label_position (outEnd/inEnd/ctr/inBase/bestFit), label_show_value/category/percent, and label_color (hex). Defaults place labels outside for readable contrast. Width/height default to 5x3 inches.")]
    async fn add_chart(&self, Parameters(input): Parameters<ChartInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let kind = match input.kind.as_str() {
                "bar" => zavora_docx::ChartKind::Bar,
                "column" => zavora_docx::ChartKind::Column,
                "line" => zavora_docx::ChartKind::Line,
                "pie" => zavora_docx::ChartKind::Pie,
                "area" => zavora_docx::ChartKind::Area,
                other => return serde_json::json!({"error": format!("unknown chart kind: {other}")}).to_string(),
            };
            let chart = zavora_docx::Chart {
                kind,
                title: input.title.clone(),
                categories: input.categories.clone(),
                series: input.series.iter()
                    .map(|s| zavora_docx::Series { name: s.name.clone(), values: s.values.clone() })
                    .collect(),
                labels: build_labels(&input, kind),
            };
            let w = zavora_docx::Length::inches(input.width_inches.unwrap_or(5.0));
            let h = zavora_docx::Length::inches(input.height_inches.unwrap_or(3.0));
            doc.add_chart(&chart, w, h);
            serde_json::json!({"added": true}).to_string()
        })
    }

    #[tool(description = "Insert a linked Table of Contents at the given position. IMPORTANT: call this AFTER all headings have been added — it scans for paragraphs styled Heading1/2/3 and builds entries with page numbers. Apply heading styles via insert_paragraph with style=Heading1/2/3 (not direct formatting), or the TOC will be empty. Returns headings_found.")]
    async fn add_toc(&self, Parameters(input): Parameters<TocInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let found = doc.insert_toc(input.index, 3);
            serde_json::json!({"index": input.index, "headings_found": found}).to_string()
        })
    }

    #[tool(description = "Insert a scene break (for novels). style: 'asterisks', 'diamond', or 'blank'")]
    async fn insert_scene_break(&self, Parameters(input): Parameters<SceneBreakInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            engine::insert_scene_break(doc, input.index, input.style.as_deref().unwrap_or("asterisks"));
            serde_json::json!({"inserted": true}).to_string()
        })
    }

    #[tool(description = "Insert a best-seller chapter opening: a drop-cap initial letter with the first few words in small caps, then the body text. Use for the FIRST paragraph of each chapter (after the Heading1 chapter title). Pass the whole first paragraph as text.")]
    async fn insert_chapter_opening(&self, Parameters(input): Parameters<ChapterOpeningInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            engine::insert_chapter_opening(doc, input.index, &input.text, input.font.as_deref().unwrap_or("Garamond"));
            serde_json::json!({"inserted": true, "paragraphs": 2}).to_string()
        })
    }

    #[tool(description = "Set header and/or footer text. Use {{PAGE}} for page numbers.")]
    async fn set_header_footer(&self, Parameters(input): Parameters<HeaderFooterInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            if let Some(h) = &input.header { doc.set_header(h); }
            if let Some(f) = &input.footer { doc.set_footer(f); }
            serde_json::json!({"set": true}).to_string()
        })
    }

    #[tool(description = "Export document as plain text")]
    async fn to_plain_text(&self, Parameters(input): Parameters<ExportInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
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
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            serde_json::json!({"markdown": doc.to_markdown()}).to_string()
        })
    }

    #[tool(description = "Export document as HTML fragment.")]
    async fn to_html(&self, Parameters(input): Parameters<ExportInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            serde_json::json!({"html": doc.to_html_fragment()}).to_string()
        })
    }

    #[tool(description = "Export document as PDF. Saves to the given file path.")]
    async fn save_pdf(&self, Parameters(input): Parameters<SavePdfInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match doc.save_pdf(&input.output_path) {
                Ok(_) => serde_json::json!({"saved": input.output_path}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        })
    }

    #[tool(description = "Read paragraphs with pagination. Returns text and index for each paragraph.")]
    async fn read_paragraphs(&self, Parameters(input): Parameters<ReadParasInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
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
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let paras = doc.paragraphs();
            if input.index >= paras.len() { return serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(); }
            let p = &paras[input.index];
            serde_json::json!({"index": input.index, "text": p.text(), "outline_level": p.style_id()}).to_string()
        })
    }

    #[tool(description = "Read table content as structured rows and cells.")]
    async fn read_table(&self, Parameters(input): Parameters<ReadTableInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
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
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let query_lower = input.query.to_lowercase();
            let results: Vec<serde_json::Value> = doc.paragraphs().iter().enumerate().filter(|(_, p)| p.text().to_lowercase().contains(&query_lower)).map(|(i, p)| serde_json::json!({"index": i, "text": p.text()})).collect();
            serde_json::json!({"query": input.query, "matches": results.len(), "results": results}).to_string()
        })
    }

    #[tool(description = "Find and replace text across all paragraphs. Returns count of replacements made.")]
    async fn replace_text(&self, Parameters(input): Parameters<ReplaceInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let count = doc.replace_text(&input.find, &input.replace);
            serde_json::json!({"find": input.find, "replace": input.replace, "replacements": count}).to_string()
        })
    }

    #[tool(description = "Delete content element (paragraph or table) by index.")]
    async fn delete_content(&self, Parameters(input): Parameters<DeleteInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            if doc.remove_content(input.index) {
                serde_json::json!({"deleted": true, "index": input.index}).to_string()
            } else {
                serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string()
            }
        })
    }

    #[tool(description = "Update paragraph text at given index (replaces entire paragraph text).")]
    async fn update_paragraph_text(&self, Parameters(input): Parameters<UpdateParaInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            if input.index >= doc.paragraph_count() { return serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(); }
            doc.remove_content(input.index);
            doc.insert_paragraph(input.index, &input.text);
            serde_json::json!({"updated": true, "index": input.index}).to_string()
        })
    }

    #[tool(description = "Add a formatted text run to a paragraph. Supports bold, italic, underline, font, size, color.")]
    async fn insert_run(&self, Parameters(input): Parameters<RunFormatInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
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

    #[tool(description = "Add a Word field to a paragraph. instruction is the field code, e.g. 'DATE \\\\@ \"yyyy-MM-dd\"', 'REF bookmarkName', 'SEQ Figure \\\\* ARABIC', 'STYLEREF \"Heading 1\"', 'PAGE', 'NUMPAGES'. Optional cached result text. Word recomputes fields on open when update-fields is enabled (see set_document_settings update_fields).")]
    async fn add_field(&self, Parameters(input): Parameters<FieldInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match doc.paragraph_mut(input.paragraph_index) {
                Some(mut para) => {
                    para.add_field(&input.instruction, input.cached.as_deref());
                    serde_json::json!({"added": true}).to_string()
                }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Set paragraph formatting: alignment (left, center, right, justify), spacing, line spacing.")]
    async fn set_paragraph_format(&self, Parameters(input): Parameters<ParaFormatInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match doc.paragraph_mut(input.index) {
                Some(mut para) => {
                    if let Some(align) = &input.alignment {
                        para = match align.as_str() {
                            "center" => para.alignment(zavora_docx::Alignment::Center),
                            "right" => para.alignment(zavora_docx::Alignment::Right),
                            "justify" => para.alignment(zavora_docx::Alignment::Justify),
                            _ => para.alignment(zavora_docx::Alignment::Left),
                        };
                    }
                    if let Some(sb) = input.space_before { para = para.space_before(zavora_docx::Length::pt(sb)); }
                    if let Some(sa) = input.space_after { para = para.space_after(zavora_docx::Length::pt(sa)); }
                    if let Some(ls) = input.line_spacing { para.line_spacing_multiple(ls); }
                    serde_json::json!({"formatted": true, "index": input.index}).to_string()
                }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Create a bulleted or numbered list. list_type: 'bullet' or 'numbered'.")]
    async fn add_list(&self, Parameters(input): Parameters<ListInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
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
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
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

    #[tool(description = "Merge table cells. direction: 'horizontal' (grid_span across columns) or 'vertical' (v_merge across rows). span = number of cells to merge.")]
    async fn merge_table_cells(&self, Parameters(input): Parameters<MergeCellsInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let ti = input.table_index.unwrap_or(0);
            match doc.table_mut(ti) {
                Some(mut table) => {
                    if input.direction == "horizontal" {
                        if let Some(cell) = table.cell(input.start_row, input.start_col) {
                            cell.grid_span(input.span as u32);
                        }
                    } else {
                        // vertical merge: restart on first cell, continue on subsequent
                        for r in 0..input.span {
                            if let Some(cell) = table.cell(input.start_row + r, input.start_col) {
                                if r == 0 { cell.v_merge_restart(); } else { cell.v_merge_continue(); }
                            }
                        }
                    }
                    serde_json::json!({"merged": true, "direction": input.direction, "span": input.span}).to_string()
                }
                None => serde_json::json!({"error": "TABLE_NOT_FOUND"}).to_string(),
            }
        })
    }

    #[tool(description = "Add a row to an existing table with the given cell values.")]
    async fn add_table_row(&self, Parameters(input): Parameters<AddRowInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let ti = input.table_index.unwrap_or(0);
            match doc.table_mut(ti) {
                Some(mut table) => {
                    let row_idx = table.row_count();
                    let cols = input.cells.len();
                    let mut row = table.add_row(cols);
                    for (c, text) in input.cells.iter().enumerate() {
                        if !text.is_empty() {
                            if let Some(mut cell) = row.cell(c) { cell.set_text(text); }
                        }
                    }
                    serde_json::json!({"added": true, "row_index": row_idx}).to_string()
                }
                None => serde_json::json!({"error": "TABLE_NOT_FOUND"}).to_string(),
            }
        })
    }

    #[tool(description = "Set page layout: size, orientation, margins, columns, gutter. All measurements in inches.")]
    async fn set_page_layout(&self, Parameters(input): Parameters<PageLayoutInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            if let (Some(w), Some(h)) = (input.width, input.height) {
                doc.set_page_size(zavora_docx::Length::inches(w), zavora_docx::Length::inches(h));
            }
            match input.orientation.as_deref() {
                Some("landscape") => doc.set_landscape(),
                Some("portrait") => doc.set_portrait(),
                _ => {}
            }
            if let Some(m) = &input.margins {
                if m.len() == 4 {
                    doc.set_margins(
                        zavora_docx::Length::inches(m[0]), zavora_docx::Length::inches(m[1]),
                        zavora_docx::Length::inches(m[2]), zavora_docx::Length::inches(m[3]),
                    );
                }
            }
            if let Some(cols) = input.columns {
                doc.set_columns(cols, zavora_docx::Length::inches(0.5));
            }
            if let Some(g) = input.gutter {
                doc.set_gutter(zavora_docx::Length::inches(g));
            }
            serde_json::json!({"set": true}).to_string()
        })
    }

    #[tool(description = "Set document metadata: title, author, subject, keywords.")]
    async fn set_metadata(&self, Parameters(input): Parameters<SetMetadataInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            if let Some(t) = &input.title { doc.set_title(t); }
            if let Some(a) = &input.author { doc.set_author(a); }
            if let Some(s) = &input.subject { doc.set_subject(s); }
            if let Some(k) = &input.keywords { doc.set_keywords(k); }
            if let Some(c) = &input.company { doc.set_company(c); }
            if let Some(a) = &input.application { doc.set_application(a); }
            serde_json::json!({"set": true}).to_string()
        })
    }

    #[tool(description = "Get document metadata: title, author, subject, keywords, word count.")]
    async fn get_metadata(&self, Parameters(input): Parameters<HandleInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            serde_json::json!({
                "title": doc.title(),
                "author": doc.author(),
                "subject": doc.subject(),
                "keywords": doc.keywords(),
                "word_count": doc.word_count(),
            }).to_string()
        })
    }

    #[tool(description = "Merge another document into this one. Appends all content with an optional section break between them.")]
    async fn merge_documents(&self, Parameters(input): Parameters<MergeDocumentsInput>) -> String {
        let other = match zavora_docx::Document::open(&input.other_path) {
            Ok(d) => d,
            Err(e) => return serde_json::json!({"error": e.to_string()}).to_string(),
        };
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match input.break_type.as_deref() {
                Some(bt) => {
                    let brk = match bt {
                        "continuous" => zavora_docx::SectionBreak::Continuous,
                        "evenPage" => zavora_docx::SectionBreak::EvenPage,
                        "oddPage" => zavora_docx::SectionBreak::OddPage,
                        _ => zavora_docx::SectionBreak::NextPage,
                    };
                    doc.append_with_break(&other, brk);
                }
                None => doc.append(&other),
            }
            serde_json::json!({"merged": true}).to_string()
        })
    }

    #[tool(description = "Render a document page as PNG image. Returns the output file path.")]
    async fn render_page(&self, Parameters(input): Parameters<RenderPageInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let dpi = input.dpi.unwrap_or(150.0);
            match doc.render_page_to_png(input.page_index, dpi) {
                Ok(Some(png_data)) => {
                    match std::fs::write(&input.output_path, &png_data) {
                        Ok(_) => serde_json::json!({"rendered": input.output_path, "bytes": png_data.len()}).to_string(),
                        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
                    }
                }
                Ok(None) => serde_json::json!({"error": "PAGE_NOT_FOUND"}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        })
    }

    #[tool(description = "Get document outline (heading structure). Returns nested heading tree with levels and text.")]
    async fn document_outline(&self, Parameters(input): Parameters<HandleInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let outline = doc.document_outline();
            let items: Vec<serde_json::Value> = outline.iter().map(|node| {
                serde_json::json!({"level": node.level, "text": node.text})
            }).collect();
            serde_json::json!({"outline": items}).to_string()
        })
    }

    #[tool(description = "Find and replace using regex pattern. Supports capture groups ($1, $2) in replacement.")]
    async fn replace_regex(&self, Parameters(input): Parameters<RegexReplaceInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match doc.replace_regex(&input.pattern, &input.replacement) {
                Ok(count) => serde_json::json!({"replacements": count}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        })
    }

    #[tool(description = "Format a table: width, alignment, borders, cell margins.")]
    async fn format_table(&self, Parameters(input): Parameters<FormatTableInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let ti = input.table_index.unwrap_or(0);
            match doc.table_mut(ti) {
                Some(mut table) => {
                    if let Some(pct) = input.width_pct { table = table.width_pct(pct); }
                    if let Some(align) = &input.alignment {
                        let a = match align.as_str() {
                            "center" => zavora_docx::Alignment::Center,
                            "right" => zavora_docx::Alignment::Right,
                            _ => zavora_docx::Alignment::Left,
                        };
                        table = table.alignment(a);
                    }
                    if let Some(style) = &input.border_style {
                        let bs = match style.as_str() {
                            "double" => zavora_docx::BorderStyle::Double,
                            "dashed" => zavora_docx::BorderStyle::Dashed,
                            "dotted" => zavora_docx::BorderStyle::Dotted,
                            "none" => zavora_docx::BorderStyle::None,
                            _ => zavora_docx::BorderStyle::Single,
                        };
                        let color = input.border_color.as_deref().unwrap_or("000000");
                        let size = input.border_size.unwrap_or(4);
                        table = table.borders(bs, size, color);
                    }
                    if let Some(m) = &input.cell_margins {
                        if m.len() == 4 {
                            table.cell_margins(
                                zavora_docx::Length::pt(m[0]), zavora_docx::Length::pt(m[1]),
                                zavora_docx::Length::pt(m[2]), zavora_docx::Length::pt(m[3]),
                            );
                        }
                    }
                    serde_json::json!({"formatted": true}).to_string()
                }
                None => serde_json::json!({"error": "TABLE_NOT_FOUND"}).to_string(),
            }
        })
    }

    #[tool(description = "Format a table cell: shading, vertical alignment, width, no-wrap.")]
    async fn format_table_cell(&self, Parameters(input): Parameters<FormatCellInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let ti = input.table_index.unwrap_or(0);
            match doc.table_mut(ti) {
                Some(mut table) => match table.cell(input.row, input.col) {
                    Some(mut cell) => {
                        if let Some(s) = &input.shading { cell = cell.shading(s); }
                        if let Some(va) = &input.vertical_alignment {
                            let align = match va.as_str() {
                                "center" => zavora_docx::VerticalAlignment::Center,
                                "bottom" => zavora_docx::VerticalAlignment::Bottom,
                                _ => zavora_docx::VerticalAlignment::Top,
                            };
                            cell = cell.vertical_alignment(align);
                        }
                        if let Some(w) = input.width { cell = cell.width(zavora_docx::Length::inches(w)); }
                        if input.no_wrap.unwrap_or(false) { cell.no_wrap(); }
                        serde_json::json!({"formatted": true}).to_string()
                    }
                    None => serde_json::json!({"error": "CELL_NOT_FOUND"}).to_string(),
                },
                None => serde_json::json!({"error": "TABLE_NOT_FOUND"}).to_string(),
            }
        })
    }

    #[tool(description = "Insert a section break at the given index. Allows changing page layout for subsequent content.")]
    async fn add_section_break(&self, Parameters(input): Parameters<SectionBreakInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let bt = match input.break_type.as_deref() {
                Some("continuous") => zavora_docx::SectionBreak::Continuous,
                Some("evenPage") => zavora_docx::SectionBreak::EvenPage,
                Some("oddPage") => zavora_docx::SectionBreak::OddPage,
                _ => zavora_docx::SectionBreak::NextPage,
            };
            let mut para = doc.insert_paragraph(input.index, "");
            para = para.section_break(bt);
            if let Some(o) = &input.orientation {
                match o.as_str() {
                    "landscape" => { para = para.section_landscape(); }
                    "portrait" => { para = para.section_portrait(); }
                    _ => {}
                }
            }
            if let (Some(w), Some(h)) = (input.page_width, input.page_height) {
                para.section_page_size(zavora_docx::Length::inches(w), zavora_docx::Length::inches(h));
            }
            serde_json::json!({"inserted": true, "index": input.index}).to_string()
        })
    }

    #[tool(description = "Audit document for accessibility issues: missing alt text, empty headings, skipped heading levels, low contrast.")]
    async fn audit_accessibility(&self, Parameters(input): Parameters<HandleInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let issues = doc.audit_accessibility();
            let items: Vec<serde_json::Value> = issues.iter().map(|issue| {
                serde_json::json!({
                    "severity": format!("{:?}", issue.severity),
                    "message": issue.message,
                })
            }).collect();
            serde_json::json!({"issues": items.len(), "details": items}).to_string()
        })
    }

    // ── New Features: Footnotes, Hyperlinks, Bookmarks, Comments, Watermarks, Track Changes, Form Fields, Protection ──

    #[tool(description = "Add a footnote. Returns the footnote ID to use with add_footnote_ref.")]
    async fn add_footnote(&self, Parameters(input): Parameters<FootnoteInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let id = doc.add_footnote(&input.text);
            serde_json::json!({"footnote_id": id}).to_string()
        })
    }

    #[tool(description = "Add a footnote reference (superscript number) to a paragraph. Use the ID from add_footnote.")]
    async fn add_footnote_ref(&self, Parameters(input): Parameters<FootnoteRefInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match doc.paragraph_mut(input.paragraph_index) {
                Some(mut para) => {
                    para.add_run("").footnote_ref(input.footnote_id);
                    serde_json::json!({"added": true}).to_string()
                }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Add a clickable hyperlink to a paragraph. Creates blue underlined text linking to the URL.")]
    async fn add_hyperlink(&self, Parameters(input): Parameters<HyperlinkInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let rel_id = doc.add_hyperlink_rel(&input.url);
            match doc.paragraph_mut(input.paragraph_index) {
                Some(mut para) => {
                    para.add_hyperlink_run(&input.text, Some(&rel_id), None)
                        .color("0563C1").underline(true);
                    serde_json::json!({"added": true, "url": input.url}).to_string()
                }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Add a bookmark to a paragraph for cross-referencing.")]
    async fn add_bookmark(&self, Parameters(input): Parameters<BookmarkInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match doc.paragraph_mut(input.paragraph_index) {
                Some(mut para) => {
                    para.bookmark(input.id, &input.name);
                    serde_json::json!({"added": true, "name": input.name}).to_string()
                }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Add a comment to the document. Use add_comment_range to mark which text the comment applies to.")]
    async fn add_comment(&self, Parameters(input): Parameters<CommentInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            doc.add_comment(input.id, &input.author, &input.text);
            serde_json::json!({"added": true, "comment_id": input.id}).to_string()
        })
    }

    #[tool(description = "Mark text in a paragraph as commented. First call add_comment to create the comment, then use this to mark the range.")]
    async fn add_comment_range(&self, Parameters(input): Parameters<CommentRangeInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let ok = match doc.paragraph_mut(input.paragraph_index) {
                Some(mut para) => {
                    para.comment_start(input.comment_id);
                    para.add_run(&input.commented_text);
                    para.comment_end(input.comment_id);
                    true
                }
                None => false,
            };
            if ok {
                doc.set_comment_anchor(input.comment_id, input.paragraph_index);
                serde_json::json!({"added": true}).to_string()
            } else {
                serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string()
            }
        })
    }

    #[tool(description = "Add a threaded reply to an existing comment. Provide a new unique id, the parent_id of the comment being replied to, author, and text.")]
    async fn reply_to_comment(&self, Parameters(input): Parameters<CommentReplyInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            doc.add_comment_reply(input.id, input.parent_id, &input.author, &input.text);
            serde_json::json!({"added": true, "comment_id": input.id}).to_string()
        })
    }

    #[tool(description = "Mark a comment (by id) as resolved/done.")]
    async fn resolve_comment(&self, Parameters(input): Parameters<ResolveCommentInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            doc.resolve_comment(input.id);
            serde_json::json!({"resolved": true, "comment_id": input.id}).to_string()
        })
    }

    #[tool(description = "Set a diagonal text watermark on every page (e.g. 'DRAFT', 'CONFIDENTIAL').")]
    async fn set_watermark(&self, Parameters(input): Parameters<WatermarkInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let color = input.color.as_deref().unwrap_or("C0C0C0");
            doc.set_text_watermark(&input.text, color, input.rotation);
            serde_json::json!({"set": true, "text": input.text}).to_string()
        })
    }

    #[tool(description = "Add tracked insertion (text shown as added in review mode).")]
    async fn add_tracked_insert(&self, Parameters(input): Parameters<TrackedInsertInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match doc.paragraph_mut(input.paragraph_index) {
                Some(mut para) => {
                    para.add_tracked_insert(&input.text, &input.author);
                    serde_json::json!({"added": true}).to_string()
                }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Add tracked deletion (text shown as strikethrough in review mode).")]
    async fn add_tracked_delete(&self, Parameters(input): Parameters<TrackedDeleteInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match doc.paragraph_mut(input.paragraph_index) {
                Some(mut para) => {
                    para.add_tracked_delete(&input.text, &input.author);
                    serde_json::json!({"added": true}).to_string()
                }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Add a text input form field to a paragraph.")]
    async fn add_form_text_field(&self, Parameters(input): Parameters<TextFieldInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match doc.paragraph_mut(input.paragraph_index) {
                Some(mut para) => {
                    if let Some(label) = &input.label { para.add_run(label); }
                    para.add_text_field(&input.name, input.default_value.as_deref().unwrap_or(""));
                    serde_json::json!({"added": true}).to_string()
                }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Add a checkbox form field to a paragraph.")]
    async fn add_form_checkbox(&self, Parameters(input): Parameters<CheckboxInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match doc.paragraph_mut(input.paragraph_index) {
                Some(mut para) => {
                    if let Some(label) = &input.label { para.add_run(label); }
                    para.add_checkbox(&input.name, input.checked.unwrap_or(false));
                    serde_json::json!({"added": true}).to_string()
                }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Add a dropdown form field to a paragraph.")]
    async fn add_form_dropdown(&self, Parameters(input): Parameters<DropdownInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match doc.paragraph_mut(input.paragraph_index) {
                Some(mut para) => {
                    if let Some(label) = &input.label { para.add_run(label); }
                    let opts: Vec<&str> = input.options.iter().map(|s| s.as_str()).collect();
                    para.add_dropdown(&input.name, &opts, input.selected.unwrap_or(0));
                    serde_json::json!({"added": true}).to_string()
                }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Protect the document. Types: 'readonly', 'forms' (only form fields editable), 'comments' (only comments allowed), 'trackedChanges'.")]
    async fn protect_document(&self, Parameters(input): Parameters<ProtectInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match input.protection_type.as_str() {
                "readonly" => doc.protect_readonly(),
                "forms" => doc.protect_forms_only(),
                "comments" => doc.protect_comments_only(),
                "trackedChanges" => doc.protect_tracked_changes_only(),
                _ => return serde_json::json!({"error": "Invalid type. Use: readonly, forms, comments, trackedChanges"}).to_string(),
            }
            serde_json::json!({"protected": true, "type": input.protection_type}).to_string()
        })
    }

    #[tool(description = "Remove document protection.")]
    async fn unprotect_document(&self, Parameters(input): Parameters<HandleInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            doc.unprotect();
            serde_json::json!({"unprotected": true}).to_string()
        })
    }

    #[tool(description = "Apply a drop cap to a paragraph (large first letter spanning multiple lines, for chapter openers).")]
    async fn set_drop_cap(&self, Parameters(input): Parameters<DropCapInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match doc.paragraph_mut(input.paragraph_index) {
                Some(para) => { para.drop_cap(input.lines); serde_json::json!({"set": true}).to_string() }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Add text with a w14 effect (shadow, glow, outline, or reflection). These are Word 2010+ effects.")]
    async fn add_text_effect(&self, Parameters(input): Parameters<TextEffectInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match doc.paragraph_mut(input.paragraph_index) {
                Some(mut para) => {
                    let color = input.color.as_deref().unwrap_or("000000");
                    let size = input.size.unwrap_or(3.0);
                    let run = para.add_run(&input.text).font("Calibri").size(18.0).bold(true);
                    match input.effect.as_str() {
                        "shadow" => { run.shadow(size, size * 0.7, color); }
                        "glow" => { run.glow(size, color); }
                        "outline" => { run.text_outline(size * 0.3, color); }
                        "reflection" => { run.reflection(); }
                        _ => {}
                    }
                    serde_json::json!({"added": true, "effect": input.effect}).to_string()
                }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Add a run with a theme color (accent1-6, dk1, dk2, lt1, lt2, hlink). Colors auto-update when theme changes.")]
    async fn add_theme_colored_text(&self, Parameters(input): Parameters<ThemeColorRunInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            match doc.paragraph_mut(input.paragraph_index) {
                Some(mut para) => {
                    para.add_run(&input.text).theme_color(&input.theme_color);
                    serde_json::json!({"added": true}).to_string()
                }
                None => serde_json::json!({"error": "INDEX_OUT_OF_BOUNDS"}).to_string(),
            }
        })
    }

    #[tool(description = "Apply banded rows and header styling to a table. Makes tables look professional with alternating colors.")]
    async fn style_table_banded(&self, Parameters(input): Parameters<BandedTableInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let ti = input.table_index.unwrap_or(0);
            match doc.table_mut(ti) {
                Some(mut table) => {
                    table.banded_rows(&input.band_color);
                    if let (Some(bg), Some(txt)) = (&input.header_bg, &input.header_text) {
                        table.header_row_style(bg, txt);
                    }
                    serde_json::json!({"styled": true}).to_string()
                }
                None => serde_json::json!({"error": "TABLE_NOT_FOUND"}).to_string(),
            }
        })
    }

    #[tool(description = "Enable line numbering in the document margin (for legal/academic documents).")]
    async fn set_line_numbering(&self, Parameters(input): Parameters<LineNumberingInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            doc.set_line_numbering(input.count_by.unwrap_or(1), input.restart.as_deref().unwrap_or("continuous"));
            serde_json::json!({"set": true}).to_string()
        })
    }

    #[tool(description = "Create a list with custom numbering: Roman numerals, letters, or custom bullet characters (★, →, ◆).")]
    async fn add_custom_list(&self, Parameters(input): Parameters<CustomListInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            for item in &input.items {
                doc.add_custom_list_item(item, 0, &input.format, input.bullet_char.as_deref());
            }
            serde_json::json!({"added": input.items.len(), "format": input.format}).to_string()
        })
    }

    #[tool(description = "Set the document theme (fonts and colors). Controls the look of all theme-colored text.")]
    async fn set_theme(&self, Parameters(input): Parameters<ThemeInput>) -> String {
        with_doc!(self.store, input.document_handle, |doc: &mut zavora_docx::Document| {
            let defaults = ["4472C4", "ED7D31", "A5A5A5", "FFC000", "5B9BD5", "70AD47"];
            let accents = input.accent_colors.as_ref();
            let a = |i: usize| accents.and_then(|v| v.get(i).map(|s| s.as_str())).unwrap_or(defaults[i]);
            doc.set_theme(
                &[("dk1", "000000"), ("lt1", "FFFFFF"), ("dk2", "44546A"), ("lt2", "E7E6E6"),
                  ("accent1", a(0)), ("accent2", a(1)), ("accent3", a(2)),
                  ("accent4", a(3)), ("accent5", a(4)), ("accent6", a(5)),
                  ("hlink", "0563C1"), ("folHlink", "954F72")],
                &input.major_font, &input.minor_font,
            );
            serde_json::json!({"set": true, "major_font": input.major_font, "minor_font": input.minor_font}).to_string()
        })
    }
}
