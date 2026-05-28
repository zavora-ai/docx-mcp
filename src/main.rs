#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use rmcp::{ServiceExt, transport::stdio};
    docx_mcp_server::DocxServer::new().serve(stdio()).await?.waiting().await?;
    Ok(())
}
