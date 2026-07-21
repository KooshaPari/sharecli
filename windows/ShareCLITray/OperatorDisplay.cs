namespace ShareCLITray;

/// Compact gate / host_watch tray strings (FR-007 / AC-007.56).
/// Parity with `sharecli-tray-linux/src/operator_display.rs` golden strings.
public static class OperatorDisplay
{
    public static string FormatBytesCompact(ulong n)
    {
        if (n >= 1_048_576)
        {
            return $"{n / 1_048_576.0:F1} MB";
        }
        if (n >= 1024)
        {
            return $"{n / 1024.0:F1} KB";
        }
        return $"{n} B";
    }

    public static string FormatGateTrayLine(GateStatusSnapshot gate) =>
        $"Gate [{gate.GateDecision}] · {gate.ThermalPressure} · agents {gate.DetectedAgents} · {gate.AgentContention}";

    public static string FormatGateRssTrayLine(GateStatusSnapshot gate) =>
        $"Agent RSS: {FormatBytesCompact(gate.AgentTotalRssBytes)}";

    public static string FormatHostWatchTrayLine(HostResourceWatchJson host) =>
        $"Host load {host.Load1m:F2} · FDs {host.FdCount} · RSS {FormatBytesCompact(host.MemRssBytes)}";

    public static string FormatHostNetTrayLine(HostResourceWatchJson host) =>
        $"Net RX {FormatBytesCompact(host.NetRxBytes)} · TX {FormatBytesCompact(host.NetTxBytes)}";

    public static string[] FormatOperatorTrayLines(
        GateStatusSnapshot gate,
        HostResourceWatchJson host) =>
    [
        FormatGateTrayLine(gate),
        FormatGateRssTrayLine(gate),
        FormatHostWatchTrayLine(host),
        FormatHostNetTrayLine(host),
    ];

    public static string FormatOperatorStatusSummary(
        GateStatusSnapshot gate,
        HostResourceWatchJson host) =>
        $"{FormatGateTrayLine(gate)} | {FormatGateRssTrayLine(gate)} | {FormatHostWatchTrayLine(host)}";
}
