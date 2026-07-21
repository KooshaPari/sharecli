using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace ShareCLITray;

/// IPC `pool.status` + `status.snapshot` wire envelopes (FR-007 / AC-007.68).
/// Parity with Swift `PoolSnapshot` / `StatusSnapshot` and Linux tray `ipc.rs`.
public sealed class PoolSnapshot
{
    [JsonPropertyName("node_total")]
    public int NodeTotal { get; set; }

    [JsonPropertyName("node_idle")]
    public int NodeIdle { get; set; }

    [JsonPropertyName("bun_total")]
    public int BunTotal { get; set; }

    [JsonPropertyName("bun_idle")]
    public int BunIdle { get; set; }

    [JsonPropertyName("max_per_type")]
    public int MaxPerType { get; set; }

    [JsonPropertyName("healthy")]
    public bool Healthy { get; set; }

    [JsonPropertyName("issues")]
    public List<string> Issues { get; set; } = [];

    [JsonPropertyName("gate")]
    public GateStatusSnapshot Gate { get; set; } = new();

    [JsonPropertyName("host_watch")]
    public HostResourceWatchJson HostWatch { get; set; } = new();

    /// Decode `pool.status` IPC response envelope `{ id, result, error }`.
    public static PoolSnapshot? TryParseIpcResponse(string? responseJson)
    {
        if (string.IsNullOrWhiteSpace(responseJson))
        {
            return null;
        }

        try
        {
            using var doc = JsonDocument.Parse(responseJson);
            var root = doc.RootElement;
            if (root.TryGetProperty("error", out var err) && err.ValueKind == JsonValueKind.String)
            {
                return null;
            }
            if (!root.TryGetProperty("result", out var result) || result.ValueKind == JsonValueKind.Null)
            {
                return null;
            }
            return JsonSerializer.Deserialize<PoolSnapshot>(result.GetRawText());
        }
        catch (JsonException)
        {
            return null;
        }
    }
}

public sealed class StatusSnapshot
{
    [JsonPropertyName("total_processes")]
    public int TotalProcesses { get; set; }

    [JsonPropertyName("agents")]
    public List<AgentProcRow> Agents { get; set; } = [];

    [JsonPropertyName("scanned")]
    public int Scanned { get; set; }

    [JsonPropertyName("watched")]
    public int Watched { get; set; }

    [JsonPropertyName("gate")]
    public GateStatusSnapshot Gate { get; set; } = new();

    [JsonPropertyName("host_watch")]
    public HostResourceWatchJson HostWatch { get; set; } = new();

    /// Decode `status.snapshot` IPC response envelope `{ id, result, error }`.
    public static StatusSnapshot? TryParseIpcResponse(string? responseJson)
    {
        if (string.IsNullOrWhiteSpace(responseJson))
        {
            return null;
        }

        try
        {
            using var doc = JsonDocument.Parse(responseJson);
            var root = doc.RootElement;
            if (root.TryGetProperty("error", out var err) && err.ValueKind == JsonValueKind.String)
            {
                return null;
            }
            if (!root.TryGetProperty("result", out var result) || result.ValueKind == JsonValueKind.Null)
            {
                return null;
            }
            return JsonSerializer.Deserialize<StatusSnapshot>(result.GetRawText());
        }
        catch (JsonException)
        {
            return null;
        }
    }
}

public sealed class AgentProcRow
{
    [JsonPropertyName("pid")]
    public uint Pid { get; set; }

    [JsonPropertyName("family")]
    public string Family { get; set; } = "";

    [JsonPropertyName("comm")]
    public string Comm { get; set; } = "";

    [JsonPropertyName("state")]
    public string State { get; set; } = "";

    [JsonPropertyName("mem_rss_bytes")]
    public ulong MemRssBytes { get; set; }

    [JsonPropertyName("mem_rss")]
    public string MemRss { get; set; } = "";

    [JsonPropertyName("fd_count")]
    public ulong? FdCount { get; set; }
}
