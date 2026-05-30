use zavora_docx::{Document, Length, Alignment, BorderStyle};
use docx_mcp_server::engine;

fn main() {
    let mut doc = Document::new();
    engine::create_kdp_technical(&mut doc);
    doc.set_title("Building MCP Servers in Rust");
    doc.set_author("James Karanja");

    // === TITLE PAGE === (named styles: Title + Subtitle)
    let mut p = doc.add_paragraph("");
    p = p.style("Title").alignment(Alignment::Center).space_before(Length::inches(2.5));
    p.add_run("Building MCP Servers in Rust");

    let mut p = doc.add_paragraph("");
    p = p.style("Subtitle").alignment(Alignment::Center);
    p.add_run("A Practical Guide to the Model Context Protocol");

    let mut p = doc.add_paragraph("");
    p = p.alignment(Alignment::Center).space_before(Length::inches(1.5));
    p.add_run("James Karanja").size(16.0);

    // === COPYRIGHT PAGE === (Caption style for fine print)
    for line in [
        "Copyright © 2026 James Karanja. All rights reserved.",
        "Published by Zavora Press, Nairobi",
        "ISBN: 978-1-234567-89-0",
        "First Edition, May 2026",
    ] {
        let mut p = doc.add_paragraph("");
        if line.starts_with("Copyright") { p = p.page_break_before(true).space_before(Length::inches(5.0)); }
        p = p.style("Caption");
        p.add_run(line);
    }

    // === TABLE OF CONTENTS ===
    // === TABLE OF CONTENTS === (page break; insert_toc adds its own title + entries)
    let mut p = doc.add_paragraph("");
    p = p.page_break_before(true);
    p.add_run("");
    let toc_index = doc.content_count();

    // === CHAPTER 1 === (Heading1 — drives navigation pane + TOC)
    let mut p = doc.add_paragraph("");
    p = p.style("Heading1").page_break_before(true);
    p.add_run("Introduction to MCP");
    p.bookmark(1, "chapter1");

    // Drop cap opening (Normal body)
    let mut p = doc.add_paragraph("");
    p = p.drop_cap(3);
    p.add_run("T").size(48.0);
    let mut p = doc.add_paragraph("");
    p.add_run("he Model Context Protocol represents a fundamental shift in how AI agents interact with external tools and data sources. Rather than building monolithic applications, MCP enables a composable architecture where servers expose capabilities and clients consume them through a standardized JSON-RPC interface.");

    // Footnote
    let fn1 = doc.add_footnote("The MCP specification was first published in November 2024 by Anthropic. See https://modelcontextprotocol.io for the full specification.");
    let mut p = doc.add_paragraph("");
    p = p.first_line_indent(Length::inches(0.3));
    p.add_run("This book will guide you through building production-grade MCP servers using Rust and the rmcp crate");
    p.add_run("").footnote_ref(fn1);
    p.add_run(". By the end, you'll have the skills to create servers that handle thousands of concurrent connections with minimal resource usage.");

    // === SECTION: What is MCP? === (Heading2)
    let mut p = doc.add_paragraph("");
    p = p.style("Heading2");
    p.add_run("What is MCP?");
    p.bookmark(2, "what_is_mcp");

    let mut p = doc.add_paragraph("");
    p = p.first_line_indent(Length::inches(0.3));
    p.add_run("MCP is a protocol that standardizes how AI applications communicate with external data sources and tools. Think of it as USB for AI — a universal connector that lets any AI model talk to any tool.");

    // Pull quote using Quote style
    let mut p = doc.add_paragraph("");
    p = p.style("Quote");
    p.add_run("The best protocols are invisible — they just work.");

    // Caption for table
    let mut p = doc.add_paragraph("");
    p = p.style("Caption");
    p.add_run("Table 1.1: Core MCP Concepts");

    let mut table = doc.add_table(5, 3);
    table = table.width_pct(100.0).borders(BorderStyle::Single, 4, "CCCCCC");
    let rows = [
        ("Concept", "Description", "Example"),
        ("Tool", "A function the server exposes", "read_file, search_code"),
        ("Resource", "Data the server provides", "file://, git://"),
        ("Prompt", "Reusable prompt templates", "code_review, summarize"),
        ("Transport", "Communication channel", "stdio, HTTP+SSE"),
    ];
    for (r, (a, b, c)) in rows.iter().enumerate() {
        if let Some(mut cell) = table.cell(r, 0) { cell.set_text(a); }
        if let Some(mut cell) = table.cell(r, 1) { cell.set_text(b); }
        if let Some(mut cell) = table.cell(r, 2) { cell.set_text(c); }
    }
    table.header_row_style("2E4057", "FFFFFF");
    table.banded_rows("F0F4F8");

    // === Code section === (Heading2)
    let mut p = doc.add_paragraph("");
    p = p.style("Heading2");
    p.add_run("Your First MCP Server");
    p.bookmark(3, "first_server");

    let mut p = doc.add_paragraph("");
    p = p.first_line_indent(Length::inches(0.3));
    p.add_run("Let's build a minimal MCP server that exposes a single tool. Add the following to your ");
    p.add_run("Cargo.toml").font("Courier New").size(10.0);
    p.add_run(":");

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

    let idx = doc.content_count();
    engine::insert_callout(&mut doc, idx, "tip", "The #[tool_router(server_handler)] macro generates both the tool router AND the ServerHandler impl automatically.");
    let idx = doc.content_count();
    engine::insert_callout(&mut doc, idx, "warning", "Always use schemars 1.x with rmcp 1.7 — not 0.8. The macro-generated schemas won't compile with the wrong version.");

    // === CHAPTER 2 === (Heading1)
    let mut p = doc.add_paragraph("");
    p = p.style("Heading1").page_break_before(true);
    p.add_run("Transport & Lifecycle");

    let mut p = doc.add_paragraph("");
    p.add_run("In this chapter, we'll explore the two transport mechanisms MCP supports: stdio (for local tools) and Streamable HTTP (for remote servers).");

    let rel = doc.add_hyperlink_rel("https://modelcontextprotocol.io/specification/2025-03-26/basic/transports");
    let mut p = doc.add_paragraph("");
    p = p.style("Caption");
    p.add_run("Further reading: ");
    p.add_hyperlink_run("MCP Transport Specification", Some(&rel), None).color("2980B9").underline(true);

    // Insert TOC now that all headings exist (scanned from document body)
    doc.insert_toc(toc_index, 2);

    doc.save("/Users/jameskaranja/Downloads/kdp_technical_sample.docx").unwrap();
    println!("✓ Saved ~/Downloads/kdp_technical_sample.docx");
}
