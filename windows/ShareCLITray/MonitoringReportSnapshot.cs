using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace ShareCLITray;

/// IPC `monitoring.report` envelope + mapping helpers (FR-007 / AC-007.51).
/// Parity with Swift `MonitoringReportSnapshot` and Linux tray `ipc.rs`.
public sealed class MonitoringReportSnapshot
{
    [JsonPropertyName("timestamp")]
    public ulong Timestamp { get; set; }

    [JsonPropertyName("total_processes")]
    public int TotalProcesses { get; set; }

    [JsonPropertyName("used_memory_mb")]
    public ulong UsedMemoryMb { get; set; }

    [JsonPropertyName("total_memory_mb")]
    public ulong TotalMemoryMb { get; set; }

    [JsonPropertyName("processes")]
    public List<MonitoringProcessEntry> Processes { get; set; } = [];

    [JsonPropertyName("gate")]
    public GateStatusSnapshot Gate { get; set; } = new();

    [JsonPropertyName("host_watch")]
    public HostResourceWatchJson HostWatch { get; set; } = new();

    public TrayHealthSnapshot AsHealthSnapshot()
    {
        return new TrayHealthSnapshot
        {
            ManagedProcesses = TotalProcesses,
            UsedMemoryMb = UsedMemoryMb,
            TotalMemoryMb = TotalMemoryMb,
            Healthy = UsedMemoryMb < TotalMemoryMb / 2,
            Gate = Gate,
            HostWatch = HostWatch,
        };
    }

    public List<ProcessInfo> AsProcessSummaries()
    {
        var rows = new List<ProcessInfo>(Processes.Count);
        foreach (var entry in Processes)
        {
            rows.Add(new ProcessInfo
            {
                pid = entry.Pid,
                name = entry.Name,
                memory_mb = entry.MemoryMb,
                project = entry.Project,
            });
        }
        return rows;
    }

    /// Decode `monitoring.report` IPC response envelope `{ id, result, error }`.
    public static MonitoringReportSnapshot? TryParseIpcResponse(string? responseJson)
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
            return JsonSerializer.Deserialize<MonitoringReportSnapshot>(result.GetRawText());
        }
        catch (JsonException)
        {
            return null;
        }
    }
}

public sealed class MonitoringProcessEntry
{
    [JsonPropertyName("pid")]
    public uint Pid { get; set; }

    [JsonPropertyName("name")]
    public string Name { get; set; } = "";

    [JsonPropertyName("memory_mb")]
    public ulong MemoryMb { get; set; }

    [JsonPropertyName("project")]
    public string? Project { get; set; }

    [JsonPropertyName("harness")]
    public string? Harness { get; set; }
}

public sealed class GateStatusSnapshot
{
    [JsonPropertyName("thermal_pressure")]
    public string ThermalPressure { get; set; } = "";

    [JsonPropertyName("detected_agents")]
    public int DetectedAgents { get; set; }

    [JsonPropertyName("agent_total_rss_bytes")]
    public ulong AgentTotalRssBytes { get; set; }

    [JsonPropertyName("agent_contention")]
    public string AgentContention { get; set; } = "";

    [JsonPropertyName("gate_decision")]
    public string GateDecision { get; set; } = "";
}

public sealed class HostResourceWatchJson
{
    [JsonPropertyName("fd_count")]
    public ulong FdCount { get; set; }

    [JsonPropertyName("net_rx_bytes")]
    public ulong NetRxBytes { get; set; }

    [JsonPropertyName("net_tx_bytes")]
    public ulong NetTxBytes { get; set; }

    [JsonPropertyName("mem_rss_bytes")]
    public ulong MemRssBytes { get; set; }

    [JsonPropertyName("load_1m")]
    public double Load1m { get; set; }
}

public sealed class TrayHealthSnapshot
{
    public int ManagedProcesses { get; set; }
    public ulong UsedMemoryMb { get; set; }
    public ulong TotalMemoryMb { get; set; }
    public bool Healthy { get; set; }
    public GateStatusSnapshot Gate { get; set; } = new();
    public HostResourceWatchJson HostWatch { get; set; } = new();
}
