// MARK: - Right-click / Popover Coordination
extension TrayMenuController {

    /// One-call setup: builds the menu, installs the event monitor, and
    /// attaches the menu to the status item.
    @MainActor static func installContextMenu(for statusItem: NSStatusItem) {
        let menu = MenuAction.shared.buildMenu(
            openDashboard: { NotificationCenter.default.post(name: .sharecliOpenDashboardRequested, object: nil) }
        )
        install(statusItem: statusItem, menu: menu)
        installRightClickMonitor()
    }.swift
//  ShareCLITray
//
//  Owns the right-click NSMenu (Open Dashboard / Refresh / Pause-Resume /
//  Kill All / Quit) and the dashboard-window positioning helper that
//  anchors the dashboard window just below the menubar icon (instead of
//  at screen center).
//
//  Wired in via 2 single-line calls from AppEntry.swift:
//    - TrayMenuController.installContextMenu(for: statusItem)
//    - TrayMenuController.positionDashboardWindow(_:below:)
//  Existing popover behavior is untouched.
//

import AppKit
import ShareCLICore

@MainActor
enum TrayMenuController {

    /// Install a right-click NSMenu on the given status item. Left-click
    /// still drives the existing popover via the button's action target.
    /// Right-click (anywhere on the status item's frame) opens the menu.
    static func installContextMenu(for statusItem: NSStatusItem) {
        let menu = NSMenu(title: "ShareCLI")
        menu.autoenablesItems = false

        let state = AppState.shared

        let openDashboard = NSMenuItem(
            title: "Open Dashboard",
            action: #selector(MenuAction.openDashboard(_:)),
            keyEquivalent: "d"
        )
        openDashboard.target = MenuAction.shared
        openDashboard.image = NSImage(systemSymbolName: "rectangle.stack.fill")
        menu.addItem(openDashboard)

        let refresh = NSMenuItem(
            title: "Refresh Now",
            action: #selector(MenuAction.refresh(_:)),
            keyEquivalent: "r"
        )
        refresh.target = MenuAction.shared
        refresh.image = NSImage(systemSymbolName: "arrow.clockwise")
        menu.addItem(refresh)

        menu.addItem(.separator())

        let pause = NSMenuItem(
            title: state.isPaused ? "Resume Updates" : "Pause Updates",
            action: #selector(MenuAction.togglePause(_:)),
            keyEquivalent: ""
        )
        pause.target = MenuAction.shared
        pause.image = NSImage(systemSymbolName: state.isPaused ? "play.fill" : "pause.fill")
        menu.addItem(pause)

        let killAll = NSMenuItem(
            title: "Kill All Processes",
            action: #selector(MenuAction.killAll(_:)),
            keyEquivalent: ""
        )
        killAll.target = MenuAction.shared
        killAll.image = NSImage(systemSymbolName: "xmark.bin")
        menu.addItem(killAll)

        menu.addItem(.separator())

        let quit = NSMenuItem(
            title: "Quit ShareCLI",
            action: #selector(MenuAction.quit(_:)),
            keyEquivalent: "q"
        )
        quit.target = MenuAction.shared
        quit.image = NSImage(systemSymbolName: "power")
        menu.addItem(quit)

        // macOS quirk: when `statusItem.menu` is set, AppKit opens the menu
        // on *every* click including left. To preserve left-click →
        // popover, we install our own button target that pops the
        // existing popover, and attach the menu dynamically only on
        // right-MouseDown via a local event monitor.
        if let button = statusItem.button {
            button.target = MenuAction.shared
            button.action = #selector(MenuAction.statusItemClicked(_:))
        }
        statusItem.menu = nil

        MenuAction.shared.install(statusItem: statusItem, menu: menu)
        MenuAction.shared.installRightClickMonitor()
    }

