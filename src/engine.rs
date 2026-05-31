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

// ── Business / professional templates ────────────────────────────────────────

/// US Letter page + a clean corporate theme. `accent` is the heading/brand hex.
fn business_base(doc: &mut Document, title: &str, accent: &str, heading_font: &str, body_font: &str) {
    doc.set_page_size(Length::inches(8.5), Length::inches(11.0));
    doc.set_margins(Length::inches(1.0), Length::inches(1.0), Length::inches(1.0), Length::inches(1.0));
    doc.set_title(title);
    doc.set_theme(
        &[("dk1","1A1A1A"),("lt1","FFFFFF"),("dk2","404040"),("lt2","F2F2F2"),
          ("accent1",accent),("accent2","C0392B"),("accent3","27AE60"),
          ("accent4","F39C12"),("accent5","8E44AD"),("accent6","16A085"),
          ("hlink",accent),("folHlink","8E44AD")],
        heading_font, body_font,
    );
    apply_book_styles(doc, &BookStyle {
        body_font, heading_font,
        body_pt: 11.0, line_spacing: 1.15, justified: false, heading_color: accent,
    });
}

/// Heading-style helper: a paragraph in Heading{level} style.
fn heading(doc: &mut Document, level: u32, text: &str) {
    doc.add_paragraph(text).style(&format!("Heading{level}"));
}

/// Set a table cell's text with alignment and optional bold, replacing the
/// default empty paragraph so the alignment actually applies.
fn cell_text(t: &mut zavora_docx::Table, row: usize, col: usize, text: &str, align: Alignment, bold: bool) {
    if let Some(mut c) = t.cell(row, col) {
        c.remove_first_empty_paragraph();
        let mut p = c.add_paragraph("").alignment(align);
        let r = p.add_run(text);
        if bold { r.bold(true); }
    }
}

/// A resume-style section heading: bold all-caps accent label with a hairline
/// rule underneath, for clean visual separation between sections.
fn section_rule(doc: &mut Document, text: &str, accent: &str) {
    use zavora_docx::BorderStyle;
    let mut p = doc
        .add_paragraph("")
        .space_before(Length::pt(12.0))
        .space_after(Length::pt(4.0))
        .border_bottom(BorderStyle::Single, 6, accent);
    p.add_run(text).bold(true).size(12.0).color(accent).all_caps(true);
}

/// A two-part line: `left` left-aligned, `right` pushed to a right tab stop at
/// `right_edge` inches — for "Title — Company .... Dates".
fn dated_entry(doc: &mut Document, left: &str, left_bold: bool, right: &str, right_edge: f64) {
    use zavora_docx::TabAlignment;
    let mut p = doc.add_paragraph("").add_tab_stop(TabAlignment::Right, Length::inches(right_edge));
    {
        let l = p.add_run(left);
        if left_bold { l.bold(true); }
    }
    p.add_tab();
    p.add_run(right).italic(true);
}

/// Optional template data: `f.get("key", "[Placeholder]")` returns the supplied
/// value or the placeholder; `f.arr("key")` yields a JSON array (or empty).
pub struct Fields<'a>(&'a serde_json::Value);
impl<'a> Fields<'a> {
    pub fn new(v: &'a serde_json::Value) -> Self { Fields(v) }
    fn get(&self, key: &str, default: &str) -> String {
        self.0.get(key).and_then(|v| v.as_str()).map(str::trim)
            .filter(|s| !s.is_empty()).map(String::from)
            .unwrap_or_else(|| default.to_string())
    }
    fn arr(&self, key: &str) -> &'a [serde_json::Value] {
        self.0.get(key).and_then(|v| v.as_array()).map(Vec::as_slice).unwrap_or(&[])
    }
}

/// Read a string field off a JSON object, falling back to `default`.
fn jstr<'a>(v: &'a serde_json::Value, key: &str, default: &'a str) -> &'a str {
    v.get(key).and_then(|x| x.as_str()).map(str::trim).filter(|s| !s.is_empty()).unwrap_or(default)
}

/// Format a currency amount with thousands separators: 12345.5 -> "$12,345.50".
fn money(v: f64) -> String {
    let s = format!("{:.2}", v.abs());
    let (int, frac) = s.split_once('.').unwrap_or((&s, "00"));
    let grouped: String = int.as_bytes().rchunks(3).rev()
        .map(|c| std::str::from_utf8(c).unwrap()).collect::<Vec<_>>().join(",");
    format!("{}${}.{}", if v < 0.0 { "-" } else { "" }, grouped, frac)
}

/// Show whole quantities as integers, fractional ones as-is.
fn fmt_qty(q: f64) -> String {
    if q.fract() == 0.0 { format!("{}", q as i64) } else { format!("{}", q) }
}

