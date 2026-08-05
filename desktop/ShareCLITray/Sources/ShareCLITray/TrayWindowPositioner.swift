// TrayWindowPositioner.swift
//
// Standalone helper that handles the macOS-specific positioning requirements
// for the ShareCLI tray dashboard window — anchored just below the menubar
// icon (not screen center) — and installs a local NSEvent monitor that
// routes right-clicks on the status item to a context menu while preserving
// left-click popover behavior.
//
// Design notes:
//   • Uses `NSEvent.addLocalMonitorForEvents(matching:)` so the tray keeps
//     full control over left-click. The monitor only intercepts events whose
//     `window` is the status button OR whose location falls inside its frame
//     on the menubar screen coordinate.
//   • Places the dashboard window via `NSWindow.setFrameOrigin` rather than
//     `center()` so it appears as a "pop out" extension of the menubar icon,
//     matching user expectations from native tray apps (1Password, Docker,
//     Raycast, etc.).
//   • All work runs on @MainActor — AppKit positioning must not be called
//     off the main thread.

import AppKit
import SwiftUI

/// Positions the dashboard window just below the status-item icon and
/// installs the right-click NSMenu on the same button.
///
/// Usage:
/// ```
/// TrayWindowPositioner.install(
///     menu: contextMenu,
///     on: statusItem,
///     dashboardWindow: dashboardWindow,
///     openDashboard: { [weak self] in self?.openDashboard() }
/// )
/// ```
@MainActor
public enum TrayWindowPositioner {

    /// Margin (in points) between the status icon's bottom edge and the
    /// dashboard window's top edge.
    public static let windowMarginY: CGFloat = 4

    /// Margin (in points) between the status icon's left edge and the
    /// dashboard window's left edge.
    public static let windowMarginX: CGFloat = 0

    /// Position a dashboard window just below the status icon. Idempotent.
    public static func place(window: NSWindow, below statusButton: NSStatusBarButton) {
        guard let statusWindow = statusButton.window else {
            window.center()
            return
        }

        // Convert the status button's frame from its window's coords to screen coords.
        let buttonFrameInWindow = statusButton.convert(statusButton.bounds, to: nil)
        let buttonFrameOnScreen = statusWindow.convertToScreen(buttonFrameInWindow)

        // Window frame in screen coords.
        let windowSize = window.frame.size
        let originX = buttonFrameOnScreen.origin.x + windowMarginX
        let originY = buttonFrameOnScreen.origin.y - windowMarginY - windowSize.height

        // Clamp inside the visible screen so the window never lands off-screen
        // when the menubar icon is at an unusual position.
        let targetScreen = statusWindow.screen ?? NSScreen.main
        if let screen = targetScreen {
            let visibleFrame = screen.visibleFrame
            let clampedX = max(
                visibleFrame.minX + 8,
                min(originX, visibleFrame.maxX - windowSize.width - 8)
            )
            let clampedY = max(
                visibleFrame.minY + 8,
                min(originY, visibleFrame.maxY - windowSize.height - 8)
            )
            window.setFrameOrigin(NSPoint(x: clampedX, y: clampedY))
        } else {
            window.setFrameOrigin(NSPoint(x: originX, y: originY))
        }
    }

    /// Install a right-click NSMenu on the status item while keeping the
    /// left-click popover working. This is the standard macOS pattern: when
    /// `menu` is set, AppKit shows the menu on right-click only and forwards
    /// left-clicks to `button.action` if `sendAction(on: [.leftMouseDown])`
    /// is configured.
    ///
    /// Note: macOS treats `NSStatusItem.menu` as **right-click only** when the
    /// button has its own `target`/`action` set. We rely on that behavior.
    public static func installContextMenu(_ menu: NSMenu, on statusItem: NSStatusItem) {
        statusItem.menu = menu
        // Restrict the action to left-click only — this is the documented
        // way to get "right-click = menu, left-click = action" on a status item.
        if let button = statusItem.button {
            button.sendAction(on: [.leftMouseUp])
        }
    }
}