# docx-mcp-server

[![Crates.io](https://img.shields.io/crates/v/docx-mcp-server.svg)](https://crates.io/crates/docx-mcp-server)
[![Docs.rs](https://docs.rs/docx-mcp-server/badge.svg)](https://docs.rs/docx-mcp-server)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)

29 MCP tools for creating, reading, editing, formatting, and converting Microsoft Word (.docx) documents. Pure Rust, local-first, no Microsoft Office required.

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

## Tools (29)

### Document Lifecycle
| Tool | Description |
|------|-------------|
| `create_document` | Create a new empty DOCX in memory |
| `open_document` | Open an existing .docx file from disk |
| `save_document` | Save document to disk as .docx |
| `close_document` | Close and free memory |
| `describe_document` | Structural overview (paragraphs, tables, children) |

### Read
| Tool | Description |
|------|-------------|
| `read_paragraphs` | Paginated paragraph listing |
| `read_paragraph` | Single paragraph with full detail (runs, formatting) |
| `read_table` | Table content as structured rows/cells |
| `search_text` | Search across paragraphs (exact, substring, regex) |

### Write
| Tool | Description |
|------|-------------|
| `insert_paragraph` | Insert paragraph at position with optional style |
| `replace_text` | Find and replace across all paragraphs |
| `delete_content` | Delete paragraph, table, or run by index |
| `insert_run` | Add formatted text run to existing paragraph |
| `update_paragraph_text` | Replace entire paragraph text |
| `batch_write` | Multiple write operations in one call |

### Format
| Tool | Description |
|------|-------------|
| `set_run_format` | Bold, italic, underline, font, size, color |
| `set_paragraph_format` | Alignment, spacing, indentation, heading level |
| `apply_style` | Apply named style to paragraph or run |

### Tables
| Tool | Description |
|------|-------------|
| `add_table` | Insert table with rows × columns |
| `set_table_cell` | Write text to specific cell |
| `add_table_row` | Append row to existing table |
| `merge_table_cells` | Merge cells horizontally or vertically |

### Media & Layout
| Tool | Description |
|------|-------------|
| `add_image` | Insert image from file |
| `add_list` | Create bulleted or numbered list |
| `add_section_break` | Section break with page layout |
| `set_header_footer` | Header/footer content per section |

### Export
| Tool | Description |
|------|-------------|
| `to_plain_text` | Export as plain text |
| `to_markdown` | Export as Markdown |
| `to_html` | Export as HTML fragment |

## Architecture

- **In-memory document store** — LRU eviction (100 docs) + TTL expiry (1 hour)
- **UUID handles** — documents referenced by handle, not file path
- **Batch operations** — `batch_write` executes multiple edits atomically
- **Pure Rust** — uses [docx-rs](https://crates.io/crates/docx-rs) for OOXML manipulation
- **No system deps** — no LibreOffice, no Microsoft Office

## Example Workflow

```
Agent: create_document(title: "Quarterly Report")
  → handle: "abc-123..."

Agent: insert_paragraph(doc_id: "abc-123", index: 0, text: "Q1 2026 Report", style: "Heading1")
Agent: insert_paragraph(doc_id: "abc-123", index: 1, text: "Revenue grew 45% YoY...")
Agent: add_table(doc_id: "abc-123", rows: 4, cols: 3, index: 2)
Agent: set_table_cell(doc_id: "abc-123", table_index: 0, row: 0, col: 0, text: "Metric")

Agent: save_document(doc_id: "abc-123", path: "/reports/q1-2026.docx")
Agent: to_markdown(doc_id: "abc-123")
  → "# Q1 2026 Report\n\nRevenue grew 45% YoY..."
```

## License

Apache-2.0

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)
