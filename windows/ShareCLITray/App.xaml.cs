using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using System;
using System.Threading;
using Windows.UI.Popups;

namespace ShareCLITray;

public partial class App : Application
{
    private Window? m_window;
    private TrayWindow? m_trayWindow;
    private static Mutex? _singleInstanceMutex;

    public App()
    {
        const string mutexName = @"Local\ShareCLITray.SingleInstance";
        _singleInstanceMutex = new Mutex(true, mutexName, out bool createdNew);
        if (!createdNew)
        {
            Environment.Exit(0);
        }
        InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        // Start the sharecli-ipc daemon.
        int startResult = ShareCLIInterop.sharecli_ipc_start();
        if (startResult != 0)
        {
            _ = new ContentDialog
            {
                Title = "sharecli-ipc startup failed",
                Content = "Failed to start the sharecli IPC server. Check that sharecli-ipc binary is available.",
                PrimaryButtonText = "OK",
                XamlRoot = null,
            }.ShowAsync();
        }

        // Create and show the tray window.
        m_trayWindow = new TrayWindow();
        m_trayWindow.Activate();
    }
}
