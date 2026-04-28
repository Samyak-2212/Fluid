//! Thread pool trait — interface definition.
//!
//! Concrete implementation (`RayonPool`) lives in `rayon_pool.rs`.
//! rayon 1.12.0 verified on docs.rs (2026-04-28); `rayon::spawn` available via rayon-core.

/// Interface for a work-item thread pool.
///
/// Concrete implementations wrap rayon or a custom scheduler behind this
/// trait so the execution backend can be swapped without touching callers.
pub trait ThreadPool: Send + Sync {
    /// Submits a work item to the pool. The item may execute on any worker thread.
    fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static;
}
