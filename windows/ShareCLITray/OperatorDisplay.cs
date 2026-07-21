namespace ShareCLITray;

/// Compact gate / host_watch tray strings + thermal visuals (FR-007 / AC-007.56 text, AC-007.57).
/// Parity with `sharecli-tray-linux/src/operator_display.rs` golden strings.
public static class OperatorDisplay
{
    public enum TrayGateSeverity
    {
        Normal,
        Warning,
        Critical,
        Offline,
    }

    public sealed class TrayGateVisual
    {
        public TrayGateSeverity Severity { get; init; }
        public string DecisionClass { get; init; } = "";
        public string ThermalClass { get; init; } = "";
        public string ColorHex { get; init; } = "";
        public string BadgeLabel { get; init; } = "";
        public string SwiftSymbolName { get; init; } = "";
    }

    public static string ResolveGateDecisionClass(string gateDecision) =>
        gateDecision switch
        {
            "ADMIT" => "gate-admit",
            "DENY" => "gate-deny",
            _ => "gate-unavailable",
        };

    public static string ResolveThermalClass(string thermalPressure) =>
        thermalPressure switch
        {
            "GREEN" => "",
            "YELLOW" => "warning",
            "RED" => "critical",
            _ => "warning",
        };

    public static TrayGateVisual ResolveTrayGateVisual(
        string thermalPressure,
        string gateDecision,
        bool connected)
    {
        if (!connected)
        {
            return new TrayGateVisual
            {
                Severity = TrayGateSeverity.Offline,
                DecisionClass = "gate-unavailable",
                ThermalClass = "warning",
                ColorHex = "#d29922",
                BadgeLabel = "Offline",
                SwiftSymbolName = "wifi.slash",
            };
        }

        var decisionClass = ResolveGateDecisionClass(gateDecision);
        var thermalClass = ResolveThermalClass(thermalPressure);

        TrayGateSeverity severity;
        if (gateDecision == "DENY" || thermalPressure == "RED")
        {
            severity = TrayGateSeverity.Critical;
        }
        else if (gateDecision == "THROTTLE" || thermalPressure == "YELLOW"
            || thermalPressure == "UNAVAILABLE" || decisionClass == "gate-unavailable")
        {
            severity = TrayGateSeverity.Warning;
        }
        else
        {
            severity = TrayGateSeverity.Normal;
        }

        return severity switch
        {
            TrayGateSeverity.Critical => new TrayGateVisual
            {
                Severity = TrayGateSeverity.Critical,
                DecisionClass = decisionClass,
                ThermalClass = thermalClass,
                ColorHex = "#f85149",
                BadgeLabel = "Critical",
                SwiftSymbolName = "flame.fill",
            },
            TrayGateSeverity.Warning => new TrayGateVisual
            {
                Severity = TrayGateSeverity.Warning,
                DecisionClass = decisionClass,
                ThermalClass = thermalClass,
                ColorHex = "#d29922",
                BadgeLabel = thermalPressure == "UNAVAILABLE" ? "Unavailable" : "Warning",
                SwiftSymbolName = "exclamationmark.triangle.fill",
            },
            TrayGateSeverity.Normal => new TrayGateVisual
            {
                Severity = TrayGateSeverity.Normal,
                DecisionClass = decisionClass,
                ThermalClass = thermalClass,
                ColorHex = "#3fb950",
                BadgeLabel = "Normal",
                SwiftSymbolName = "cpu",
            },
            _ => new TrayGateVisual
            {
                Severity = TrayGateSeverity.Offline,
                DecisionClass = "gate-unavailable",
                ThermalClass = "warning",
                ColorHex = "#d29922",
                BadgeLabel = "Offline",
                SwiftSymbolName = "wifi.slash",
            },
        };
    }

    public static TrayGateVisual ResolveTrayGateVisual(GateStatusSnapshot gate, bool connected) =>
        ResolveTrayGateVisual(gate.ThermalPressure, gate.GateDecision, connected);

    public static Microsoft.UI.Xaml.Media.Brush SeverityBrush(TrayGateSeverity severity) =>
        severity switch
        {
            TrayGateSeverity.Normal => new Microsoft.UI.Xaml.Media.SolidColorBrush(
                Microsoft.UI.Colors.LimeGreen),
            TrayGateSeverity.Critical => new Microsoft.UI.Xaml.Media.SolidColorBrush(
                Microsoft.UI.Colors.IndianRed),
            _ => new Microsoft.UI.Xaml.Media.SolidColorBrush(
                Microsoft.UI.Colors.Goldenrod),
        };

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
