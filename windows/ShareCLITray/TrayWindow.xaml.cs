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
            DispatcherQueue?.TryEnqueue(() =>
            {
                HealthStatusText.Text = "Daemon offline or monitoring.report failed";
                ProcessGrid.ItemsSource = null;
            });
            return;
        }

        var health = report.AsHealthSnapshot();
        m_processes = report.AsProcessSummaries();

        DispatcherQueue?.TryEnqueue(() =>
        {
            HealthStatusText.Text =
                $"Health: {(health.Healthy ? "✓ OK" : "✗ Unhealthy")} | " +
                $"Managed: {health.ManagedProcesses} | " +
                $"Memory: {health.UsedMemoryMb} / {health.TotalMemoryMb} MB | " +
                $"Gate: {health.Gate.GateDecision} | " +
                $"Load: {health.HostWatch.Load1m:F2}";
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
}