/// A bordered Description/Qty/Unit Price/Amount table with an auto-summed
/// total row (`total_label` names it, e.g. "TOTAL" or "PAID"). Reused by
/// invoice-like documents (quote, purchase order, receipt).
fn priced_items_table(doc: &mut Document, f: &Fields, accent: &str, total_label: &str) {
    use zavora_docx::BorderStyle;
    let items = f.arr("items");
    let n = items.len().max(4);
    let mut t = doc.add_table(n + 2, 4)
        .borders(BorderStyle::Single, 4, "D0D0D0")
        .column_widths(&[Length::inches(3.4), Length::inches(0.8), Length::inches(1.2), Length::inches(1.2)]);
    t.header_row_style(accent, "FFFFFF");
    cell_text(&mut t, 0, 0, "Description", Alignment::Left, true);
    cell_text(&mut t, 0, 1, "Qty", Alignment::Center, true);
    cell_text(&mut t, 0, 2, "Unit Price", Alignment::Right, true);
    cell_text(&mut t, 0, 3, "Amount", Alignment::Right, true);
    let mut total = 0.0;
    for r in 1..=n {
        match items.get(r - 1) {
            Some(it) => {
                let qty = it.get("qty").and_then(|v| v.as_f64()).unwrap_or(1.0);
                let price = it.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let amount = it.get("amount").and_then(|v| v.as_f64()).unwrap_or(qty * price);
                total += amount;
                cell_text(&mut t, r, 0, jstr(it, "description", "[Item or service description]"), Alignment::Left, false);
                cell_text(&mut t, r, 1, &fmt_qty(qty), Alignment::Center, false);
                cell_text(&mut t, r, 2, &money(price), Alignment::Right, false);
                cell_text(&mut t, r, 3, &money(amount), Alignment::Right, false);
            }
            None => {
                cell_text(&mut t, r, 0, "[Item or service description]", Alignment::Left, false);
                cell_text(&mut t, r, 1, "1", Alignment::Center, false);
                cell_text(&mut t, r, 2, "$0.00", Alignment::Right, false);
                cell_text(&mut t, r, 3, "$0.00", Alignment::Right, false);
            }
        }
    }
    let tr = n + 1;
    for col in 0..4 {
        if let Some(c) = t.cell(tr, col) { c.shading("EFEFEF"); }
    }
    cell_text(&mut t, tr, 2, total_label, Alignment::Right, true);
    cell_text(&mut t, tr, 3, &if items.is_empty() { "$0.00".into() } else { money(total) }, Alignment::Right, true);
}

/// Business report: title page-style header, sections scaffold, page numbers.
pub fn create_business_report(doc: &mut Document, f: &Fields) {
    business_base(doc, "Business Report", "2980B9", "Calibri", "Cambria");
    doc.set_footer_page_number();
    doc.add_paragraph(&f.get("title", "BUSINESS REPORT").to_uppercase()).style("Heading1").alignment(Alignment::Center);
    doc.add_paragraph(&f.get("subtitle", "Subtitle / Reporting Period")).alignment(Alignment::Center);
    doc.add_paragraph(&format!("Prepared by: {} · {}", f.get("author", "[Author]"), f.get("date", "[Date]"))).alignment(Alignment::Center);
    doc.add_paragraph("");
    let accent = "2980B9";
    section_rule(doc, "Executive Summary", accent);
    doc.add_paragraph(&f.get("summary", "[Summarize the report's key findings and recommendations.]"));
    section_rule(doc, "Introduction", accent);
    doc.add_paragraph(&f.get("introduction", "[Background and objectives.]"));
    section_rule(doc, "Findings", accent);
    doc.add_paragraph(&f.get("findings", "[Present your analysis and data.]"));
    section_rule(doc, "Recommendations", accent);
    doc.add_paragraph(&f.get("recommendations", "[Actionable next steps.]"));
    section_rule(doc, "Conclusion", accent);
    doc.add_paragraph(&f.get("conclusion", "[Closing remarks.]"));
}

/// Resume / CV: name header, contact line, ruled sections, tab-aligned dates,
/// and proper bulleted accomplishments.
pub fn create_resume(doc: &mut Document, f: &Fields) {
    let accent = "1F3A5F";
    business_base(doc, "Resume", accent, "Calibri", "Calibri");
    doc.set_margins(Length::inches(0.75), Length::inches(0.75), Length::inches(0.75), Length::inches(0.75));
    let edge = 7.0; // 8.5" - 2×0.75" printable width

    doc.add_paragraph(&f.get("name", "YOUR NAME").to_uppercase()).style("Heading1").alignment(Alignment::Center);
    doc.add_paragraph(&f.get("contact", "City, State · email@example.com · (555) 555-5555 · linkedin.com/in/you"))
        .alignment(Alignment::Center);

    section_rule(doc, "Professional Summary", accent);
    doc.add_paragraph(&f.get("summary", "[2-3 sentence summary of your experience, strengths, and target role.]"));

    section_rule(doc, "Experience", accent);
    let exp = f.arr("experience");
    if exp.is_empty() {
        dated_entry(doc, "Job Title — Company", true, "[Start – End]", edge);
        { doc.add_paragraph("").add_run("[City, State]").italic(true); }
        doc.add_bullet_list_item("[Accomplishment with a measurable result — e.g. grew X by 30%.]", 0);
        doc.add_bullet_list_item("[Accomplishment demonstrating ownership and impact.]", 0);
        dated_entry(doc, "Previous Title — Company", true, "[Start – End]", edge);
        doc.add_bullet_list_item("[Key contribution or initiative led.]", 0);
    } else {
        for e in exp {
            dated_entry(doc, jstr(e, "title", "Job Title — Company"), true, jstr(e, "dates", ""), edge);
            let loc = jstr(e, "location", "");
            if !loc.is_empty() { doc.add_paragraph("").add_run(loc).italic(true); }
            for b in e.get("bullets").and_then(|v| v.as_array()).map(Vec::as_slice).unwrap_or(&[]) {
                if let Some(s) = b.as_str() { doc.add_bullet_list_item(s, 0); }
            }
        }
    }

    section_rule(doc, "Education", accent);
    let edu = f.arr("education");
    if edu.is_empty() {
        dated_entry(doc, "Degree, Institution", true, "[Year]", edge);
    } else {
        for e in edu {
            dated_entry(doc, jstr(e, "degree", "Degree, Institution"), true, jstr(e, "year", ""), edge);
        }
    }

    section_rule(doc, "Skills", accent);
    doc.add_paragraph(&f.get("skills", "[Skill] · [Skill] · [Skill] · [Skill] · [Skill] · [Skill]"));
}

