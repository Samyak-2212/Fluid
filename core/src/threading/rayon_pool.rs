// [REVIEWED: claude — C1 complete gate + C7 quality gate, 2026-05-02. No issues found.]
//! Rayon thread pool wrapper.

use super::traits::ThreadPool;

/// Wrapper around Rayon's global thread pool.
pub struct RayonPool;

impl RayonPool {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }
}

impl ThreadPool for RayonPool {
    fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        rayon::spawn(f);
    }
}
