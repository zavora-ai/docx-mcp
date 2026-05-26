use docx_mcp_server::server::DocxMcpServer;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting docx-mcp-server...");

    if let Err(e) = run().await {
        tracing::error!("Server error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let server = DocxMcpServer::new();
    let transport = stdio();
    let server = server.serve(transport).await?;
    server.waiting().await?;
    Ok(())
}
