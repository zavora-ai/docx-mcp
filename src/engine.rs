//! Core document engine using rdocx.

use zavora_docx::{Document, Length, BorderStyle, Alignment};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// In-memory document store.
pub struct Store {
    docs: HashMap<String, Document>,
}

impl Store {
    pub fn new() -> Self {
        Self { docs: HashMap::new() }
    }

    pub fn insert(&mut self, doc: Document) -> String {
        let handle = Uuid::new_v4().to_string();
        self.docs.insert(handle.clone(), doc);
        handle
    }

    pub fn get_mut(&mut self, handle: &str) -> Option<&mut Document> {
        self.docs.get_mut(handle)
    }

    pub fn remove(&mut self, handle: &str) -> bool {
        self.docs.remove(handle).is_some()
    }
}

pub type SharedStore = Arc<Mutex<Store>>;

pub fn new_store() -> SharedStore {
    Arc::new(Mutex::new(Store::new()))
}

// ── KDP Templates ────────────────────────────────────────────────────────────

/// Create a KDP-formatted technical book document (6×9, Garamond, proper styles).
pub fn create_kdp_technical(doc: &mut Document) {
    doc.set_page_size(Length::inches(6.0), Length::inches(9.0));
    doc.set_margins(Length::inches(0.75), Length::inches(0.75), Length::inches(0.75), Length::inches(0.875));
    doc.set_footer_page_number();
    doc.set_different_first_page(true);
    doc.set_first_page_footer("");
    doc.set_title("Untitled Technical Book");
    doc.set_gutter(Length::inches(0.125));
    doc.set_theme(
        &[("dk1","000000"),("lt1","FFFFFF"),("dk2","2E4057"),("lt2","F5F5F5"),
          ("accent1","2980B9"),("accent2","E74C3C"),("accent3","27AE60"),
          ("accent4","F39C12"),("accent5","8E44AD"),("accent6","16A085"),
          ("hlink","2980B9"),("folHlink","8E44AD")],
        "Garamond", "Garamond",
    );
}

/// Per-book novel settings so different authors can produce different novels
/// at different trim sizes from the same engine.
pub struct NovelConfig {
    pub title: String,
    pub author: String,
    /// Trim size in inches (width, height). Common KDP: 5x8, 5.25x8, 5.5x8.5, 6x9.
    pub trim: (f64, f64),
    /// Body + heading font family.
    pub font: String,
    /// Body text size in points.
    pub body_pt: f64,
    /// Line spacing multiple for body text.
    pub line_spacing: f64,
    /// Justify body text (true for most fiction).
    pub justified: bool,
    /// Running header text (author on verso, title on recto). None = no header.
    pub running_header: bool,
}

impl Default for NovelConfig {
    fn default() -> Self {
        Self {
            title: "Untitled Novel".into(),
            author: "Anonymous".into(),
            trim: (5.25, 8.0),
            font: "Garamond".into(),
            body_pt: 11.5,
            line_spacing: 1.3,
            justified: true,
            running_header: true,
        }
    }
}

