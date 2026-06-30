#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

/// Start the IPC daemon sidecar in the background (idempotent).
/// Returns 0 on success.
int sharecli_ipc_start(void);

/// Returns the IPC Unix socket path (heap-allocated, free with sharecli_free_string).
char *sharecli_ipc_socket_path(void);

/// Free a string returned by this library.
void sharecli_free_string(char *ptr);

/// Synchronous health snapshot; returns JSON string or NULL on error.
/// Free with sharecli_free_string.
char *sharecli_health_json(void);

/// Send a raw JSON-RPC request, return the response JSON string.
/// Free result with sharecli_free_string.
char *sharecli_request(const char *request_json);

#ifdef __cplusplus
}
#endif
