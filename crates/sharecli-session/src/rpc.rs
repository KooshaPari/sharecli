use crate::SessionService;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}
#[derive(Debug, Serialize)]
pub struct Response {
    pub id: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub async fn dispatch(service: Arc<SessionService>, line: &str) -> Response {
    let request = match serde_json::from_str::<Request>(line) {
        Ok(value) => value,
        Err(error) => {
            return Response {
                id: serde_json::Value::Null,
                result: None,
                error: Some(error.to_string()),
            }
        }
    };
    let outcome = match request.method.as_str() {
        "session.list" => service.list().map(|v| serde_json::to_value(v).unwrap_or_default()),
        "session.inspect" => request
            .params
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("params.id is required"))
            .and_then(|id| {
                service.inspect(id).map(|v| serde_json::to_value(v).unwrap_or_default())
            }),
        "recovery.plan" => {
            service.recovery_plan().map(|v| serde_json::to_value(v).unwrap_or_default())
        }
        method => Err(anyhow::anyhow!("unknown method: {method}")),
    };
    match outcome {
        Ok(result) => Response { id: request.id, result: Some(result), error: None },
        Err(error) => Response { id: request.id, result: None, error: Some(error.to_string()) },
    }
}

#[cfg(unix)]
pub async fn serve_unix(path: &std::path::Path, service: Arc<SessionService>) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let _ = std::fs::remove_file(path);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let listener = tokio::net::UnixListener::bind(path)?;
    loop {
        let (stream, _) = listener.accept().await?;
        let service = service.clone();
        tokio::spawn(async move {
            let (reader, mut writer) = stream.into_split();
            let mut lines = BufReader::new(reader).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(mut payload) =
                    serde_json::to_string(&dispatch(service.clone(), &line).await)
                {
                    payload.push('\n');
                    if writer.write_all(payload.as_bytes()).await.is_err() {
                        break;
                    }
                }
            }
        });
    }
}
