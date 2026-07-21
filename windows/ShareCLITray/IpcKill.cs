using System.Text.Json;

namespace ShareCLITray;

/// IPC kill operator actions (FR-007 / AC-007.54).
/// Parity with Linux tray `ipc::kill` / `ipc::kill_all` and Swift `AppState.kill` / `killAll`.
public static class IpcKill
{
    private static int _nextId = 2;

    /// Send `process.kill` for one managed PID; returns true when IPC result is `true`.
    public static bool TryKill(uint pid)
    {
        var id = _nextId++;
        var request =
            $"{{\"id\": {id}, \"method\": \"process.kill\", \"params\": {{\"pid\": {pid}}}}}";
        return TryParseBoolResult(ShareCLIInterop.SendRequest(request));
    }

    /// Send `process.kill_all`; returns true when IPC result is `true`.
    public static bool TryKillAll()
    {
        var id = _nextId++;
        var request =
            $"{{\"id\": {id}, \"method\": \"process.kill_all\", \"params\": {{}}}}";
        return TryParseBoolResult(ShareCLIInterop.SendRequest(request));
    }

    internal static bool TryParseBoolResult(string? responseJson)
    {
        if (string.IsNullOrWhiteSpace(responseJson))
        {
            return false;
        }

        try
        {
            using var doc = JsonDocument.Parse(responseJson);
            var root = doc.RootElement;
            if (root.TryGetProperty("error", out var err) && err.ValueKind == JsonValueKind.String)
            {
                return false;
            }
            if (!root.TryGetProperty("result", out var result))
            {
                return false;
            }
            return result.ValueKind == JsonValueKind.True;
        }
        catch (JsonException)
        {
            return false;
        }
    }
}