/// Build a professionally-styled novel. Sets page geometry, theme, and overrides
/// the Normal + Heading1 named styles so chapters, body text, and the TOC all
/// inherit book-quality formatting (justified body, widow control, chapter
/// openers). Authors call this with their own NovelConfig.
pub fn create_novel(doc: &mut Document, cfg: &NovelConfig) {
    use zavora_docx::{Length, StyleBuilder};

    let (w, h) = cfg.trim;
    doc.set_page_size(Length::inches(w), Length::inches(h));
    // Generous inner (binding) margin scales a little with page height.
    doc.set_margins(Length::inches(0.75), Length::inches(0.625), Length::inches(0.75), Length::inches(0.875));
    doc.set_gutter(Length::inches(0.125));
    doc.set_footer_page_number();
    doc.set_different_first_page(true);
    doc.set_first_page_footer("");
    doc.set_title(&cfg.title);
    if cfg.running_header {
        doc.set_running_header(&cfg.author, &cfg.title);
    }
    // Hyphenation tightens justified prose by breaking long words.
    if cfg.justified {
        doc.set_auto_hyphenation(true);
    }
    doc.set_theme(
        &[("dk1","1A1A1A"),("lt1","FFFFF8"),("dk2","333333"),("lt2","F8F4E8"),
          ("accent1","8B4513"),("accent2","2F4F4F"),("accent3","800020"),
          ("accent4","4A4A4A"),("accent5","6B4423"),("accent6","2E4A3E"),
          ("hlink","8B4513"),("folHlink","800020")],
        &cfg.font, &cfg.font,
    );

    // Body text: justified, widow control, author's font/size/spacing.
    doc.add_style(
        StyleBuilder::paragraph("Normal", "Normal")
            .font(&cfg.font)
            .size(cfg.body_pt)
            .line_spacing(cfg.line_spacing)
            .align(if cfg.justified { "both" } else { "left" })
            .widow_control(true),
    );

    // Chapter opener (Heading1): centered, dropped down the page, dark small-caps.
    // outline_level(0) keeps it detectable by the TOC + navigation pane.
    doc.add_style(
        StyleBuilder::paragraph("Heading1", "heading 1")
            .based_on("Normal")
            .next_style("Normal")
            .font(&cfg.font)
            .size((cfg.body_pt + 8.0).max(18.0))
            .bold(true)
            .small_caps(true)
            .color("1A1A1A")
            .align("center")
            .spacing(Length::pt(72.0), Length::pt(24.0))
            .keep_with_next(true)
            .outline_level(0),
    );

    // Subhead within a chapter (Heading2): centered, smaller, italic.
    doc.add_style(
        StyleBuilder::paragraph("Heading2", "heading 2")
            .based_on("Normal")
            .next_style("Normal")
            .font(&cfg.font)
            .size(cfg.body_pt + 1.0)
            .italic(true)
            .color("333333")
            .align("center")
            .spacing(Length::pt(18.0), Length::pt(6.0))
            .keep_with_next(true)
            .outline_level(1),
    );
}

/// Default novel (back-compat for the "kdp:novel" format dispatch).
pub fn create_kdp_novel(doc: &mut Document) {
    create_novel(doc, &NovelConfig::default());
}

/// Per-genre body/heading styling applied on top of a template's theme + geometry.
struct BookStyle<'a> {
    body_font: &'a str,
    heading_font: &'a str,
    body_pt: f64,
    line_spacing: f64,
    justified: bool,
    /// Heading color hex (usually the theme's accent1 so headings match the genre).
    heading_color: &'a str,
}

/// Apply professional Normal + Heading1-3 styles for non-fiction/illustrated
/// genres: readable justified (or left) body with widow control, and clean
/// left-aligned headings in the genre's display font and accent color. Enables
/// hyphenation when justified. Call AFTER set_theme so fonts/colors are set.
fn apply_book_styles(doc: &mut Document, s: &BookStyle) {
    use zavora_docx::{Length, StyleBuilder};

    if s.justified {
        doc.set_auto_hyphenation(true);
    }
    doc.add_style(
        StyleBuilder::paragraph("Normal", "Normal")
            .font(s.body_font)
            .size(s.body_pt)
            .line_spacing(s.line_spacing)
            .align(if s.justified { "both" } else { "left" })
            .widow_control(true),
    );
    let heads = [(1u32, 1.7, 0u32), (2, 1.35, 1), (3, 1.15, 2)];
    for (n, scale, lvl) in heads {
        doc.add_style(
            StyleBuilder::paragraph(&format!("Heading{n}"), &format!("heading {n}"))
                .based_on("Normal")
                .next_style("Normal")
                .font(s.heading_font)
                .size((s.body_pt * scale).round())
                .bold(true)
                .color(s.heading_color)
                .align("left")
                .spacing(Length::pt(if n == 1 { 20.0 } else { 14.0 }), Length::pt(6.0))
                .keep_with_next(true)
                .outline_level(lvl),
        );
    }
}

