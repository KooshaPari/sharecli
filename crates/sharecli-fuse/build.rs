//! Build script: WinFsp delay-load flags (AC-009.25).
fn main() {
    #[cfg(windows)]
    {
        winfsp::build::winfsp_link_delayload();
    }
}