/// Business letter: block format with sender/recipient/date/salutation/body/closing.
pub fn create_letter(doc: &mut Document, f: &Fields) {
    business_base(doc, "Letter", "1A1A1A", "Cambria", "Cambria");
    doc.add_paragraph(&f.get("sender_name", "[Your Name]"));
    doc.add_paragraph(&f.get("sender_address", "[Street Address]"));
    doc.add_paragraph(&f.get("sender_city", "[City, State ZIP]"));
    doc.add_paragraph("");
    doc.add_paragraph(&f.get("date", "[Date]"));
    doc.add_paragraph("");
    doc.add_paragraph(&f.get("recipient_name", "[Recipient Name]"));
    doc.add_paragraph(&f.get("recipient_title", "[Title, Company]"));
    doc.add_paragraph(&f.get("recipient_address", "[Address]"));
    doc.add_paragraph("");
    doc.add_paragraph(&format!("Dear {},", f.get("salutation", "[Recipient]")));
    doc.add_paragraph("");
    doc.add_paragraph(&f.get("opening", "[Opening paragraph — state your purpose.]"));
    doc.add_paragraph(&f.get("body", "[Body paragraph — provide detail and context.]"));
    doc.add_paragraph(&f.get("closing", "[Closing paragraph — call to action / next steps.]"));
    doc.add_paragraph("");
    doc.add_paragraph(&f.get("sign_off", "Sincerely,"));
    doc.add_paragraph("");
    doc.add_paragraph(&f.get("sender_name", "[Your Name]"));
}

/// Memo: ruled MEMORANDUM header + sized TO/FROM/DATE/RE block + body.
pub fn create_memo(doc: &mut Document, f: &Fields) {
    let accent = "404040";
    business_base(doc, "Memo", accent, "Calibri", "Calibri");
    section_rule(doc, "Memorandum", accent);
    let mut t = doc.add_table(4, 2)
        .column_widths(&[Length::inches(1.0), Length::inches(5.5)]);
    let vals = [
        ("TO:", f.get("to", "[ ... ]")),
        ("FROM:", f.get("from", "[ ... ]")),
        ("DATE:", f.get("date", "[ ... ]")),
        ("RE:", f.get("re", "[ ... ]")),
    ];
    for (i, (label, value)) in vals.iter().enumerate() {
        cell_text(&mut t, i, 0, label, Alignment::Left, true);
        cell_text(&mut t, i, 1, value, Alignment::Left, false);
    }
    doc.add_paragraph("");
    doc.add_paragraph(&f.get("purpose", "[Purpose of the memo in one line.]"));
    doc.add_paragraph(&f.get("body", "[Body — context, details, and any action required.]"));
    doc.add_paragraph(&f.get("closing", "[Closing — deadlines or contact for questions.]"));
}

/// Invoice: branded header, bill-to/meta blocks, a bordered line-items table
/// with right-aligned currency, and an emphasized total row.
pub fn create_invoice(doc: &mut Document, f: &Fields) {
    let accent = "1F6F54";
    business_base(doc, "Invoice", accent, "Calibri", "Calibri");

    // Masthead: company on the left, big INVOICE wordmark established by Heading1.
    {
        let mut p = doc.add_paragraph("");
        p.add_run(&f.get("company", "[Your Company]")).bold(true).size(15.0).color(accent);
    }
    doc.add_paragraph(&f.get("company_details", "[Street Address · City, State ZIP · email@company.com · (555) 555-5555]"));
    doc.add_paragraph("INVOICE").style("Heading1");

    // Meta block: a borderless 2-col table puts label/value pairs on the right.
    {
        let mut m = doc.add_table(3, 2)
            .column_widths(&[Length::inches(1.2), Length::inches(2.0)]);
        let meta = [("Invoice #", f.get("number", "[0001]")), ("Date", f.get("date", "[Date]")), ("Due", f.get("due", "[Date]"))];
        for (i, (k, v)) in meta.iter().enumerate() {
            cell_text(&mut m, i, 0, k, Alignment::Left, true);
            cell_text(&mut m, i, 1, v, Alignment::Left, false);
        }
    }
    doc.add_paragraph("");
    {
        let mut p = doc.add_paragraph("");
        p.add_run("Bill To").bold(true).color(accent);
    }
    doc.add_paragraph(&f.get("bill_to", "[Client Name · Company · Address]"));
    doc.add_paragraph("");

    priced_items_table(doc, f, accent, "TOTAL");

    doc.add_paragraph("");
    {
        let mut p = doc.add_paragraph("");
        p.add_run("Payment terms: ").bold(true);
        p.add_run(&f.get("terms", "Net 30. Make checks payable to [Your Company]. Thank you for your business."));
    }
}

/// Newsletter: ruled masthead + two-column body with ruled article headings.
pub fn create_newsletter(doc: &mut Document, f: &Fields) {
    let accent = "6C3483";
    business_base(doc, "Newsletter", accent, "Calibri", "Georgia");
    doc.add_paragraph(&f.get("title", "THE NEWSLETTER").to_uppercase()).style("Heading1").alignment(Alignment::Center);
    {
        use zavora_docx::BorderStyle;
        doc.add_paragraph("")
            .alignment(Alignment::Center)
            .border_bottom(BorderStyle::Single, 8, accent)
            .add_run(&f.get("masthead", "[Organization]  ·  [Volume / Issue]  ·  [Date]"));
    }
    doc.add_paragraph("");
    doc.set_columns(2, Length::inches(0.3));
    section_rule(doc, "Lead Story", accent);
    doc.add_paragraph(&f.get("lead", "[Open with the most important news for your readers.]"));
    section_rule(doc, "In This Issue", accent);
    doc.add_paragraph(&f.get("secondary", "[Secondary article or announcements.]"));
    section_rule(doc, "Upcoming Events", accent);
    doc.add_paragraph(&f.get("events", "[Dates and details.]"));
}

