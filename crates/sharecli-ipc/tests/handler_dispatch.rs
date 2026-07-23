//! FR-003 / C01 — sharecli-ipc JSON-RPC handler dispatch coverage.

use sharecli_ipc::handler::Handler;

#[tokio::test]
async fn fr003_ipc_handler_process_list_and_health() {
    let handler = Handler::new().await.expect("handler init");

    let list_resp = handler.dispatch(r#"{"id":1,"method":"process.list","params":{}}"#).await;
    assert!(list_resp.error.is_none(), "process.list error: {:?}", list_resp.error);
    assert!(list_resp.result.is_array());

    let health_resp = handler.dispatch(r#"{"id":2,"method":"health.status","params":{}}"#).await;
    assert!(health_resp.error.is_none(), "health.status error: {:?}", health_resp.error);
    assert!(health_resp.result.get("healthy").is_some());
}

#[tokio::test]
async fn fr003_ipc_handler_config_get_and_unknown_method() {
    let handler = Handler::new().await.expect("handler init");

    let cfg_resp = handler.dispatch(r#"{"id":3,"method":"config.get","params":{}}"#).await;
    assert!(cfg_resp.error.is_none());
    assert!(cfg_resp.result.get("projects").is_some());

    let bad_resp = handler.dispatch(r#"{"id":4,"method":"nope.method","params":{}}"#).await;
    assert!(bad_resp.error.is_some());
    assert!(bad_resp.error.unwrap().contains("unknown method"));
}

#[tokio::test]
async fn fr003_ipc_handler_rejects_invalid_json() {
    let handler = Handler::new().await.expect("handler init");
    let resp = handler.dispatch("not-json").await;
    assert!(resp.error.is_some());
    assert!(resp.error.unwrap().contains("parse error"));
}

#[tokio::test]
async fn fr003_ipc_handler_config_set_and_process_kill_all() {
    let handler = Handler::new().await.expect("handler init");

    let set_resp = handler
        .dispatch(
            r#"{"id":6,"method":"config.set","params":{"key":"pool.max_per_type","value":3}}"#,
        )
        .await;
    assert!(set_resp.error.is_none(), "config.set error: {:?}", set_resp.error);
    assert_eq!(set_resp.result, serde_json::json!(true));

    let get_resp = handler.dispatch(r#"{"id":7,"method":"config.get","params":{}}"#).await;
    assert!(get_resp.error.is_none());
    assert_eq!(get_resp.result["pool"]["max_per_type"], 3);

    let kill_all = handler.dispatch(r#"{"id":8,"method":"process.kill_all","params":{}}"#).await;
    assert!(kill_all.error.is_none(), "process.kill_all error: {:?}", kill_all.error);
    assert_eq!(kill_all.result, serde_json::json!(true));
}

#[tokio::test]
async fn fr003_ipc_handler_process_kill_missing_pid() {
    let handler = Handler::new().await.expect("handler init");
    let resp = handler.dispatch(r#"{"id":9,"method":"process.kill","params":{}}"#).await;
    assert!(resp.error.is_some());
}

#[tokio::test]
async fn fr003_ipc_handler_monitoring_report() {
    let handler = Handler::new().await.expect("handler init");
    let resp = handler.dispatch(r#"{"id":10,"method":"monitoring.report","params":{}}"#).await;
    assert!(resp.error.is_none(), "monitoring.report error: {:?}", resp.error);
    assert!(resp.result.is_object());
}
