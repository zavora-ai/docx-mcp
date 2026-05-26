//! # docx-mcp-server
//!
//! A Model Context Protocol (MCP) server providing 29 tools for creating, reading,
//! editing, formatting, and converting Microsoft Word (.docx) documents.
//!
//! ## Overview
//!
//! Gives AI agents full DOCX manipulation capabilities — create documents from scratch,
//! open existing files, edit paragraphs, format text, manage tables, insert images,
//! and export to plain text, Markdown, or HTML.
//!
//! ## Installation
//!
//! ```bash
//! cargo install docx-mcp-server
//! ```
//!
//! ## Configuration
//!
//! ```json
//! {
//!   "mcpServers": {
//!     "docx": {
//!       "command": "docx-mcp-server"
//!     }
//!   }
//! }
//! ```
//!
//! ## Tools (29)
//!
//! | Category | Tools |
//! |----------|-------|
//! | Lifecycle | create, open, save, close, describe |
//! | Read | paragraphs, paragraph, table, search |
//! | Write | insert_paragraph, replace_text, delete, insert_run, update_paragraph, batch_write |
//! | Format | set_run_format, set_paragraph_format, apply_style |
//! | Tables | add_table, set_table_cell, add_table_row, merge_table_cells |
//! | Media | add_image, add_list, add_section_break, set_header_footer |
//! | Export | to_plain_text, to_markdown, to_html |
//!
//! ## Architecture
//!
//! - **In-memory document store** with LRU eviction and TTL expiry
//! - **Document references** via UUID handles (no file paths in tool responses)
//! - **Pure Rust** — uses [`docx-rs`](https://crates.io/crates/docx-rs) for OOXML manipulation

/// Custom error types.
pub mod error;

/// Input/output type definitions for MCP tools.
pub mod types;

/// In-memory document store with LRU eviction.
pub mod store;

/// Document reference handles (UUID-based).
pub mod doc_ref;

/// Core DOCX manipulation engine.
pub mod engine;

/// MCP server with tool routing.
pub mod server;

/// Tool handler implementations by category.
pub mod tools;
