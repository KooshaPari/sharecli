const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // --- Static library consumed by Rust via build.rs / extern "C" ---
    const lib_module = b.createModule(.{
        .root_source_file = b.path("src/spawn_core.zig"),
        .target = target,
        .optimize = optimize,
    });
    lib_module.link_libc = true;

    const lib = b.addLibrary(.{
        .name = "spawn_core",
        .root_module = lib_module,
        .linkage = .static,
    });
    // Explicit libc link for Darwin (Zig 0.14+ no longer always auto-links
    // libSystem when producing archives that pull POSIX via std.c).
    lib.linkLibC();
    // Do NOT set bundle_compiler_rt: on macOS that forces a full link of the
    // archive objects and fails with undefined _getcwd/_fork/… when Zig cannot
    // resolve libSystem into the static archive step. Rustc supplies
    // compiler-rt when linking the final binary.
    b.installArtifact(lib);

    // --- Test step: `zig build test` ---
    const test_module = b.createModule(.{
        .root_source_file = b.path("src/spawn_core.zig"),
        .target = target,
        .optimize = optimize,
    });
    test_module.link_libc = true;

    const unit_tests = b.addTest(.{
        .root_module = test_module,
    });
    unit_tests.linkLibC();

    const run_tests = b.addRunArtifact(unit_tests);
    const test_step = b.step("test", "Run spawn-core unit tests");
    test_step.dependOn(&run_tests.step);
}
