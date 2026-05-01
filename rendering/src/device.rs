//! rendering/src/device.rs
//!
//! [REVIEWED: claude — BUG-008 closed, 2026-05-01. No architecture issues found.]
//! wgpu device/adapter/queue initialization.
//! Tier A work: architecture, adapter selection, feature set negotiation.
//!
//! This module provides a headless wgpu `Device` + `Queue` pair, suitable for
//! compute-only or offscreen rendering (the IDX HTTP preview path).  A
//! windowed surface is created separately in `surface.rs` when a window handle
//! is available.
//!
//! Supported backends (wgpu selects automatically at runtime):
//!   - Vulkan   (Linux, Windows, Android)
//!   - DX12     (Windows)
//!   - Metal    (macOS, iOS)
//!   - GL/ANGLE (fallback)
//!
//! Do NOT add manual backend selection — that violates the wgpu strategy.

use wgpu::{Adapter, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits, Queue, RequestAdapterOptions};

/// Holds the wgpu instance, chosen adapter, logical device, and command queue.
pub struct GpuContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl GpuContext {
    /// Initialise wgpu with the best available backend on the current platform.
    ///
    /// Returns `Err` if no suitable adapter is found (e.g., running in a
    /// headless CI environment with no GPU).  Callers must fall back to the
    /// Tier 0 softbuffer path in that case.
    pub async fn init() -> Result<Self, InitError> {
        let instance = Instance::new(InstanceDescriptor {
            // Let wgpu pick the best available backend for the platform.
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: Default::default(),
        });

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| InitError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("fluid_rendering_device"),
                    required_features: Features::empty(),
                    required_limits: Limits::downlevel_defaults(),
                    memory_hints: Default::default(),
                    experimental_features: Default::default(),
                    trace: wgpu::Trace::Off,
                },
            )
            .await
            .map_err(InitError::DeviceRequest)?;

        Ok(Self { instance, adapter, device, queue })
    }

    /// Returns a human-readable description of the chosen adapter.
    pub fn adapter_info(&self) -> String {
        let info = self.adapter.get_info();
        format!("{} ({:?}) — backend: {:?}", info.name, info.device_type, info.backend)
    }
}

/// Errors that can occur during GPU context initialisation.
#[derive(Debug)]
pub enum InitError {
    /// No adapter matched the requested options (no GPU available).
    NoAdapter,
    /// Adapter found but device creation failed.
    DeviceRequest(wgpu::RequestDeviceError),
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::NoAdapter => write!(f, "no wgpu adapter found (no GPU available)"),
            InitError::DeviceRequest(e) => write!(f, "wgpu device request failed: {e}"),
        }
    }
}

impl std::error::Error for InitError {}