/// Academic paper: title block, abstract, numbered sections, references.
pub fn create_academic(doc: &mut Document, f: &Fields) {
    business_base(doc, "Academic Paper", "1A1A1A", "Times New Roman", "Times New Roman");
    apply_book_styles(doc, &BookStyle {
        body_font: "Times New Roman", heading_font: "Times New Roman",
        body_pt: 12.0, line_spacing: 2.0, justified: false, heading_color: "1A1A1A",
    });
    doc.set_footer_page_number();
    doc.add_paragraph(&f.get("title", "Paper Title")).style("Heading1").alignment(Alignment::Center);
    doc.add_paragraph(&f.get("author", "Author Name")).alignment(Alignment::Center);
    doc.add_paragraph(&f.get("affiliation", "Institution / Affiliation")).alignment(Alignment::Center);
    doc.add_paragraph("");
    heading(doc, 2, "Abstract");
    doc.add_paragraph(&f.get("abstract", "[150-250 word summary of the research.]"));
    heading(doc, 2, "1. Introduction");
    doc.add_paragraph(&f.get("introduction", "[Problem statement, motivation, and contributions.]"));
    heading(doc, 2, "2. Methods");
    doc.add_paragraph(&f.get("methods", "[Describe your approach and materials.]"));
    heading(doc, 2, "3. Results");
    doc.add_paragraph(&f.get("results", "[Present findings, with figures/tables.]"));
    heading(doc, 2, "4. Discussion");
    doc.add_paragraph(&f.get("discussion", "[Interpret results and limitations.]"));
    heading(doc, 2, "References");
    doc.add_paragraph(&f.get("references", "[1] Author, A. (Year). Title. Journal."));
}

/// Project proposal: cover block, ruled sections, and a priced deliverables
/// table with an auto-summed total.
pub fn create_proposal(doc: &mut Document, f: &Fields) {
    use zavora_docx::BorderStyle;
    let accent = "B9770E";
    business_base(doc, "Proposal", accent, "Calibri", "Cambria");
    doc.set_footer_page_number();
    doc.add_paragraph(&f.get("title", "PROJECT PROPOSAL").to_uppercase()).style("Heading1").alignment(Alignment::Center);
    doc.add_paragraph(&format!("Prepared for {} · by {}", f.get("client", "[Client]"), f.get("author", "[Your Company]"))).alignment(Alignment::Center);
    doc.add_paragraph(&f.get("date", "[Date]")).alignment(Alignment::Center);
    section_rule(doc, "Overview", accent);
    doc.add_paragraph(&f.get("overview", "[Summarize the problem and your proposed solution.]"));
    section_rule(doc, "Scope of Work", accent);
    doc.add_paragraph(&f.get("scope", "[Describe deliverables, approach, and boundaries.]"));
    section_rule(doc, "Timeline", accent);
    doc.add_paragraph(&f.get("timeline", "[Phases, milestones, and dates.]"));
    section_rule(doc, "Investment", accent);
    let items = f.arr("items");
    let n = items.len().max(3);
    let mut t = doc.add_table(n + 2, 3)
        .borders(BorderStyle::Single, 4, "D0D0D0")
        .column_widths(&[Length::inches(4.2), Length::inches(1.0), Length::inches(1.3)]);
    t.header_row_style(accent, "FFFFFF");
    cell_text(&mut t, 0, 0, "Deliverable", Alignment::Left, true);
    cell_text(&mut t, 0, 1, "Qty", Alignment::Center, true);
    cell_text(&mut t, 0, 2, "Amount", Alignment::Right, true);
    let mut total = 0.0;
    for r in 1..=n {
        match items.get(r - 1) {
            Some(it) => {
                let qty = it.get("qty").and_then(|v| v.as_f64()).unwrap_or(1.0);
                let price = it.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let amount = it.get("amount").and_then(|v| v.as_f64()).unwrap_or(qty * price);
                total += amount;
                cell_text(&mut t, r, 0, jstr(it, "description", "[Deliverable]"), Alignment::Left, false);
                cell_text(&mut t, r, 1, &fmt_qty(qty), Alignment::Center, false);
                cell_text(&mut t, r, 2, &money(amount), Alignment::Right, false);
            }
            None => {
                cell_text(&mut t, r, 0, "[Deliverable]", Alignment::Left, false);
                cell_text(&mut t, r, 1, "1", Alignment::Center, false);
                cell_text(&mut t, r, 2, "$0.00", Alignment::Right, false);
            }
        }
    }
    let tr = n + 1;
    for col in 0..3 {
        if let Some(c) = t.cell(tr, col) { c.shading("EFEFEF"); }
    }
    cell_text(&mut t, tr, 1, "TOTAL", Alignment::Right, true);
    cell_text(&mut t, tr, 2, &if items.is_empty() { "$0.00".into() } else { money(total) }, Alignment::Right, true);
    section_rule(doc, "Acceptance", accent);
    doc.add_paragraph(&f.get("acceptance", "[Signature, date, and terms of acceptance.]"));
}

