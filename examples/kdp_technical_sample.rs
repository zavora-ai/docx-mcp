use zavora_docx::{Document, Length, Alignment, BorderStyle};
use docx_mcp_server::engine;

fn main() {
    let mut doc = Document::new();
    engine::create_kdp_technical(&mut doc);
    doc.set_title("Building MCP Servers in Rust");
    doc.set_author("James Karanja");

    // === TITLE PAGE ===
    let mut p = doc.add_paragraph("");
    p = p.alignment(Alignment::Center).space_before(Length::inches(2.5));
    p.add_run("Building MCP Servers").font("Garamond").size(32.0).bold(true).theme_color("dk2");
    let mut p = doc.add_paragraph("");
    p = p.alignment(Alignment::Center);
    p.add_run("in Rust").font("Garamond").size(32.0).bold(true).theme_color("accent1");
    let mut p = doc.add_paragraph("");
    p = p.alignment(Alignment::Center).space_before(Length::pt(24.0));
    p.add_run("A Practical Guide to the Model Context Protocol").font("Garamond").size(14.0).italic(true);
    let mut p = doc.add_paragraph("");
    p = p.alignment(Alignment::Center).space_before(Length::inches(1.5));
    p.add_run("James Karanja").font("Garamond").size(16.0);

    // === COPYRIGHT PAGE ===
    let mut p = doc.add_paragraph("");
    p = p.page_break_before(true).space_before(Length::inches(5.0));
    p.add_run("Copyright © 2026 James Karanja. All rights reserved.").font("Garamond").size(9.0);
    let mut p = doc.add_paragraph("");
    p.add_run("Published by Zavora Press, Nairobi").font("Garamond").size(9.0);
    let mut p = doc.add_paragraph("");
    p.add_run("ISBN: 978-1-234567-89-0").font("Garamond").size(9.0);
    let mut p = doc.add_paragraph("");
    p.add_run("First Edition, May 2026").font("Garamond").size(9.0);

    // === TABLE OF CONTENTS ===
    let mut p = doc.add_paragraph("");
    p = p.page_break_before(true).alignment(Alignment::Center).space_after(Length::pt(24.0));
    p.add_run("Contents").font("Garamond").size(20.0).bold(true);
    doc.insert_toc(doc.content_count(), 2);

    // === CHAPTER 1 ===
    let mut p = doc.add_paragraph("");
    p = p.page_break_before(true).alignment(Alignment::Center).space_before(Length::inches(2.0));
    p.add_run("CHAPTER ONE").font("Garamond").size(12.0).small_caps(true).theme_color("accent1");
    p.bookmark(1, "chapter1");

    let mut p = doc.add_paragraph("");
    p = p.alignment(Alignment::Center).space_after(Length::pt(36.0)).outline_level(0);
    p.add_run("Introduction to MCP").font("Garamond").size(24.0).bold(true);

    // Drop cap opening
    let mut p = doc.add_paragraph("");
    p = p.drop_cap(3);
    p.add_run("T").font("Garamond").size(48.0);
    let mut p = doc.add_paragraph("");
    p = p.first_line_indent(Length::inches(0.0));
    p.add_run("he Model Context Protocol represents a fundamental shift in how AI agents interact with external tools and data sources. Rather than building monolithic applications, MCP enables a composable architecture where servers expose capabilities and clients consume them through a standardized JSON-RPC interface.").font("Garamond").size(11.0);

    // Footnote
    let fn1 = doc.add_footnote("The MCP specification was first published in November 2024 by Anthropic. See https://modelcontextprotocol.io for the full specification.");
    let mut p = doc.add_paragraph("");
    p = p.first_line_indent(Length::inches(0.3));
    p.add_run("This book will guide you through building production-grade MCP servers using Rust and the rmcp crate").font("Garamond").size(11.0);
    p.add_run("").footnote_ref(fn1);
    p.add_run(". By the end, you'll have the skills to create servers that handle thousands of concurrent connections with minimal resource usage.").font("Garamond").size(11.0);

    // === SECTION: What is MCP? ===
    let mut p = doc.add_paragraph("");
    p = p.space_before(Length::pt(24.0)).outline_level(1);
    p.add_run("What is MCP?").font("Garamond").size(14.0).bold(true);
    p.bookmark(2, "what_is_mcp");

    let mut p = doc.add_paragraph("");
    p = p.first_line_indent(Length::inches(0.3));
    p.add_run("MCP is a protocol that standardizes how AI applications communicate with external data sources and tools. Think of it as USB for AI — a universal connector that lets any AI model talk to any tool.").font("Garamond").size(11.0);

    // Key concepts table
    let mut p = doc.add_paragraph("");
    p = p.space_before(Length::pt(12.0));
    p.add_run("Table 1.1: ").font("Garamond").size(9.0).italic(true);
    p.add_run("Core MCP Concepts").font("Garamond").size(9.0).italic(true);

    let mut table = doc.add_table(5, 3);
    table = table.width_pct(100.0).borders(BorderStyle::Single, 4, "CCCCCC");
    if let Some(mut c) = table.cell(0, 0) { c.set_text("Concept"); }
    if let Some(mut c) = table.cell(0, 1) { c.set_text("Description"); }
    if let Some(mut c) = table.cell(0, 2) { c.set_text("Example"); }
    if let Some(mut c) = table.cell(1, 0) { c.set_text("Tool"); }
    if let Some(mut c) = table.cell(1, 1) { c.set_text("A function the server exposes"); }
    if let Some(mut c) = table.cell(1, 2) { c.set_text("read_file, search_code"); }
    if let Some(mut c) = table.cell(2, 0) { c.set_text("Resource"); }
    if let Some(mut c) = table.cell(2, 1) { c.set_text("Data the server provides"); }
    if let Some(mut c) = table.cell(2, 2) { c.set_text("file://, git://"); }
    if let Some(mut c) = table.cell(3, 0) { c.set_text("Prompt"); }
    if let Some(mut c) = table.cell(3, 1) { c.set_text("Reusable prompt templates"); }
    if let Some(mut c) = table.cell(3, 2) { c.set_text("code_review, summarize"); }
    if let Some(mut c) = table.cell(4, 0) { c.set_text("Transport"); }
    if let Some(mut c) = table.cell(4, 1) { c.set_text("Communication channel"); }
    if let Some(mut c) = table.cell(4, 2) { c.set_text("stdio, HTTP+SSE"); }
    table.header_row_style("2E4057", "FFFFFF");
    table.banded_rows("F0F4F8");

    // === CODE EXAMPLE ===
    let mut p = doc.add_paragraph("");
    p = p.space_before(Length::pt(18.0)).outline_level(1);
    p.add_run("Your First MCP Server").font("Garamond").size(14.0).bold(true);
    p.bookmark(3, "first_server");

    let mut p = doc.add_paragraph("");
    p = p.first_line_indent(Length::inches(0.3));
    p.add_run("Let's build a minimal MCP server that exposes a single tool. Create a new Rust project and add the following to your ").font("Garamond").size(11.0);
    p.add_run("Cargo.toml").font("Courier New").size(10.0);
    p.add_run(":").font("Garamond").size(11.0);

    let code = r#"use rmcp::{tool, tool_router, ServiceExt, transport::stdio};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::schemars;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GreetInput { pub name: String }

#[derive(Clone)]
pub struct MyServer;

#[tool_router(server_handler)]
impl MyServer {
    #[tool(description = "Greet someone by name")]
    fn greet(&self, Parameters(input): Parameters<GreetInput>) -> String {
        format!("Hello, {}!", input.name)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    MyServer.serve(stdio()).await?.waiting().await?;
    Ok(())
}"#;
    let idx = doc.content_count();
    engine::insert_code_block(&mut doc, idx, code, Some("rust"));

    // Callout
    let idx = doc.content_count();
    engine::insert_callout(&mut doc, idx, "tip", "The #[tool_router(server_handler)] macro generates both the tool router AND the ServerHandler impl automatically. Use #[tool_router] alone when you need to implement resources or prompts manually.");

    // Warning callout
    let idx = doc.content_count();
    engine::insert_callout(&mut doc, idx, "warning", "Always use schemars 1.x with rmcp 1.7 — not 0.8. The macro-generated schemas won't compile with the wrong version.");

    // === CHAPTER 2 TEASER ===
    let mut p = doc.add_paragraph("");
    p = p.page_break_before(true).alignment(Alignment::Center).space_before(Length::inches(2.0));
    p.add_run("CHAPTER TWO").font("Garamond").size(12.0).small_caps(true).theme_color("accent1");

    let mut p = doc.add_paragraph("");
    p = p.alignment(Alignment::Center).space_after(Length::pt(36.0)).outline_level(0);
    p.add_run("Transport & Lifecycle").font("Garamond").size(24.0).bold(true);

    let mut p = doc.add_paragraph("");
    p = p.first_line_indent(Length::inches(0.0));
    p.add_run("In this chapter, we'll explore the two transport mechanisms MCP supports: stdio (for local tools) and Streamable HTTP (for remote servers). We'll implement both and discuss when to use each.").font("Garamond").size(11.0);

    // Hyperlink
    let rel = doc.add_hyperlink_rel("https://modelcontextprotocol.io/specification/2025-03-26/basic/transports");
    let mut p = doc.add_paragraph("");
    p = p.space_before(Length::pt(12.0));
    p.add_run("Further reading: ").font("Garamond").size(10.0).italic(true);
    p.add_hyperlink_run("MCP Transport Specification", Some(&rel), None)
        .color("2980B9").underline(true).size(10.0);

    // Save
    doc.save("/Users/jameskaranja/Downloads/kdp_technical_sample.docx").unwrap();
    println!("✓ Saved ~/Downloads/kdp_technical_sample.docx");
}
