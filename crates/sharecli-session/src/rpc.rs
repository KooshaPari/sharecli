use crate::{SessionService, SurfaceCapabilities, SurfaceRecord};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

const JSON_RPC_VERSION: &str = "2.0";
const MAX_SURFACE_READ_BYTES: usize = 1024 * 1024;

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

/// Shell-free operations exposed by the surface control socket.
pub trait SurfaceControl: Send + Sync {
    /// Enumerate stable terminal surface identities.
    fn list(&self) -> Result<Vec<SurfaceRecord>> {
        anyhow::bail!("surface discovery unavailable")
    }
    fn send(&self, surface_id: &str, bytes: &[u8]) -> Result<()>;
    fn read(&self, surface_id: &str, max_bytes: usize) -> Result<Vec<u8>>;
    fn resize(&self, surface_id: &str, rows: u16, cols: u16) -> Result<()>;
    fn capabilities(&self, surface_id: &str) -> Result<SurfaceCapabilities>;
}

#[derive(Debug, Deserialize)]
struct SurfaceRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SurfaceResponse {
    pub jsonrpc: &'static str,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Deserialize)]
struct SurfaceIdParams {
    surface_id: String,
}

#[derive(Deserialize)]
struct SendParams {
    surface_id: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    bytes: Option<Vec<u8>>,
}

#[derive(Deserialize)]
struct ReadParams {
    surface_id: String,
    max_bytes: usize,
}

#[derive(Deserialize)]
struct ResizeParams {
    surface_id: String,
    rows: u16,
    cols: u16,
}

#[derive(Serialize)]
struct ReadResult {
    bytes: Vec<u8>,
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

/// Dispatch one JSON-RPC 2.0 surface request.
///
/// Input text is passed directly to [`SurfaceControl::send`]. It is never
/// interpreted as a command line or evaluated by a shell.
pub async fn dispatch_surface(control: Arc<dyn SurfaceControl>, line: &str) -> SurfaceResponse {
    let request = match serde_json::from_str::<SurfaceRequest>(line) {
        Ok(request) => request,
        Err(error) => {
            return surface_error(Value::Null, -32700, format!("parse error: {error}"));
        }
    };
    if request.jsonrpc != JSON_RPC_VERSION {
        return surface_error(request.id, -32600, "jsonrpc must be \"2.0\"".to_string());
    }

    let id = request.id;
    let outcome = dispatch_surface_method(control.as_ref(), &request.method, request.params);
    match outcome {
        Ok(result) => {
            SurfaceResponse { jsonrpc: JSON_RPC_VERSION, id, result: Some(result), error: None }
        }
        Err((code, message)) => surface_error(id, code, message),
    }
}

fn dispatch_surface_method(
    control: &dyn SurfaceControl,
    method: &str,
    params: Value,
) -> std::result::Result<Value, (i32, String)> {
    match method {
        "surface.list" => {
            let surfaces = control.list().map_err(control_error)?;
            serde_json::to_value(surfaces).map_err(control_error)
        }
        "surface.io.send" => {
            let params: SendParams = decode_params(params)?;
            let bytes = match (params.text, params.bytes) {
                (Some(text), None) => text.into_bytes(),
                (None, Some(bytes)) => bytes,
                _ => {
                    return Err((
                        -32602,
                        "exactly one of params.text or params.bytes is required".to_string(),
                    ))
                }
            };
            control.send(&params.surface_id, &bytes).map_err(control_error)?;
            Ok(Value::Null)
        }
        "surface.io.read" => {
            let params: ReadParams = decode_params(params)?;
            if params.max_bytes > MAX_SURFACE_READ_BYTES {
                return Err((
                    -32602,
                    format!("max_bytes must not exceed {MAX_SURFACE_READ_BYTES}"),
                ));
            }
            let bytes =
                control.read(&params.surface_id, params.max_bytes).map_err(control_error)?;
            serde_json::to_value(ReadResult { bytes }).map_err(control_error)
        }
        "surface.io.resize" => {
            let params: ResizeParams = decode_params(params)?;
            if params.rows == 0 || params.cols == 0 {
                return Err((-32602, "rows and cols must be greater than zero".to_string()));
            }
            control.resize(&params.surface_id, params.rows, params.cols).map_err(control_error)?;
            Ok(Value::Null)
        }
        "surface.io.capabilities" => {
            let params: SurfaceIdParams = decode_params(params)?;
            let capabilities = control.capabilities(&params.surface_id).map_err(control_error)?;
            serde_json::to_value(capabilities).map_err(control_error)
        }
        _ => Err((-32601, format!("unknown method: {method}"))),
    }
}

fn decode_params<T: for<'de> Deserialize<'de>>(
    params: Value,
) -> std::result::Result<T, (i32, String)> {
    serde_json::from_value(params).map_err(|error| (-32602, format!("invalid params: {error}")))
}

fn control_error(error: impl std::fmt::Display) -> (i32, String) {
    (-32000, error.to_string())
}

fn surface_error(id: Value, code: i32, message: String) -> SurfaceResponse {
    SurfaceResponse {
        jsonrpc: JSON_RPC_VERSION,
        id,
        result: None,
        error: Some(RpcError { code, message }),
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

/// Serve shell-free surface control requests over a newline-delimited Unix socket.
#[cfg(unix)]
pub async fn serve_surface_unix(
    path: &std::path::Path,
    control: Arc<dyn SurfaceControl>,
) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let _ = std::fs::remove_file(path);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let listener = tokio::net::UnixListener::bind(path)?;
    loop {
        let (stream, _) = listener.accept().await?;
        let control = control.clone();
        tokio::spawn(async move {
            let (reader, mut writer) = stream.into_split();
            let mut lines = BufReader::new(reader).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let response = dispatch_surface(control.clone(), &line).await;
                let Ok(mut payload) = serde_json::to_string(&response) else {
                    break;
                };
                payload.push('\n');
                if writer.write_all(payload.as_bytes()).await.is_err() {
                    break;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SurfaceCapabilities;
    use serde_json::json;
    use std::sync::Mutex;

    #[derive(Default)]
    struct RecordingControl {
        sent: Mutex<Vec<(String, Vec<u8>)>>,
        resized: Mutex<Vec<(String, u16, u16)>>,
    }

    impl SurfaceControl for RecordingControl {
        fn send(&self, surface_id: &str, bytes: &[u8]) -> Result<()> {
            self.sent.lock().unwrap().push((surface_id.to_string(), bytes.to_vec()));
            Ok(())
        }

        fn read(&self, surface_id: &str, max_bytes: usize) -> Result<Vec<u8>> {
            assert_eq!(surface_id, "surface-1");
            Ok(b"terminal output"[..max_bytes.min(15)].to_vec())
        }

        fn resize(&self, surface_id: &str, rows: u16, cols: u16) -> Result<()> {
            self.resized.lock().unwrap().push((surface_id.to_string(), rows, cols));
            Ok(())
        }

        fn capabilities(&self, surface_id: &str) -> Result<SurfaceCapabilities> {
            assert_eq!(surface_id, "surface-1");
            Ok(SurfaceCapabilities {
                read: true,
                write: true,
                resize: true,
                layout: false,
                durable_pty: true,
            })
        }
    }

    async fn request(
        control: &Arc<RecordingControl>,
        id: u64,
        method: &str,
        params: Value,
    ) -> SurfaceResponse {
        let raw = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        })
        .to_string();
        dispatch_surface(control.clone(), &raw).await
    }

    #[tokio::test]
    async fn surface_send_passes_text_directly_to_control() {
        let control = Arc::new(RecordingControl::default());
        let response = request(
            &control,
            1,
            "surface.io.send",
            json!({"surface_id": "surface-1", "text": "printf 'not a shell'"}),
        )
        .await;

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.result, Some(Value::Null));
        assert!(response.error.is_none());
        assert_eq!(
            *control.sent.lock().unwrap(),
            vec![("surface-1".to_string(), b"printf 'not a shell'".to_vec())]
        );
    }

    #[tokio::test]
    async fn surface_list_reports_degraded_discovery_without_a_native_adapter() {
        let control = Arc::new(RecordingControl::default());
        let response = request(&control, 7, "surface.list", json!({})).await;
        assert!(response.result.is_none());
        assert_eq!(response.error.unwrap().code, -32000);
    }

    #[tokio::test]
    async fn surface_send_accepts_an_explicit_byte_vector() {
        let control = Arc::new(RecordingControl::default());
        let response = request(
            &control,
            6,
            "surface.io.send",
            json!({"surface_id": "surface-1", "bytes": [0, 255, 10]}),
        )
        .await;

        assert_eq!(response.result, Some(Value::Null));
        assert_eq!(
            *control.sent.lock().unwrap(),
            vec![("surface-1".to_string(), vec![0, 255, 10])]
        );
    }

    #[tokio::test]
    async fn surface_read_returns_a_typed_byte_vector() {
        let control = Arc::new(RecordingControl::default());
        let response = request(
            &control,
            2,
            "surface.io.read",
            json!({"surface_id": "surface-1", "max_bytes": 8}),
        )
        .await;

        assert_eq!(response.result, Some(json!({"bytes": b"terminal".to_vec()})));
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn surface_resize_forwards_dimensions_without_a_command_string() {
        let control = Arc::new(RecordingControl::default());
        let response = request(
            &control,
            3,
            "surface.io.resize",
            json!({"surface_id": "surface-1", "rows": 42, "cols": 120}),
        )
        .await;

        assert_eq!(response.result, Some(Value::Null));
        assert_eq!(*control.resized.lock().unwrap(), vec![("surface-1".to_string(), 42, 120)]);
    }

    #[tokio::test]
    async fn surface_capabilities_are_returned_from_the_control_contract() {
        let control = Arc::new(RecordingControl::default());
        let response =
            request(&control, 4, "surface.io.capabilities", json!({"surface_id": "surface-1"}))
                .await;

        assert_eq!(
            response.result,
            Some(json!({
                "read": true,
                "write": true,
                "resize": true,
                "layout": false,
                "durable_pty": true
            }))
        );
    }

    #[tokio::test]
    async fn invalid_surface_params_return_json_rpc_invalid_params() {
        let control = Arc::new(RecordingControl::default());
        let response =
            request(&control, 5, "surface.io.resize", json!({"surface_id": "surface-1"})).await;

        assert!(response.result.is_none());
        assert_eq!(response.error.unwrap().code, -32602);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn unix_server_round_trips_surface_json_rpc() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let socket =
            std::path::PathBuf::from(format!("/tmp/sharecli-sc-{}.sock", std::process::id()));
        let control = Arc::new(RecordingControl::default());
        let server_path = socket.clone();
        let server_control = control.clone();
        let server =
            tokio::spawn(async move { serve_surface_unix(&server_path, server_control).await });

        let mut stream = loop {
            match tokio::net::UnixStream::connect(&socket).await {
                Ok(stream) => break stream,
                Err(_error) if !server.is_finished() => tokio::task::yield_now().await,
                Err(error) => panic!("surface server failed before accepting connections: {error}"),
            }
        };
        stream
            .write_all(
                br#"{"jsonrpc":"2.0","id":9,"method":"surface.io.read","params":{"surface_id":"surface-1","max_bytes":8}}
"#,
            )
            .await
            .unwrap();
        let mut response = String::new();
        BufReader::new(stream).read_line(&mut response).await.unwrap();

        assert_eq!(
            serde_json::from_str::<Value>(&response).unwrap(),
            json!({"jsonrpc":"2.0","id":9,"result":{"bytes":b"terminal".to_vec()}})
        );
        server.abort();
        let _ = std::fs::remove_file(socket);
    }
}