/// Meeting agenda: meta block + ruled item list with right-aligned durations.
pub fn create_agenda(doc: &mut Document, f: &Fields) {
    let accent = "2C3E50";
    business_base(doc, "Agenda", accent, "Calibri", "Calibri");
    doc.set_margins(Length::inches(0.75), Length::inches(0.75), Length::inches(0.75), Length::inches(0.75));
    let edge = 7.0;
    doc.add_paragraph(&f.get("title", "MEETING AGENDA").to_uppercase()).style("Heading1").alignment(Alignment::Center);
    let mut t = doc.add_table(3, 2)
        .column_widths(&[Length::inches(1.2), Length::inches(5.3)]);
    let meta = [("Date", f.get("date", "[Date]")), ("Time", f.get("time", "[Time]")), ("Location", f.get("location", "[Location]"))];
    for (i, (k, v)) in meta.iter().enumerate() {
        cell_text(&mut t, i, 0, k, Alignment::Left, true);
        cell_text(&mut t, i, 1, v, Alignment::Left, false);
    }
    doc.add_paragraph(&format!("Attendees: {}", f.get("attendees", "[Names]")));
    section_rule(doc, "Agenda", accent);
    let items = f.arr("items");
    if items.is_empty() {
        for d in ["[Topic — owner]", "[Topic — owner]", "[Topic — owner]"] {
            dated_entry(doc, d, false, "[min]", edge);
        }
    } else {
        for it in items {
            dated_entry(doc, jstr(it, "topic", "[Topic]"), true, jstr(it, "duration", ""), edge);
            let owner = jstr(it, "owner", "");
            if !owner.is_empty() { doc.add_paragraph("").add_run(owner).italic(true); }
        }
    }
    section_rule(doc, "Notes", accent);
    doc.add_paragraph(&f.get("notes", "[Action items and decisions.]"));
}

/// Press release: FOR IMMEDIATE RELEASE banner, headline, dateline, body,
/// boilerplate, and an end marker.
pub fn create_press_release(doc: &mut Document, f: &Fields) {
    let accent = "1A1A1A";
    business_base(doc, "Press Release", accent, "Arial", "Georgia");
    {
        let mut p = doc.add_paragraph("");
        p.add_run(&f.get("status", "FOR IMMEDIATE RELEASE")).bold(true).all_caps(true).color(accent);
    }
    doc.add_paragraph("");
    doc.add_paragraph(&f.get("headline", "Headline Goes Here")).style("Heading1").alignment(Alignment::Center);
    {
        let mut p = doc.add_paragraph("").alignment(Alignment::Center);
        p.add_run(&f.get("subhead", "[Optional subheadline]")).italic(true);
    }
    doc.add_paragraph("");
    {
        let mut p = doc.add_paragraph("");
        p.add_run(&format!("{} — ", f.get("dateline", "CITY, State, [Date]"))).bold(true);
        p.add_run(&f.get("body", "[Lead paragraph — the who/what/when/where/why.]"));
    }
    doc.add_paragraph(&f.get("body2", "[Supporting detail, quote, and context.]"));
    doc.add_paragraph("");
    section_rule(doc, &format!("About {}", f.get("organization", "")), accent);
    doc.add_paragraph(&f.get("boilerplate", "[One paragraph about your organization.]"));
    {
        let mut p = doc.add_paragraph("");
        p.add_run("Media contact: ").bold(true);
        p.add_run(&f.get("contact", "[Name · email · phone]"));
    }
    doc.add_paragraph("###").alignment(Alignment::Center);
}

/// Certificate: centered ornamental award with a large recipient name and a
/// signature/date footer.
pub fn create_certificate(doc: &mut Document, f: &Fields) {
    use zavora_docx::BorderStyle;
    let accent = "8A6D1F";
    business_base(doc, "Certificate", accent, "Georgia", "Georgia");
    doc.set_landscape();
    for _ in 0..2 { doc.add_paragraph(""); }
    doc.add_paragraph(&f.get("title", "Certificate of Achievement")).style("Heading1").alignment(Alignment::Center);
    {
        let mut p = doc.add_paragraph("").alignment(Alignment::Center);
        p.add_run(&f.get("presented", "This certificate is proudly presented to")).italic(true).size(13.0);
    }
    doc.add_paragraph("");
    {
        let mut p = doc.add_paragraph("")
            .alignment(Alignment::Center)
            .border_bottom(BorderStyle::Single, 6, accent);
        p.add_run(&f.get("recipient", "Recipient Name")).bold(true).size(30.0).color(accent);
    }
    doc.add_paragraph("");
    {
        let mut p = doc.add_paragraph("").alignment(Alignment::Center);
        p.add_run(&f.get("reason", "in recognition of outstanding achievement.")).size(13.0);
    }
    for _ in 0..2 { doc.add_paragraph(""); }
    let mut t = doc.add_table(2, 2)
        .column_widths(&[Length::inches(4.0), Length::inches(4.0)]);
    cell_text(&mut t, 0, 0, f.get("date", "[Date]").as_str(), Alignment::Center, false);
    cell_text(&mut t, 0, 1, f.get("signature", "[Signature]").as_str(), Alignment::Center, false);
    cell_text(&mut t, 1, 0, "Date", Alignment::Center, true);
    cell_text(&mut t, 1, 1, "Authorized by", Alignment::Center, true);
}

