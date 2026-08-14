#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rmcp::{transport::stdio, ServiceExt};
    docx_mcp_server::DocxServer::new()
        .serve(stdio())
        .await?
        .waiting()
        .await?;
    Ok(())
}
