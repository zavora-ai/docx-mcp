# docx-mcp-server

[![Crates.io](https://img.shields.io/crates/v/docx-mcp-server.svg)](https://crates.io/crates/docx-mcp-server)
[![Docs.rs](https://docs.rs/docx-mcp-server/badge.svg)](https://docs.rs/docx-mcp-server)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)

80 MCP tools for creating, reading, editing, formatting, and converting Microsoft Word (.docx) documents — plus KDP book layouts and a library of 21 parameterized business templates. Pure Rust, local-first, no Microsoft Office required.

## Install

```bash
cargo install docx-mcp-server
```

## Configure

```json
{
  "mcpServers": {
    "docx": {
      "command": "docx-mcp-server"
    }
  }
}
```

## Highlights

- **80 tools** spanning document lifecycle, reading, writing, formatting, tables, media, math/charts/shapes, and export to PDF / HTML / Markdown / plain text.
- **KDP book formats** — `create_document` accepts `kdp:technical`, `kdp:novel`, `kdp:cookbook`, `kdp:children`, `kdp:interior_design`, `kdp:encyclopedia`, `kdp:manga`.
- **21 business templates** — publication-grade, fully parameterized via a `data` object (see below).
- **Discovery** — `list_templates` returns every template's structured `data_fields` (name + type + item shape) and the universal `style_params`, so an agent can learn the schema programmatically.

## Business templates

`create_document` accepts a `business:*` format plus an optional `data` object that fills the template (missing keys keep a placeholder, so it also works as a blank scaffold). Priced documents auto-sum their line-item totals.

```
report · resume · letter · memo · invoice · newsletter · academic ·
proposal · agenda · press_release · certificate · cover_letter ·
fax_cover · quote · purchase_order · receipt · flyer · contract ·
meeting_minutes · sign_in_sheet · business_plan
```

Every template also accepts universal styling keys: `accent` (hex brand color), `logo` (image path), `logo_align`, `logo_height`, `heading_font`, `body_font`.

```jsonc
// create_document
{
  "format": "business:invoice",
  "data": {
    "accent": "#1F6F54",
    "logo": "/path/to/logo.png",
    "company": "Northwind Studio",
    "number": "INV-2048",
    "bill_to": "Acme Corp",
    "items": [
      { "description": "Brand identity design", "qty": 1, "price": 4500 },
      { "description": "Website UI (12 screens)", "qty": 12, "price": 350 }
    ]
  }
}
// → TOTAL auto-summed to $8,700.00
```

Call `list_templates` to discover the accepted `data_fields` and `style_params` for every format.

## Tool groups

| Group | Examples |
|------|----------|
| Lifecycle | `create_document`, `open_document`, `save_document`, `close_document`, `describe_document`, `list_templates` |
| Read | `read_paragraphs`, `read_paragraph`, `read_table`, `search_text` |
| Write | `insert_paragraph`, `replace_text`, `delete_content`, `insert_run` |
| Format | `set_run_format`, `set_paragraph_format`, `apply_style` |
| Tables | `add_table`, `set_table_cell`, `add_table_row`, `merge_table_cells` |
| Media & layout | `add_image`, `add_page_background`, `add_list`, `add_section_break`, `set_header_footer` |
| Rich content | `add_equation`, `add_chart`, `add_shape`, `add_content_control`, `embed_font`, `add_building_block`, `add_custom_xml` |
| Export | `to_plain_text`, `to_markdown`, `to_html`, `save_pdf`, `render_page` |

The authoritative tool list with risk classes lives in [`mcp-server.toml`](mcp-server.toml).

## Architecture

- **In-memory document store** — LRU eviction (100 docs) + TTL expiry (1 hour)
- **UUID handles** — documents referenced by handle, not file path
- **Pure Rust** — built on [`zavora-docx`](https://crates.io/crates/zavora-docx) for OOXML, layout, and PDF/HTML/Markdown rendering
- **No system deps** — no LibreOffice, no Microsoft Office

## Example workflow

```
Agent: list_templates()
  → discovers business:invoice accepts items[{description,qty,price,amount?}]

Agent: create_document(format: "business:invoice", data: { company: "...", items: [...] })
  → handle: "abc-123..."

Agent: render_page(document_handle: "abc-123", page_index: 0, output_path: "/tmp/preview.png")
Agent: save_document(document_handle: "abc-123", output_path: "/invoices/inv-2048.docx")
```

## License

Apache-2.0

---

Document engine: [`zavora-docx`](https://crates.io/crates/zavora-docx), itself built on [rdocx](https://github.com/tensorbee/rdocx) by Atul Sharma.

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem. Built with ❤️ by [Zavora AI](https://zavora.ai)