    /// Anchor the dashboard window so its top edge sits just below the
    /// menubar icon. Replaces the prior `win.center()` so the dashboard
    /// appears as a natural "extend" from the menubar (the way a
    /// well-behaved tray utility should).
    static func positionDashboardWindow(_ window: NSWindow, below statusItem: NSStatusItem) {
        guard let button = statusItem.button,
              let buttonWindow = button.window else {
            window.center()
            return
        }
        let buttonFrame = button.convert(button.bounds, to: nil)
        let buttonOnScreen = buttonWindow.convertToScreen(buttonFrame)
        let winSize = window.frame.size
        let margin: CGFloat = 6  // gap between icon and dashboard top

        var origin = NSPoint(
            x: buttonOnScreen.origin.x,
            y: buttonOnScreen.origin.y - winSize.height - margin
        )
        // Clamp horizontally so the window stays on-screen even if the
        // icon is near the right edge.
        if let screen = buttonWindow.screen {
            let minX = screen.visibleFrame.origin.x
            let maxX = screen.visibleFrame.origin.x + screen.visibleFrame.width - winSize.width
            origin.x = min(max(origin.x, minX), max(maxX, minX))
        }
        window.setFrameOrigin(origin)
    }
}

// MARK: - MenuAction (selector target)

/// Singleton object that receives the menu selectors and bridges to
/// `AppState`. NSMenu selectors must target a real object, so we keep
/// our own singleton.
@MainActor
final class MenuAction: NSObject {
    static let shared = MenuAction()
    private var statusItem: NSStatusItem?
    private var menu: NSMenu?
    private var rightClickMonitor: Any?

    func install(statusItem: NSStatusItem, menu: NSMenu) {
        self.statusItem = statusItem
        self.menu = menu
    }

    /// Install a local NSEvent monitor that intercepts right-mouseDown
    /// events over the status item button and pops the context menu.
    /// Left-mouseDown events fall through to the existing button.action
    /// (popover).
    func installRightClickMonitor() {
        guard rightClickMonitor == nil, let statusItem, let menu else { return }
        let button = statusItem.button
        rightClickMonitor = NSEvent.addLocalMonitorForEvents(
            matching: [.rightMouseDown, .rightMouseUp]
        ) { [weak statusItem, weak menu] event in
            guard let statusItem, let menu, let button = statusItem.button else {
                return event
            }
            let buttonWindow = button.window
            let buttonScreenFrame = buttonWindow?
                .convertToScreen(button.convert(button.bounds, to: nil))
            let mouseInScreen = NSEvent.mouseLocation
            if let buttonScreenFrame, buttonScreenFrame.contains(mouseInScreen) {
                if event.type == .rightMouseDown {
                    menu.popUp(positioning: nil, at: NSPoint(x: 0, y: button.bounds.height + 2), in: button)
                }
                return nil  // Swallow the right-click event
            }
            return event
        }
    }

    /// Left-click on the status item button. Pops the existing
    /// popover (preserves the prior UX).
    @objc func statusItemClicked(_ sender: NSStatusBarButton) {
        guard let popover = popover, let btn = statusItem?.button else { return }
        if popover.isShown {
            popover.performClose(nil)
        } else {
            popover.show(relativeTo: btn.bounds, of: btn, preferredEdge: .minY)
            popover.contentViewController?.view.window?.makeKey()
        }
    }

    @objc func openDashboard(_ sender: Any?) {
        NotificationCenter.default.post(
            name: .sharecliOpenDashboardRequested, object: nil
        )
    }

    @objc func refresh(_ sender: Any?) {
        Task { @MainActor in
            await AppState.shared.refresh()
        }
    }

    @objc func togglePause(_ sender: Any?) {
        let state = AppState.shared
        if state.isPaused {
            state.resumePolling()
        } else {
            state.pausePolling()
        }
        // Rebuild menu so the title/icon reflects new state.
        if let statusItem, let menu {
            // Rebuild by calling install with new state.
            TrayMenuController.installContextMenu(for: statusItem)
            _ = menu  // keep ref alive
        }
    }

    @objc func killAll(_ sender: Any?) {
        Task { @MainActor in
            await AppState.shared.killAll()
        }
    }

    @objc func quit(_ sender: Any?) {
        NSApp.terminate(nil)
    }

    private var popover: NSPopover?
    func attachPopover(_ popover: NSPopover) {
        self.popover = popover
    }
}

extension Notification.Name {
    /// Posted when the user picks "Open Dashboard" from the tray
    /// context menu. AppDelegate observes and brings the dashboard
    /// window forward.
    static let sharecliOpenDashboardRequested =
        Notification.Name("sharecliOpenDashboardRequested")
}