/// Cover letter: sender/recipient block, salutation, persuasive body, closing.
pub fn create_cover_letter(doc: &mut Document, f: &Fields) {
    business_base(doc, "Cover Letter", "1A1A1A", "Cambria", "Cambria");
    doc.add_paragraph(&f.get("name", "[Your Name]"));
    doc.add_paragraph(&f.get("contact", "[email · phone · city]"));
    doc.add_paragraph("");
    doc.add_paragraph(&f.get("date", "[Date]"));
    doc.add_paragraph("");
    doc.add_paragraph(&f.get("recipient", "[Hiring Manager]"));
    doc.add_paragraph(&f.get("company", "[Company]"));
    doc.add_paragraph("");
    doc.add_paragraph(&format!("Dear {},", f.get("salutation", "Hiring Manager")));
    doc.add_paragraph("");
    doc.add_paragraph(&f.get("opening", "[State the role you're applying for and a one-line hook.]"));
    doc.add_paragraph(&f.get("body", "[Why you're a fit — relevant achievements and skills.]"));
    doc.add_paragraph(&f.get("closing", "[Reiterate interest and invite next steps.]"));
    doc.add_paragraph("");
    doc.add_paragraph("Sincerely,");
    doc.add_paragraph("");
    doc.add_paragraph(&f.get("name", "[Your Name]"));
}

/// Fax cover sheet: ruled header + labeled routing table + note.
pub fn create_fax_cover(doc: &mut Document, f: &Fields) {
    let accent = "404040";
    business_base(doc, "Fax", accent, "Calibri", "Calibri");
    section_rule(doc, "Fax", accent);
    let mut t = doc.add_table(6, 2)
        .column_widths(&[Length::inches(1.2), Length::inches(5.3)]);
    let rows = [
        ("TO:", f.get("to", "[ ... ]")), ("FROM:", f.get("from", "[ ... ]")),
        ("FAX:", f.get("fax", "[ ... ]")), ("PHONE:", f.get("phone", "[ ... ]")),
        ("DATE:", f.get("date", "[ ... ]")), ("PAGES:", f.get("pages", "[ ... ]")),
    ];
    for (i, (k, v)) in rows.iter().enumerate() {
        cell_text(&mut t, i, 0, k, Alignment::Left, true);
        cell_text(&mut t, i, 1, v, Alignment::Left, false);
    }
    section_rule(doc, "Message", accent);
    doc.add_paragraph(&f.get("message", "[Note to the recipient.]"));
}

/// Price quote: company/meta/bill-to + priced items with a quoted TOTAL.
pub fn create_quote(doc: &mut Document, f: &Fields) {
    let accent = "2471A3";
    business_base(doc, "Quote", accent, "Calibri", "Calibri");
    {
        let mut p = doc.add_paragraph("");
        p.add_run(&f.get("company", "[Your Company]")).bold(true).size(15.0).color(accent);
    }
    doc.add_paragraph(&f.get("company_details", "[Address · email · phone]"));
    doc.add_paragraph("QUOTE").style("Heading1");
    {
        let mut m = doc.add_table(3, 2).column_widths(&[Length::inches(1.2), Length::inches(2.0)]);
        let meta = [("Quote #", f.get("number", "[0001]")), ("Date", f.get("date", "[Date]")), ("Valid until", f.get("valid_until", "[Date]"))];
        for (i, (k, v)) in meta.iter().enumerate() {
            cell_text(&mut m, i, 0, k, Alignment::Left, true);
            cell_text(&mut m, i, 1, v, Alignment::Left, false);
        }
    }
    doc.add_paragraph("");
    { let mut p = doc.add_paragraph(""); p.add_run("Prepared For").bold(true).color(accent); }
    doc.add_paragraph(&f.get("client", "[Client · Company · Address]"));
    doc.add_paragraph("");
    priced_items_table(doc, f, accent, "TOTAL");
    doc.add_paragraph("");
    { let mut p = doc.add_paragraph(""); p.add_run("Notes: ").bold(true); p.add_run(&f.get("notes", "Prices valid for 30 days. Taxes not included.")); }
}

/// Purchase order: vendor/ship-to blocks + priced items with a PO total.
pub fn create_purchase_order(doc: &mut Document, f: &Fields) {
    let accent = "1F6F54";
    business_base(doc, "Purchase Order", accent, "Calibri", "Calibri");
    {
        let mut p = doc.add_paragraph("");
        p.add_run(&f.get("company", "[Your Company]")).bold(true).size(15.0).color(accent);
    }
    doc.add_paragraph("PURCHASE ORDER").style("Heading1");
    {
        let mut m = doc.add_table(2, 2).column_widths(&[Length::inches(1.2), Length::inches(2.0)]);
        let meta = [("PO #", f.get("number", "[0001]")), ("Date", f.get("date", "[Date]"))];
        for (i, (k, v)) in meta.iter().enumerate() {
            cell_text(&mut m, i, 0, k, Alignment::Left, true);
            cell_text(&mut m, i, 1, v, Alignment::Left, false);
        }
    }
    doc.add_paragraph("");
    { let mut p = doc.add_paragraph(""); p.add_run("Vendor").bold(true).color(accent); }
    doc.add_paragraph(&f.get("vendor", "[Vendor · Address]"));
    { let mut p = doc.add_paragraph(""); p.add_run("Ship To").bold(true).color(accent); }
    doc.add_paragraph(&f.get("ship_to", "[Recipient · Address]"));
    doc.add_paragraph("");
    priced_items_table(doc, f, accent, "TOTAL");
    doc.add_paragraph("");
    { let mut p = doc.add_paragraph(""); p.add_run("Terms: ").bold(true); p.add_run(&f.get("terms", "Deliver by [date]. Reference this PO number on all correspondence.")); }
}

