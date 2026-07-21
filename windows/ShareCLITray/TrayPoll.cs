namespace ShareCLITray;

/// Periodic tray poll cadence (FR-007 / AC-007.52).
/// MUST match `sharecli_tray_windows::poll::TRAY_POLL_INTERVAL_SECS` (Linux/Swift parity).
public static class TrayPoll
{
    public const int IntervalSeconds = 3;
}
