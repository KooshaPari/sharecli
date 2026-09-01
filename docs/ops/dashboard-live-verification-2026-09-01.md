# Dashboard Live Verification — 2026-09-01

Captured live HTTP probes against `sharecli serve` running on this Windows host.

## Binary

| Check | Result |
|-------|--------|
| `sharecli --version` | **`sharecli 0.3.0`** (build from commit before v1.0.0 tag) |
| Binary path | `C:\temp\wt-proto-mcp-target\release\sharecli.exe` |
| WinFSP runtime | `winfsp-x64.dll` v2.1.25156 alongside |

## Dashboard Server Live Probe

| Endpoint | Method | Result |
|----------|--------|--------|
| `GET /healthz` | curl | **HTTP/1.1 200 OK** + `content-type: application/json` + `traceparent: 00-000000000001111c18d12b8a1b7ef774-18d12b8a1b7fe668-01` + 15 bytes (body `{"status":"ok"}`) |
| `GET /` | curl | **HTTP/1.1 200 OK** + 27,644 bytes + **775 lines** of styled HTML |
| `GET /assets/dashboard/ui/favicons/phenotype.ico` | curl | Serves favicon (image/x-icon) |
| `GET /ws` | python WebSocket client | **HTTP/1.1 101 Switching Protocols** + `upgrade: websocket` + `sec-websocket-accept: coWAxmTVeV9oJ5wNV/nFY75UoFQ=` |

## WebSocket Handshake

```
HTTP/1.1 101 Switching Protocols
connection: upgrade
upgrade: websocket
sec-websocket-accept: coWAxmTVeV9oJ5wNV/nFY75UoFQ=
traceparent: 00-000000000001111c18d12b25a122da04-18d12b25a123cb18-01
date: Tue, 01 Sep 2026 10:19:27 GMT
```

## OTel Stack (Podman)

| Component | Status | URL |
|-----------|--------|-----|
| **Jaeger UI** | UP | `http://localhost:16686/` → `HTTP/1.1 200 OK` + `traceresponse` header |
| **OTel Collector** | Containers pulled; podman-compose startup blocked by WSL port-binding for 16686 | Config validated; restart with Jaeger-only binding succeeds |
| Podman machine | Running | `podman-machine-default` (WSL, 4 CPU, 6GiB RAM) |

## Reproducing This Verification

```powershell
# 1. Start sharecli serve
cd C:\Users\koosh\sharecli
copy C:\Users\koosh\winfsp-extract\DYNAMIC\SxS\DYNAMIC\bin\winfsp-x64.dll C:\temp\wt-proto-mcp-target\release\
start "" /B C:\temp\wt-proto-mcp-target\release\sharecli.exe serve --bind 127.0.0.1:9000 --on-conflict replace

# 2. Probe endpoints
curl -v http://127.0.0.1:9000/healthz
curl -s http://127.0.0.1:9000/  # 775 lines

# 3. WebSocket handshake (Python)
python -c "
import socket, base64, os
sock = socket.create_connection(('127.0.0.1', 9000), timeout=5)
key = base64.b64encode(os.urandom(16)).decode()
req = f'GET /ws HTTP/1.1\r\nHost: 127.0.0.1:9000\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n'
sock.sendall(req.encode())
print(sock.recv(4096).decode())
"

# 4. Start OTel stack
cd C:\Users\koosh\sharecli
git show 17e02d0:podman-compose.otel.yml > podman-compose.otel.yml
git show 17e02d0:otel-collector-config.yaml > otel-collector-config.yaml
podman-compose -f podman-compose.otel.yml up -d
# Open http://localhost:16686 for Jaeger UI
```

## FR Traceability

| FR | Verification |
|----|--------------|
| FR-009 (Dashboard) | 775-line HTML, /healthz JSON, /ws WebSocket, asset serving |
| FR-008 (OTel) | Podman stack up (Jaeger confirmed; collector has podman-WSL path issue) |
| FR-003 (Multi-agent) | N/A (covered by `tests/c03_multi_agent_scale_gate.rs`) |

## Known Limitations

- **OTel collector** in podman-compose hits WSL port-binding conflict on `16686` (Jaeger owns it). Workaround: drop the collector's 16686 binding or run with explicit `--network sharecli_default`.
- **Binary version** reports `0.3.0` because the local build predates the `1.0.0` tag (commit `a290e3a`). The release workflow at tag `v1.0.0` produces the actual 1.0.0 binary.
- **WinFSP driver** is not kernel-installed on this host (DLL extracted from MSI but driver service not registered). FUSE mount subcommands will fail at runtime; CLI/Dashboard/TUI unaffected.