pub fn create_kdp_cookbook(doc: &mut Document) {
    doc.set_page_size(Length::inches(8.0), Length::inches(10.0));
    doc.set_margins(Length::inches(0.75), Length::inches(0.75), Length::inches(0.75), Length::inches(1.0));
    doc.set_footer_page_number();
    doc.set_different_first_page(true);
    doc.set_first_page_footer("");
    doc.set_title("Untitled Cookbook");
    doc.set_theme(
        &[("dk1","2C2C2C"),("lt1","FFFFFF"),("dk2","4A3728"),("lt2","FFF8F0"),
          ("accent1","D4A574"),("accent2","C0392B"),("accent3","27AE60"),
          ("accent4","F4D03F"),("accent5","E67E22"),("accent6","6C3483"),
          ("hlink","C0392B"),("folHlink","6C3483")],
        "Gill Sans MT", "Georgia",
    );
    apply_book_styles(doc, &BookStyle {
        body_font: "Georgia", heading_font: "Gill Sans MT",
        body_pt: 11.0, line_spacing: 1.25, justified: true, heading_color: "C0392B",
    });
}

pub fn create_kdp_children(doc: &mut Document) {
    doc.set_page_size(Length::inches(8.5), Length::inches(8.5));
    doc.set_margins(Length::inches(0.5), Length::inches(0.5), Length::inches(0.5), Length::inches(0.5));
    doc.set_footer_page_number();
    doc.set_different_first_page(true);
    doc.set_first_page_footer("");
    doc.set_title("Untitled Children's Book");
    doc.set_theme(
        &[("dk1","2C3E50"),("lt1","FFFFFF"),("dk2","34495E"),("lt2","FDFEFE"),
          ("accent1","3498DB"),("accent2","E74C3C"),("accent3","2ECC71"),
          ("accent4","F1C40F"),("accent5","9B59B6"),("accent6","E67E22"),
          ("hlink","3498DB"),("folHlink","9B59B6")],
        "Century Schoolbook", "Century Schoolbook",
    );
    // Children: large, well-spaced, left-aligned (never justified) for early readers.
    apply_book_styles(doc, &BookStyle {
        body_font: "Century Schoolbook", heading_font: "Century Schoolbook",
        body_pt: 16.0, line_spacing: 1.5, justified: false, heading_color: "E74C3C",
    });
}

pub fn create_kdp_interior_design(doc: &mut Document) {
    doc.set_page_size(Length::inches(8.5), Length::inches(11.0));
    doc.set_margins(Length::inches(0.75), Length::inches(0.75), Length::inches(0.75), Length::inches(0.875));
    doc.set_footer_page_number();
    doc.set_different_first_page(true);
    doc.set_first_page_footer("");
    doc.set_title("Untitled Interior Design Book");
    doc.set_theme(
        &[("dk1","1C1C1C"),("lt1","FFFFFF"),("dk2","3D3D3D"),("lt2","F7F7F7"),
          ("accent1","B8860B"),("accent2","2F4F4F"),("accent3","8B7355"),
          ("accent4","CD853F"),("accent5","556B2F"),("accent6","4682B4"),
          ("hlink","B8860B"),("folHlink","2F4F4F")],
        "Futura", "Minion Pro",
    );
    apply_book_styles(doc, &BookStyle {
        body_font: "Minion Pro", heading_font: "Futura",
        body_pt: 11.0, line_spacing: 1.3, justified: true, heading_color: "B8860B",
    });
}

pub fn create_kdp_encyclopedia(doc: &mut Document) {
    doc.set_page_size(Length::inches(8.5), Length::inches(11.0));
    doc.set_margins(Length::inches(0.625), Length::inches(0.75), Length::inches(0.625), Length::inches(1.0));
    doc.set_columns(2, Length::inches(0.25));
    doc.set_footer_page_number();
    doc.set_different_first_page(true);
    doc.set_first_page_footer("");
    doc.set_title("Untitled Encyclopedia");
    doc.set_theme(
        &[("dk1","000000"),("lt1","FFFFFF"),("dk2","1B2631"),("lt2","EAECEE"),
          ("accent1","1A5276"),("accent2","922B21"),("accent3","196F3D"),
          ("accent4","7D6608"),("accent5","6C3483"),("accent6","1B4F72"),
          ("hlink","1A5276"),("folHlink","6C3483")],
        "Myriad Pro", "Minion Pro",
    );
    // Two-column reference: justified + hyphenation are essential to avoid rivers.
    apply_book_styles(doc, &BookStyle {
        body_font: "Minion Pro", heading_font: "Myriad Pro",
        body_pt: 10.0, line_spacing: 1.15, justified: true, heading_color: "1A5276",
    });
}

