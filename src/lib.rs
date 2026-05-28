//! docx-mcp-server — DOCX MCP Server with professional book formatting.
//!
//! 32 tools for creating, reading, editing, formatting, and converting Word documents.
//! Powered by zavora-docx for paragraph shading, borders, syntax highlighting, and PDF export.

pub mod engine;
pub mod server;

pub use server::DocxServer;
