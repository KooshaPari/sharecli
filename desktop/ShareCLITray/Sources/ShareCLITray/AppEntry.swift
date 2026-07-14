/// main.swift — NSStatusItem tray entry point for ShareCLI Desktop.
///
/// Architecture:
///   NSStatusItem (menu bar icon)
///     └─ NSPopover  (click → popover with summary + quick actions)
///          └─ "Open Dashboard" button → NSWindow (full dashboard NSHostingView)

import AppKit
import SwiftUI
import ShareCLICore

@main
struct ShareCLITrayApp {
    static func main() {
        let app = NSApplication.shared
        let delegate = AppDelegate()
        app.delegate = delegate
        app.run()
    }
}

final class AppDelegate: NSObject, NSApplicationDelegate {
    private var statusItem: NSStatusItem!
    private var popover: NSPopover!
    private var dashboardWindow: NSWindow?

    @MainActor private let state = AppState()

    func applicationDidFinishLaunching(_ notification: Notification) {
        // Hide from Dock — pure tray app
        NSApp.setActivationPolicy(.accessory)

        // Ensure IPC sidecar is running
        Task { @MainActor in
            await ensureIPC()
            state.startPolling()
        }

        setupStatusItem()
        setupPopover()
    }

    // MARK: - Status item

    private func setupStatusItem() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)

        let btn = statusItem.button!
        btn.image = NSImage(systemSymbolName: "cpu", accessibilityDescription: "ShareCLI")
        btn.imagePosition = .imageLeading
        btn.action = #selector(togglePopover)
        btn.target = self

        // Keep title updated from live health
        NotificationCenter.default.addObserver(
            forName: .sharecliHealthChanged,
            object: nil,
            queue: .main
        ) { [weak self] note in
            guard let snap = note.object as? HealthSnapshot else { return }
            self?.statusItem.button?.title =
                " \(snap.managed_processes) | \(snap.used_memory_mb)M"
        }
    }

    // MARK: - Popover

    private func setupPopover() {
        popover = NSPopover()
        popover.contentSize = NSSize(width: 360, height: 480)
        popover.behavior = .transient
        popover.contentViewController = NSHostingController(
            rootView: TrayPopoverView(state: state, onOpenDashboard: { [weak self] in
                self?.openDashboard()
            })
        )
    }

    @objc private func togglePopover() {
        guard let btn = statusItem.button else { return }
        if popover.isShown {
            popover.performClose(nil)
        } else {
            popover.show(relativeTo: btn.bounds, of: btn, preferredEdge: .minY)
            popover.contentViewController?.view.window?.makeKey()
        }
    }

    // MARK: - Dashboard window

    private func openDashboard() {
        if let existing = dashboardWindow {
            existing.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            return
        }

        let win = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 900, height: 620),
            styleMask: [.titled, .closable, .resizable, .miniaturizable],
            backing: .buffered,
            defer: false
        )
        win.title = "ShareCLI Dashboard"
        win.center()
        win.contentView = NSHostingView(rootView: DashboardView(state: state))
        win.isReleasedWhenClosed = false
        win.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        dashboardWindow = win
    }

    // MARK: - IPC lifecycle

    private func ensureIPC() async {
        // Try connecting; if that fails, attempt to start the sidecar.
        do {
            _ = try await IPCClient().health()
        } catch {
            // Sidecar not running — launch it.
            let exe = sidecarPath("sharecli-ipc")
            if let exe {
                let proc = Process()
                proc.executableURL = URL(fileURLWithPath: exe)
                try? proc.run()
                try? await Task.sleep(nanoseconds: 300_000_000)
            }
        }
    }

    private func sidecarPath(_ name: String) -> String? {
        let bundle = Bundle.main.bundlePath
        let bundleExe = "\(bundle)/Contents/Resources/bin/\(name)"
        if FileManager.default.fileExists(atPath: bundleExe) { return bundleExe }

        let output = Process()
        output.executableURL = URL(fileURLWithPath: "/usr/bin/which")
        output.arguments = [name]
        let pipe = Pipe()
        output.standardOutput = pipe
        try? output.run()
        output.waitUntilExit()
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        let path = String(data: data, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines)
        return path?.isEmpty == false ? path : nil
    }
}

extension Notification.Name {
    static let sharecliHealthChanged = Notification.Name("sharecliHealthChanged")
}
