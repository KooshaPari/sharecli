// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "ShareCLITray",
    platforms: [.macOS(.v14)],
    products: [
        .executable(name: "ShareCLITray", targets: ["ShareCLITray"]),
    ],
    targets: [
        // Thin C wrapper target so Swift can import the Rust FFI header.
        // The header + modulemap live inside Sources/CShareCLIFFI/ so the
        // package is self-contained; a copy is kept under desktop/include/
        // for non-SPM consumers (e.g. Xcode project / cmake).
        .target(
            name: "CShareCLIFFI",
            path: "Sources/CShareCLIFFI",
            publicHeadersPath: "."
        ),

        // Main tray + app target. Icon resources are bundled into the .app
        // post-build by scripts/install-tray-macos.sh (which copies
        // assets/icons/sharecli.icns into Contents/Resources/AppIcon.icns).
        // We intentionally do NOT declare them here — SwiftPM requires
        // resources to live inside the package, but icons live one
        // directory up at sharecli/assets/.
        .executableTarget(
            name: "ShareCLITray",
            dependencies: ["ShareCLICore", "CShareCLIFFI"],
            path: "Sources/ShareCLITray"
            // Link libsharecli_ffi via desktop/build.sh -Xlinker flags.
        ),

        // Shared core (IPC client, data models)
        .target(
            name: "ShareCLICore",
            dependencies: [],
            path: "Sources/ShareCLICore"
        ),
    ]
)