pub fn create_kdp_manga(doc: &mut Document) {
    doc.set_page_size(Length::inches(5.0), Length::inches(7.5));
    doc.set_margins(Length::inches(0.25), Length::inches(0.25), Length::inches(0.25), Length::inches(0.25));
    doc.set_different_first_page(true);
    doc.set_title("Untitled Manga");
}

// ── Paragraph helpers ────────────────────────────────────────────────────────

/// Syntax highlighting colors for common tokens.
struct SyntaxColors;
impl SyntaxColors {
    const KEYWORD: &str = "0000FF";    // blue
    const STRING: &str = "A31515";     // dark red
    const COMMENT: &str = "008000";    // green
    const FUNCTION: &str = "795E26";   // brown
    const TYPE: &str = "267F99";       // teal
    const NUMBER: &str = "098658";     // dark green
    const MACRO: &str = "AF00DB";      // purple
}

/// Simple token types for syntax highlighting.
enum TokenKind { Keyword, String, Comment, Function, Type, Number, Macro, Plain }

/// Tokenize a line of Rust code (simple heuristic-based).
fn tokenize_rust(line: &str) -> Vec<(TokenKind, String)> {
    let mut tokens = Vec::new();
    let keywords = ["use", "fn", "let", "mut", "pub", "struct", "impl", "async", "await",
        "match", "if", "else", "for", "in", "return", "self", "Self", "crate", "mod",
        "true", "false", "Ok", "Err", "Some", "None", "where", "trait", "enum", "type"];

    if line.trim_start().starts_with("//") {
        tokens.push((TokenKind::Comment, line.to_string()));
        return tokens;
    }

    let mut chars = line.chars().peekable();
    let mut current = String::new();

    while let Some(&ch) = chars.peek() {
        if ch == '"' {
            if !current.is_empty() {
                tokens.push((classify_word(&current, &keywords), current.clone()));
                current.clear();
            }
            let mut s = String::new();
            s.push(chars.next().unwrap());
            while let Some(&c) = chars.peek() {
                s.push(chars.next().unwrap());
                if c == '"' && !s.ends_with("\\\"") { break; }
            }
            tokens.push((TokenKind::String, s));
        } else if ch == '#' {
            if !current.is_empty() {
                tokens.push((classify_word(&current, &keywords), current.clone()));
                current.clear();
            }
            let mut s = String::new();
            while let Some(&c) = chars.peek() {
                if c == ']' { s.push(chars.next().unwrap()); break; }
                s.push(chars.next().unwrap());
            }
            tokens.push((TokenKind::Macro, s));
        } else if ch.is_alphanumeric() || ch == '_' {
            current.push(chars.next().unwrap());
        } else {
            if !current.is_empty() {
                tokens.push((classify_word(&current, &keywords), current.clone()));
                current.clear();
            }
            tokens.push((TokenKind::Plain, chars.next().unwrap().to_string()));
        }
    }
    if !current.is_empty() {
        tokens.push((classify_word(&current, &keywords), current));
    }
    tokens
}

fn classify_word(word: &str, keywords: &[&str]) -> TokenKind {
    if keywords.contains(&word) { TokenKind::Keyword }
    else if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) { TokenKind::Type }
    else if word.chars().all(|c| c.is_ascii_digit() || c == '.') { TokenKind::Number }
    else { TokenKind::Plain }
}

fn token_color(kind: &TokenKind) -> Option<&'static str> {
    match kind {
        TokenKind::Keyword => Some(SyntaxColors::KEYWORD),
        TokenKind::String => Some(SyntaxColors::STRING),
        TokenKind::Comment => Some(SyntaxColors::COMMENT),
        TokenKind::Function => Some(SyntaxColors::FUNCTION),
        TokenKind::Type => Some(SyntaxColors::TYPE),
        TokenKind::Number => Some(SyntaxColors::NUMBER),
        TokenKind::Macro => Some(SyntaxColors::MACRO),
        TokenKind::Plain => None,
    }
}