/// Payment receipt: receipt meta + priced items showing amount PAID.
pub fn create_receipt(doc: &mut Document, f: &Fields) {
    let accent = "117864";
    business_base(doc, "Receipt", accent, "Calibri", "Calibri");
    {
        let mut p = doc.add_paragraph("");
        p.add_run(&f.get("company", "[Your Company]")).bold(true).size(15.0).color(accent);
    }
    doc.add_paragraph("RECEIPT").style("Heading1");
    {
        let mut m = doc.add_table(3, 2).column_widths(&[Length::inches(1.4), Length::inches(2.0)]);
        let meta = [("Receipt #", f.get("number", "[0001]")), ("Date", f.get("date", "[Date]")), ("Method", f.get("method", "[Card / Cash]"))];
        for (i, (k, v)) in meta.iter().enumerate() {
            cell_text(&mut m, i, 0, k, Alignment::Left, true);
            cell_text(&mut m, i, 1, v, Alignment::Left, false);
        }
    }
    doc.add_paragraph("");
    { let mut p = doc.add_paragraph(""); p.add_run("Received From").bold(true).color(accent); }
    doc.add_paragraph(&f.get("payer", "[Payer name]"));
    doc.add_paragraph("");
    priced_items_table(doc, f, accent, "PAID");
    doc.add_paragraph("");
    doc.add_paragraph(&f.get("note", "Thank you for your payment.")).alignment(Alignment::Center);
}

/// Event flyer: large centered headline, subhead, ruled details, and call-out.
pub fn create_flyer(doc: &mut Document, f: &Fields) {
    let accent = "C0392B";
    business_base(doc, "Flyer", accent, "Arial", "Arial");
    for _ in 0..2 { doc.add_paragraph(""); }
    doc.add_paragraph(&f.get("headline", "EVENT TITLE").to_uppercase()).style("Heading1").alignment(Alignment::Center);
    { let mut p = doc.add_paragraph("").alignment(Alignment::Center); p.add_run(&f.get("subhead", "[Catchy one-liner]")).italic(true).size(15.0); }
    doc.add_paragraph("");
    for (label, key, dflt) in [("When", "when", "[Date · Time]"), ("Where", "where", "[Venue · Address]"), ("Details", "details", "[What to expect]")] {
        let mut p = doc.add_paragraph("").alignment(Alignment::Center);
        p.add_run(&format!("{}: ", label)).bold(true).color(accent);
        p.add_run(&f.get(key, dflt));
    }
    doc.add_paragraph("");
    { let mut p = doc.add_paragraph("").alignment(Alignment::Center); p.add_run(&f.get("cta", "RSVP at [email / link]")).bold(true).size(14.0).color(accent); }
}

/// Contract / agreement: title, parties, numbered clauses, signature blocks.
pub fn create_contract(doc: &mut Document, f: &Fields) {
    let accent = "1A1A1A";
    business_base(doc, "Agreement", accent, "Times New Roman", "Times New Roman");
    doc.set_footer_page_number();
    doc.add_paragraph(&f.get("title", "SERVICE AGREEMENT").to_uppercase()).style("Heading1").alignment(Alignment::Center);
    doc.add_paragraph(&format!("This agreement is made on {} between {} (\"Provider\") and {} (\"Client\").",
        f.get("date", "[Date]"), f.get("party_a", "[Party A]"), f.get("party_b", "[Party B]")));
    doc.add_paragraph("");
    let clauses = f.arr("clauses");
    if clauses.is_empty() {
        for (h, b) in [("Scope", "[Describe the services provided.]"), ("Payment", "[Fees, schedule, and method.]"),
            ("Term & Termination", "[Duration and how either party may end the agreement.]"),
            ("Confidentiality", "[Treatment of confidential information.]"),
            ("Governing Law", "[Jurisdiction whose laws apply.]")] {
            doc.add_numbered_list_item(&format!("{}. {}", h, b), 0);
        }
    } else {
        for c in clauses {
            let title = jstr(c, "title", "");
            let body = jstr(c, "body", "");
            let line = if title.is_empty() { body.to_string() } else { format!("{}. {}", title, body) };
            doc.add_numbered_list_item(&line, 0);
        }
    }
    doc.add_paragraph("");
    let mut t = doc.add_table(2, 2).column_widths(&[Length::inches(3.25), Length::inches(3.25)]);
    cell_text(&mut t, 0, 0, "_______________________", Alignment::Left, false);
    cell_text(&mut t, 0, 1, "_______________________", Alignment::Left, false);
    cell_text(&mut t, 1, 0, &f.get("party_a", "Provider"), Alignment::Left, true);
    cell_text(&mut t, 1, 1, &f.get("party_b", "Client"), Alignment::Left, true);
}

/// Meeting minutes: meta block, attendees, ruled discussion, and action items.
pub fn create_meeting_minutes(doc: &mut Document, f: &Fields) {
    let accent = "2C3E50";
    business_base(doc, "Minutes", accent, "Calibri", "Calibri");
    doc.add_paragraph(&f.get("title", "MEETING MINUTES").to_uppercase()).style("Heading1").alignment(Alignment::Center);
    let mut t = doc.add_table(3, 2).column_widths(&[Length::inches(1.2), Length::inches(5.3)]);
    let meta = [("Date", f.get("date", "[Date]")), ("Time", f.get("time", "[Time]")), ("Location", f.get("location", "[Location]"))];
    for (i, (k, v)) in meta.iter().enumerate() {
        cell_text(&mut t, i, 0, k, Alignment::Left, true);
        cell_text(&mut t, i, 1, v, Alignment::Left, false);
    }
    doc.add_paragraph(&format!("Attendees: {}", f.get("attendees", "[Names]")));
    section_rule(doc, "Discussion", accent);
    doc.add_paragraph(&f.get("discussion", "[Topics discussed and decisions reached.]"));
    section_rule(doc, "Action Items", accent);
    let actions = f.arr("actions");
    if actions.is_empty() {
        doc.add_bullet_list_item("[Owner — task — due date]", 0);
    } else {
        for a in actions {
            if let Some(s) = a.as_str() { doc.add_bullet_list_item(s, 0); }
        }
    }
}

