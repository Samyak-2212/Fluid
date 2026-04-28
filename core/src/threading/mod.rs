//! Thread pool trait.
//!
//! Wraps `rayon` behind a trait so it can be swapped.
//! Do not implement a custom thread pool — evaluate rayon first.

pub mod traits;
pub mod rayon_pool;

pub use rayon_pool::RayonPool;
pub use traits::ThreadPool;
