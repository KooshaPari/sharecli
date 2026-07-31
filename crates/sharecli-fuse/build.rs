//! Build script: WinFsp delay-load flags and macFUSE MFMount linkage.

fn main() {
    #[cfg(windows)]
    {
        winfsp::build::winfsp_link_delayload();
    }

    #[cfg(target_os = "macos")]
    {
        println!(
            "cargo:rustc-link-search=framework=/Library/Filesystems/macfuse.fs/Contents/Frameworks"
        );
        println!("cargo:rustc-link-lib=framework=MFMount");
    }
}
