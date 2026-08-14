use zavora_docx::{Alignment, Document, Length};

fn main() {
    let mut doc = Document::new();

    // Cookbook setup: 8x10
    doc.set_page_size(Length::inches(8.0), Length::inches(10.0));
    doc.set_margins(
        Length::inches(0.75),
        Length::inches(0.75),
        Length::inches(0.75),
        Length::inches(1.0),
    );
    doc.set_footer_page_number();
    doc.set_different_first_page(true);
    doc.set_first_page_footer("");
    doc.set_title("The Rustic Kitchen");
    doc.set_author("Chef Maria Santos");
    doc.set_text_watermark("SAMPLE", "E8E8E8", Some(-45));

    // Title page
    let mut p = doc.add_paragraph("");
    p = p
        .alignment(Alignment::Center)
        .space_before(Length::inches(3.0));
    p.add_run("The Rustic Kitchen")
        .font("Georgia")
        .size(36.0)
        .bold(true);

    let mut p = doc.add_paragraph("");
    p = p.alignment(Alignment::Center);
    p.add_run("Simple Recipes for Extraordinary Meals")
        .font("Georgia")
        .size(14.0)
        .italic(true);

    let mut p = doc.add_paragraph("");
    p = p
        .alignment(Alignment::Center)
        .space_before(Length::inches(1.0));
    p.add_run("Chef Maria Santos").font("Georgia").size(16.0);

    // Copyright page
    let mut p = doc.add_paragraph("");
    p = p.page_break_before(true);
    p.add_run("Copyright © 2026 Maria Santos. All rights reserved.")
        .font("Georgia")
        .size(9.0);
    let mut p = doc.add_paragraph("");
    p.add_run("Published by Zavora Press")
        .font("Georgia")
        .size(9.0);
    let mut p = doc.add_paragraph("");
    p.add_run("ISBN: 978-0-000000-00-0")
        .font("Georgia")
        .size(9.0);

    // TOC
    let mut p = doc.add_paragraph("");
    p = p.page_break_before(true).alignment(Alignment::Center);
    p.add_run("Contents").font("Georgia").size(20.0).bold(true);
    doc.insert_toc(doc.paragraph_count(), 2);

    // Chapter 1
    let mut p = doc.add_paragraph("");
    p = p
        .page_break_before(true)
        .alignment(Alignment::Center)
        .space_before(Length::inches(2.0));
    p.add_run("Chapter 1")
        .font("Georgia")
        .size(12.0)
        .small_caps(true);
    p.bookmark(1, "chapter1");

    let mut p = doc.add_paragraph("");
    p = p
        .alignment(Alignment::Center)
        .space_after(Length::pt(24.0))
        .outline_level(0);
    p.add_run("Breakfast Favorites")
        .font("Georgia")
        .size(24.0)
        .bold(true);

    // Chapter intro with footnote
    let fn1 = doc.add_footnote("Studies show that a protein-rich breakfast improves cognitive function by 23% (Harvard Medical, 2024).");
    let mut p = doc.add_paragraph("");
    p.add_run("There's nothing quite like starting the day with a meal made from scratch. These breakfast recipes have been perfected over twenty years of early mornings").font("Georgia").size(11.0);
    p.add_run("").footnote_ref(fn1);
    p.add_run(". Each one is designed to be ready in under 30 minutes.")
        .font("Georgia")
        .size(11.0);

    // Recipe 1
    let mut p = doc.add_paragraph("");
    p = p.space_before(Length::pt(24.0)).outline_level(1);
    p.add_run("Sourdough Pancakes with Maple Butter")
        .font("Georgia")
        .size(18.0)
        .bold(true);
    p.bookmark(2, "sourdough_pancakes");

    let mut p = doc.add_paragraph("");
    p.add_run("Prep: 10 min  |  Cook: 15 min  |  Serves: 4")
        .font("Georgia")
        .size(9.0)
        .color("666666");

    // Ingredients as a table
    let mut table = doc.insert_table(doc.content_count(), 2, 6);
    table = table.width_pct(60.0);
    if let Some(mut c) = table.cell(0, 0) {
        c.set_text("INGREDIENTS");
    }
    if let Some(mut c) = table.cell(0, 1) {
        c.set_text("");
    }
    if let Some(mut c) = table.cell(1, 0) {
        c.set_text("2 cups");
    }
    if let Some(mut c) = table.cell(1, 1) {
        c.set_text("sourdough starter (fed)");
    }
    if let Some(mut c) = table.cell(2, 0) {
        c.set_text("1½ cups");
    }
    if let Some(mut c) = table.cell(2, 1) {
        c.set_text("buttermilk");
    }
    if let Some(mut c) = table.cell(3, 0) {
        c.set_text("2");
    }
    if let Some(mut c) = table.cell(3, 1) {
        c.set_text("large eggs");
    }
    if let Some(mut c) = table.cell(4, 0) {
        c.set_text("3 tbsp");
    }
    if let Some(mut c) = table.cell(4, 1) {
        c.set_text("melted butter");
    }
    if let Some(mut c) = table.cell(5, 0) {
        c.set_text("1 tsp");
    }
    if let Some(mut c) = table.cell(5, 1) {
        c.set_text("vanilla extract");
    }

    // Instructions
    let mut p = doc.add_paragraph("");
    p = p.space_before(Length::pt(12.0));
    p.add_run("INSTRUCTIONS")
        .font("Georgia")
        .size(10.0)
        .bold(true)
        .small_caps(true);

    doc.add_numbered_list_item(
        "Whisk starter, buttermilk, eggs, butter, and vanilla in a large bowl until smooth.",
        0,
    );
    doc.add_numbered_list_item(
        "In a separate bowl, combine flour, sugar, baking soda, and salt.",
        0,
    );
    doc.add_numbered_list_item(
        "Fold dry ingredients into wet — don't overmix. A few lumps are fine.",
        0,
    );
    doc.add_numbered_list_item("Heat a griddle to 375°F. Pour ¼ cup batter per pancake.", 0);
    doc.add_numbered_list_item(
        "Cook until bubbles form on surface (2-3 min), flip, cook 1-2 min more.",
        0,
    );

    // Chef's tip (callout)
    let idx = doc.content_count();
    docx_mcp_server::engine::insert_callout(&mut doc, idx, "tip", "The secret to fluffy sourdough pancakes is a well-fed starter. Feed it 12 hours before and let it double in size.");

    // Comment from editor
    doc.add_comment(
        1,
        "Editor",
        "Consider adding a photo of the finished pancakes here.",
    );
    let mut p = doc.add_paragraph("");
    p.comment_start(1);
    p.add_run("Serve immediately with maple butter and fresh berries.")
        .font("Georgia")
        .size(11.0)
        .italic(true);
    p.comment_end(1);

    // Hyperlink to source
    let rel = doc.add_hyperlink_rel("https://www.kingarthurbaking.com/recipes/sourdough-pancakes");
    let mut p = doc.add_paragraph("");
    p.add_run("Adapted from: ")
        .font("Georgia")
        .size(9.0)
        .color("666666");
    p.add_hyperlink_run("King Arthur Baking", Some(&rel), None)
        .color("0563C1")
        .underline(true)
        .size(9.0);

    // Save
    doc.save("/Users/jameskaranja/Downloads/kdp_cookbook_sample.docx")
        .unwrap();
    println!("✓ Saved ~/Downloads/kdp_cookbook_sample.docx");
}
