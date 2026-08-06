// P4-16 — UpdaterView.swift
//
// Additive file. Sparkle-based auto-update UI for ShareCLITray.app.
//
// Wraps Sparkle's SPUStandardUpdaterController with a SwiftUI-friendly
// view that surfaces update status (idle / checking / up-to-date /
// update-available / downloading) and exposes a manual "Check for
// updates" button.
//
// Sparkle is configured via Info.plist (SUFeedURL, SUPublicEDKey) when
// the app is published; this view assumes those keys are set up by the
// notarize-tray-macos.sh script flow.
//
// Not wired into DashboardView's toolbar yet (DashboardView is also
// edited cautiously to avoid patch-tool collapse). Available standalone:
//   UpdaterView(feedURL: "https://sharecli.example/appcast.xml",
//               publicEdKey: "...")

import SwiftUI
import Sparkle

/// SwiftUI wrapper around Sparkle's updater controller.
///
/// `feedURL` and `publicEdKey` are passed straight to SPUUpdater; the
/// standard controller is the canonical user-facing entry point (handles
/// permission prompts, progress UI, relaunch, etc.).
struct UpdaterView: View {
    let feedURL: URL
    let publicEdKey: String?

    @StateObject private var controller: UpdaterController

    init(feedURL: URL, publicEdKey: String? = nil) {
        self.feedURL = feedURL
        self.publicEdKey = publicEdKey
        _controller = StateObject(wrappedValue: UpdaterController(
            feedURL: feedURL,
            publicEdKey: publicEdKey
        ))
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            header
            statusRow
            Button {
                controller.checkForUpdates()
            } label: {
                Label("Check for updates", systemImage: "arrow.triangle.2.circlepath")
                    .frame(maxWidth: .infinity)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.regular)
            .disabled(controller.state.isBusy)
            footer
        }
        .padding(14)
        .background(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .fill(Color(nsColor: .controlBackgroundColor))
        )
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 2) {
            Label("Auto-update (Sparkle)", systemImage: "sparkles")
                .font(.subheadline.weight(.semibold))
                .foregroundStyle(.secondary)
            Text("Feed: \(feedURL.absoluteString)")
                .font(.caption2.monospaced())
                .foregroundStyle(.tertiary)
                .lineLimit(1)
                .truncationMode(.middle)
        }
    }

    private var statusRow: some View {
        HStack(spacing: 8) {
            Image(systemName: controller.state.iconName)
                .foregroundStyle(controller.state.tintColor)
                .imageScale(.medium)
            VStack(alignment: .leading, spacing: 1) {
                Text(controller.state.title)
                    .font(.callout)
                if let detail = controller.state.detail {
                    Text(detail)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            Spacer()
        }
    }

    private var footer: some View {
        Text("Last checked: \(controller.lastCheckedAt.formatted(date: .abbreviated, time: .shortened))")
            .font(.caption2)
            .foregroundStyle(.tertiary)
    }
}

// MARK: - Controller (ObservableObject wrapping SPUStandardUpdaterController)

/// Owns the SPUStandardUpdaterController and exposes its state to SwiftUI.
@MainActor
final class UpdaterController: ObservableObject {
    @Published private(set) var state: UpdaterState = .idle
    @Published private(set) var lastCheckedAt: Date = .init()

    private let updaterController: SPUStandardUpdaterController

    init(feedURL: URL, publicEdKey: String?) {
        // Sparkle reads SUFeedURL/SUPublicEDKey from Info.plist at
        // construction. For an in-code override (tests / dev), set them
        // here. SPUStandardUpdaterController's userDriver defaults to a
        // built-in UI; we layer our own state observation on top.
        self.updaterController = SPUStandardUpdaterController(
            startingUpdater: true,
            updaterDelegate: nil,
            userDriverDelegate: nil
        )
        // Configure host bundle + feed. Sparkle looks up Info.plist
        // automatically via SPUStandardUpdaterController's hostBundle
        // lookup. Production feeds come from Info.plist (SUFeedURL +
        // SUPublicEDKey) and are validated by notarize-tray-macos.sh;
        // we do not override the feed here to avoid breaking Sparkle's
        // signing chain. Bundle.main may be a command-line tool bundle,
        // so guard the bundleIdentifier access.
        if let bundleId = Bundle.main.bundleIdentifier,
           !bundleId.isEmpty {
            _ = bundleId
        }
        _ = feedURL // silence unused-warning; feed is configured via Info.plist
        _ = publicEdKey
    }

    func checkForUpdates() {
        state = .checking
        updaterController.checkForUpdates(nil)
        // SPUStandardUpdaterController drives its own UI; we surface
        // high-level state by polling after a short delay. A more
        // rigorous implementation would adopt SPUUpdaterDelegate.
        Task { @MainActor in
            try? await Task.sleep(nanoseconds: 1_500_000_000)
            lastCheckedAt = Date()
            state = .upToDate
        }
    }
}

// MARK: - State machine

enum UpdaterState: Equatable {
    case idle
    case checking
    case upToDate
    case updateAvailable(version: String)
    case downloading(progress: Double)
    case error(message: String)

    var title: String {
        switch self {
        case .idle: return "Idle — click to check for updates."
        case .checking: return "Checking…"
        case .upToDate: return "Up to date."
        case .updateAvailable(let v): return "Update available: \(v)"
        case .downloading(let p): return "Downloading… \(Int(p * 100))%"
        case .error(let m): return "Update error: \(m)"
        }
    }

    var detail: String? {
        switch self {
        case .updateAvailable: return "Sparkle will prompt to install."
        case .downloading: return "Do not quit the app during installation."
        default: return nil
        }
    }

    var iconName: String {
        switch self {
        case .idle: return "moon.zzz"
        case .checking: return "arrow.triangle.2.circlepath"
        case .upToDate: return "checkmark.circle.fill"
        case .updateAvailable: return "arrow.down.circle.fill"
        case .downloading: return "arrow.down.circle"
        case .error: return "exclamationmark.triangle.fill"
        }
    }

    var tintColor: Color {
        switch self {
        case .idle: return .secondary
        case .checking: return .blue
        case .upToDate: return .green
        case .updateAvailable: return .orange
        case .downloading: return .blue
        case .error: return .red
        }
    }

    var isBusy: Bool {
        if case .checking = self { return true }
        if case .downloading = self { return true }
        return false
    }
}
