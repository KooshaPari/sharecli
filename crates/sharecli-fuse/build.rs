fn main() {
    #[cfg(target_os = "macos")]
    {
        println!(
            "cargo:rustc-link-search=framework=/Library/Filesystems/macfuse.fs/Contents/Frameworks"
        );
        println!("cargo:rustc-link-lib=framework=MFMount");
    }
}
