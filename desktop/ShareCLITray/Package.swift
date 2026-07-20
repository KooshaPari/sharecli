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

        // Main tray + app target
        .executableTarget(
            name: "ShareCLITray",
            dependencies: ["ShareCLICore", "CShareCLIFFI"],
            path: "Sources/ShareCLITray",
            resources: [
                .copy("../../../../assets/icons/sharecli.iconset"),
                .copy("../../../../assets/icons/sharecli-256x256.png"),
                .copy("../../../../assets/icons/sharecli-512x512.png"),
            ]
            // Link libsharecli_ffi via just build-tray-macos / desktop/build.sh -Xlinker flags.
        ),

        // Shared core (IPC client, data models)
        .target(
            name: "ShareCLICore",
            dependencies: [],
            path: "Sources/ShareCLICore"
        ),
    ]
)