/// Sign-in sheet: title + a bordered grid (Name / Organization / Time / Signature).
pub fn create_sign_in_sheet(doc: &mut Document, f: &Fields) {
    use zavora_docx::BorderStyle;
    let accent = "34495E";
    business_base(doc, "Sign-In Sheet", accent, "Calibri", "Calibri");
    doc.add_paragraph(&f.get("title", "SIGN-IN SHEET").to_uppercase()).style("Heading1").alignment(Alignment::Center);
    doc.add_paragraph(&format!("{} · {}", f.get("event", "[Event]"), f.get("date", "[Date]"))).alignment(Alignment::Center);
    doc.add_paragraph("");
    let rows = 16usize;
    let mut t = doc.add_table(rows, 4)
        .borders(BorderStyle::Single, 4, "B0B0B0")
        .column_widths(&[Length::inches(2.2), Length::inches(2.0), Length::inches(1.0), Length::inches(1.3)]);
    t.header_row_style(accent, "FFFFFF");
    for (c, h) in ["Name", "Organization", "Time", "Signature"].iter().enumerate() {
        cell_text(&mut t, 0, c, h, Alignment::Left, true);
    }
}

/// Business plan: title page-style header, page numbers, standard sections.
pub fn create_business_plan(doc: &mut Document, f: &Fields) {
    let accent = "6C3483";
    business_base(doc, "Business Plan", accent, "Calibri", "Cambria");
    doc.set_footer_page_number();
    doc.add_paragraph(&f.get("company", "COMPANY NAME").to_uppercase()).style("Heading1").alignment(Alignment::Center);
    doc.add_paragraph(&f.get("tagline", "Business Plan")).alignment(Alignment::Center);
    doc.add_paragraph(&f.get("date", "[Date]")).alignment(Alignment::Center);
    for (h, key, dflt) in [
        ("Executive Summary", "summary", "[The business in brief — what, who, and why now.]"),
        ("Company Description", "description", "[Mission, structure, and offering.]"),
        ("Market Analysis", "market", "[Target market, size, and competition.]"),
        ("Organization & Management", "organization", "[Team and roles.]"),
        ("Products & Services", "products", "[What you sell and the value.]"),
        ("Marketing & Sales", "marketing", "[How you reach and convert customers.]"),
        ("Financial Plan", "financials", "[Projections, funding needs, and milestones.]"),
    ] {
        section_rule(doc, h, accent);
        doc.add_paragraph(&f.get(key, dflt));
    }
}

/// Catalog of business template formats: (format id, description, accepted
/// data keys). Single source of truth for the list_templates tool.
pub fn template_catalog() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("business:report", "Business report with ruled sections and page numbers", "title, subtitle, author, date, summary, introduction, findings, recommendations, conclusion"),
        ("business:resume", "Resume/CV with ruled sections and tab-aligned dates", "name, contact, summary, skills, experience[{title,dates,location,bullets[]}], education[{degree,year}]"),
        ("business:letter", "Block-format business letter", "sender_name, sender_address, sender_city, date, recipient_name, recipient_title, recipient_address, salutation, opening, body, closing, sign_off"),
        ("business:memo", "Memo with TO/FROM/DATE/RE block", "to, from, date, re, purpose, body, closing"),
        ("business:invoice", "Invoice with priced line items and total", "company, company_details, number, date, due, bill_to, items[{description,qty,price,amount?}], terms"),
        ("business:newsletter", "Two-column newsletter with ruled masthead", "title, masthead, lead, secondary, events"),
        ("business:academic", "Double-spaced academic paper", "title, author, affiliation, abstract, introduction, methods, results, discussion, references"),
        ("business:proposal", "Project proposal with priced deliverables", "title, client, author, date, overview, scope, timeline, items[{description,qty,price,amount?}], acceptance"),
        ("business:agenda", "Meeting agenda with timed items", "title, date, time, location, attendees, items[{topic,owner,duration}], notes"),
        ("business:press_release", "Press release with dateline and boilerplate", "status, headline, subhead, dateline, body, body2, organization, boilerplate, contact"),
        ("business:certificate", "Landscape award certificate", "title, presented, recipient, reason, date, signature"),
        ("business:cover_letter", "Job application cover letter", "name, contact, date, recipient, company, salutation, opening, body, closing"),
        ("business:fax_cover", "Fax cover sheet with routing block", "to, from, fax, phone, date, pages, message"),
        ("business:quote", "Price quote with priced items", "company, company_details, number, date, valid_until, client, items[{description,qty,price,amount?}], notes"),
        ("business:purchase_order", "Purchase order with vendor/ship-to", "company, number, date, vendor, ship_to, items[{description,qty,price,amount?}], terms"),
        ("business:receipt", "Payment receipt showing amount paid", "company, number, date, method, payer, items[{description,qty,price,amount?}], note"),
        ("business:flyer", "Event flyer with centered details", "headline, subhead, when, where, details, cta"),
        ("business:contract", "Agreement with numbered clauses and signatures", "title, date, party_a, party_b, clauses[{title,body}]"),
        ("business:meeting_minutes", "Minutes with discussion and action items", "title, date, time, location, attendees, discussion, actions[]"),
        ("business:sign_in_sheet", "Bordered attendee sign-in grid", "title, event, date"),
        ("business:business_plan", "Business plan with standard sections", "company, tagline, date, summary, description, market, organization, products, marketing, financials"),
    ]
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
