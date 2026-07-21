using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using System;
using System.Collections.Generic;
using System.Threading.Tasks;

namespace ShareCLITray;

public sealed partial class TrayWindow : Window
{
    private List<ProcessInfo> m_processes = [];
    private DispatcherQueueTimer? _pollTimer;

    public TrayWindow()
    {
        InitializeComponent();

        StartPeriodicPolling();
        _ = RefreshDataAsync();
    }

    private void StartPeriodicPolling()
    {
        var queue = DispatcherQueue.GetForCurrentThread();
        _pollTimer = queue.CreateTimer();
        _pollTimer.Interval = TimeSpan.FromSeconds(TrayPoll.IntervalSeconds);
        _pollTimer.Tick += async (_, _) => await RefreshDataAsync();
        _pollTimer.Start();
    }

    private async Task RefreshDataAsync()
    {
        // Single `monitoring.report` round-trip drives operator gate/host_watch + process
        // inventory (AC-007.51); avoids split `health.status` + `process.list` polls.
        var reportJson = ShareCLIInterop.SendRequest(
            "{\"id\": 1, \"method\": \"monitoring.report\", \"params\": {}}");
        var report = MonitoringReportSnapshot.TryParseIpcResponse(reportJson);
        if (report == null)
        {
            var offlineVisual = OperatorDisplay.ResolveTrayGateVisual("", "", connected: false);
            DispatcherQueue?.TryEnqueue(() =>
            {
                HealthStatusText.Text = OperatorDisplay.FormatHealthStatusOfflineLine(
                    offlineVisual,
                    "Daemon offline or monitoring.report failed");
                HealthStatusText.Foreground = OperatorDisplay.SeverityBrush(offlineVisual.Severity);
                ThermalBadgeText.Text = "Offline";
                ThermalBadgeText.Foreground = OperatorDisplay.SeverityBrush(
                    OperatorDisplay.TrayGateSeverity.Offline);
                GateStatusText.Text = "";
                HostWatchStatusText.Text = "";
                PoolStatusText.Text = "";
                StatusSnapshotText.Text = "";
                ProcessGrid.ItemsSource = null;
            });
            return;
        }

        var health = report.AsHealthSnapshot();
        m_processes = report.AsProcessSummaries();
        var visual = OperatorDisplay.ResolveTrayGateVisual(health.Gate, connected: true);

        // Supplementary pool/status IPC enriches operator panels (AC-007.69); primary refresh
        // stays monitoring.report (AC-007.51).
        var poolJson = ShareCLIInterop.SendRequest(
            "{\"id\": 2, \"method\": \"pool.status\", \"params\": {}}");
        var statusJson = ShareCLIInterop.SendRequest(
            "{\"id\": 3, \"method\": \"status.snapshot\", \"params\": {}}");
        var pool = PoolSnapshot.TryParseIpcResponse(poolJson);
        var status = StatusSnapshot.TryParseIpcResponse(statusJson);

        DispatcherQueue?.TryEnqueue(() =>
        {
            HealthStatusText.Text = OperatorDisplay.FormatHealthStatusLine(
                visual, health.Gate, health);
            HealthStatusText.Foreground = OperatorDisplay.SeverityBrush(visual.Severity);
            ThermalBadgeText.Text =
                $"{visual.BadgeLabel} · {health.Gate.ThermalPressure} · {health.Gate.GateDecision}";
            ThermalBadgeText.Foreground = OperatorDisplay.SeverityBrush(visual.Severity);
            GateStatusText.Text =
                $"{OperatorDisplay.FormatGateTrayLine(health.Gate)} | " +
                OperatorDisplay.FormatGateRssTrayLine(health.Gate);
            GateStatusText.Foreground = OperatorDisplay.SeverityBrush(visual.Severity);
            HostWatchStatusText.Text =
                $"{OperatorDisplay.FormatHostWatchTrayLine(health.HostWatch)} | " +
                OperatorDisplay.FormatHostNetTrayLine(health.HostWatch);
            if (pool != null && status != null)
            {
                PoolStatusText.Text = OperatorDisplay.FormatPoolTrayLine(pool);
                StatusSnapshotText.Text = OperatorDisplay.FormatStatusSnapshotTrayLine(status);
            }
            else
            {
                PoolStatusText.Text = "";
                StatusSnapshotText.Text = "";
            }
            ProcessGrid.ItemsSource = m_processes;
        });
    }

    private async void OnRefreshClick(object sender, RoutedEventArgs e)
    {
        await RefreshDataAsync();
    }

    private async void OnKillProcessClick(object sender, RoutedEventArgs e)
    {
        if (sender is Button { Tag: uint pid })
        {
            IpcKill.TryKill(pid);
            await RefreshDataAsync();
        }
    }

    private async void OnKillAllClick(object sender, RoutedEventArgs e)
    {
        IpcKill.TryKillAll();
        await RefreshDataAsync();
    }

    private void OnCloseClick(object sender, RoutedEventArgs e)
    {
        _pollTimer?.Stop();
        Close();
    }
}

public record ProcessInfo
{
    public uint pid { get; set; }
    public string name { get; set; } = "";
    public ulong memory_mb { get; set; }
    public string? project { get; set; }
    public string? harness { get; set; }
}