/// Insert a syntax-highlighted code block with gray background.
pub fn insert_code_block(doc: &mut Document, index: usize, code: &str, _language: Option<&str>) -> usize {
    let lines: Vec<&str> = code.lines().collect();
    let count = lines.len();

    for (i, line) in lines.iter().enumerate() {
        let mut para = doc.insert_paragraph(index + i, "");
        para = para.shading("F5F5F5")
            .indent_left(Length::inches(0.2))
            .indent_right(Length::inches(0.2))
            .line_spacing_multiple(1.0)
            .keep_together(true);

        if i == 0 {
            para = para.space_before(Length::pt(8.0));
        }
        if i == count - 1 {
            para = para.space_after(Length::pt(8.0));
        }

        // Single run per line — clean monospace rendering
        para.add_run(if line.is_empty() { " " } else { line })
            .font("Courier New").size(9.0);
    }
    count
}

/// Insert a callout box with colored left border.
pub fn insert_callout(doc: &mut Document, index: usize, callout_type: &str, text: &str) {
    let (prefix, border_color, bg_color) = match callout_type {
        "warning" => ("⚠ WARNING: ", "ED7D31", "FFF2CC"),
        "note" => ("📝 NOTE: ", "4472C4", "D9E2F3"),
        _ => ("💡 TIP: ", "70AD47", "E2EFDA"),
    };

    let mut para = doc.insert_paragraph(index, "");
    para = para
        .shading(bg_color)
        .border_all(BorderStyle::Single, 4, border_color)
        .indent_left(Length::inches(0.3))
        .indent_right(Length::inches(0.2))
        .space_before(Length::pt(8.0))
        .space_after(Length::pt(8.0));

    para.add_run(prefix).font("Garamond").size(10.0).bold(true);
    para.add_run(text).font("Garamond").size(10.0);
}

/// Insert a scene break for novels.
pub fn insert_scene_break(doc: &mut Document, index: usize, style: &str) {
    let symbol = match style {
        "diamond" => "◆",
        "blank" => "",
        _ => "* * *",
    };
    let mut para = doc.insert_paragraph(index, "");
    para = para
        .alignment(Alignment::Center)
        .space_before(Length::pt(18.0))
        .space_after(Length::pt(18.0));
    if !symbol.is_empty() {
        para.add_run(symbol).font("Garamond").size(11.0);
    }
}

/// Insert a best-seller chapter opening at `index`: a drop-cap initial letter
/// (its own frame paragraph), the next few words in small caps, then the rest
/// of the paragraph in the body style. The body paragraph wraps around the
/// drop cap and is NOT first-line indented, per fiction convention.
pub fn insert_chapter_opening(doc: &mut Document, index: usize, text: &str, font: &str) {
    let mut chars = text.chars();
    let first: String = chars.by_ref().take(1).collect();
    let rest: String = chars.collect();

    // Paragraph 1: the drop-cap letter alone. Use a 2-line frame and size the
    // letter (~3.8x body) so its cap-height fills exactly those two lines —
    // matching the frame avoids the empty gap a 3-line frame leaves.
    let mut cap = doc.insert_paragraph(index, "");
    cap = cap.drop_cap(2);
    cap.add_run(&first).font(font).bold(true).size(44.0);

    // Lead-in: first ~3 words after the initial letter rendered in small caps.
    let mut lead_words = 0;
    let mut split = rest.len();
    for (i, c) in rest.char_indices() {
        if c == ' ' {
            lead_words += 1;
            if lead_words == 3 { split = i; break; }
        }
    }
    let (lead, tail) = rest.split_at(split);

    // Paragraph 2: small-caps lead + body tail; wraps around the drop cap.
    let mut body = doc.insert_paragraph(index + 1, "");
    body = body.first_line_indent(Length::pt(0.0));
    if !lead.is_empty() {
        body.add_run(lead).font(font).small_caps(true);
    }
    if !tail.is_empty() {
        body.add_run(&tail).font(font);
    }
}
