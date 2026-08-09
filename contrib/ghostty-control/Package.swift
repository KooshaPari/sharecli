// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "ShareCLIGhosttyControl",
    platforms: [.macOS(.v13)],
    products: [
        .library(name: "ShareCLIGhosttyControl", targets: ["ShareCLIGhosttyControl"]),
    ],
    targets: [
        .target(name: "ShareCLIGhosttyControl"),
        .testTarget(name: "ShareCLIGhosttyControlTests", dependencies: ["ShareCLIGhosttyControl"]),
    ]
)
