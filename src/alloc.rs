//! Global allocator selection for long-running `sharecli serve` (C00 L8).
//!
//! - `jemalloc` feature: production serve/container builds on Unix (not MSVC).
//! - `dhat-heap` feature: dev-only heap profiling (mutually exclusive with `jemalloc`).
//!
//! Contract: `docs/ops/memory.md` · `docs/ops/alloc-profiling.md`

#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static DHAT: dhat::Alloc = dhat::Alloc;

/// Returns the active allocator label for operator diagnostics (`sharecli --version` path).
pub fn active_allocator_label() -> &'static str {
    if cfg!(feature = "dhat-heap") {
        "dhat"
    } else if cfg!(all(feature = "jemalloc", not(target_env = "msvc"))) {
        "jemalloc"
    } else {
        "system"
    }
}

#[cfg(test)]
mod tests {
    use super::active_allocator_label;

    #[test]
    fn default_allocator_is_system() {
        assert_eq!(active_allocator_label(), "system");
    }
}
