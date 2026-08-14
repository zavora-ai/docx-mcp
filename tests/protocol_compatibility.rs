use docx_mcp_server::DocxServer;
use rmcp::{
    model::{
        CacheScope, CallToolRequestParams, CallToolResponse, ClientCapabilities, ClientInfo,
        GetTaskParams, Implementation, ProtocolVersion, TaskPayload,
    },
    ClientHandler, ServiceExt,
};

#[derive(Clone)]
struct LegacyClient;

impl ClientHandler for LegacyClient {
    fn get_info(&self) -> ClientInfo {
        let mut info = ClientInfo::new(
            ClientCapabilities::default(),
            Implementation::new("docx-legacy-test", "1"),
        );
        info.protocol_version = ProtocolVersion::V_2025_11_25;
        info
    }
}

#[derive(Clone)]
struct CurrentClient;

impl ClientHandler for CurrentClient {
    fn get_info(&self) -> ClientInfo {
        let mut info = ClientInfo::new(
            ClientCapabilities::builder().enable_tasks().build(),
            Implementation::new("docx-current-test", "1"),
        );
        info.protocol_version = ProtocolVersion::V_2026_07_28;
        info
    }
}

#[tokio::test]
async fn legacy_protocol_lists_all_tools() {
    let (server_transport, client_transport) = tokio::io::duplex(256 * 1024);
    let server = tokio::spawn(async move {
        DocxServer::new()
            .serve(server_transport)
            .await
            .unwrap()
            .waiting()
            .await
            .unwrap();
    });

    let client = LegacyClient.serve(client_transport).await.unwrap();
    assert_eq!(client.list_tools(None).await.unwrap().tools.len(), 88);
    client.cancel().await.unwrap();
    server.await.unwrap();
}

#[tokio::test]
async fn current_protocol_advertises_tasks_and_cache_hints() {
    let (server_transport, client_transport) = tokio::io::duplex(256 * 1024);
    let server = tokio::spawn(async move {
        DocxServer::new()
            .serve(server_transport)
            .await
            .unwrap()
            .waiting()
            .await
            .unwrap();
    });

    let client = CurrentClient.serve(client_transport).await.unwrap();
    assert!(client
        .peer_info()
        .expect("server peer info")
        .capabilities
        .supports_tasks());
    let tools = client.list_tools(None).await.unwrap();
    assert_eq!(tools.tools.len(), 88);
    assert_eq!(tools.ttl_ms, Some(60_000));
    assert_eq!(tools.cache_scope, Some(CacheScope::Public));

    let response = client
        .call_tool_once(
            CallToolRequestParams::new("insert_run").with_arguments(
                serde_json::json!({
                    "document_handle": "missing",
                    "paragraph_index": 0,
                    "text": "task smoke test"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await
        .unwrap();
    let created = match response {
        CallToolResponse::Task(created) => created,
        other => panic!("expected task response, got {other:?}"),
    };
    loop {
        let task = client
            .peer()
            .get_task(GetTaskParams::new(created.task.task_id.clone()))
            .await
            .unwrap()
            .task;
        if task.status().is_terminal() {
            assert!(matches!(task.payload, TaskPayload::Completed { .. }));
            break;
        }
        tokio::task::yield_now().await;
    }
    client.cancel().await.unwrap();
    server.await.unwrap();
}
