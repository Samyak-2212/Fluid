// [NEEDS_REVIEW: claude]
//! Compute FFI trait — Tier 3 only.
//!
//! Isolates all CUDA and ROCm FFI behind trait boundaries.
//! No crate outside C5 may depend on CUDA or ROCm directly.

/// Opaque GPU compute kernel handle.
#[cfg(feature = "tier_3")]
pub struct ComputeKernel {
    /// Kernel identifier (implementation-defined).
    pub id: u32,
}

/// Arguments for a GPU compute dispatch.
#[cfg(feature = "tier_3")]
pub struct KernelArgs {
    /// Raw byte buffer passed to the kernel.
    pub data: Vec<u8>,
    /// Work-group dimensions [x, y, z].
    pub work_groups: [u32; 3],
}

/// GPU compute backend trait — Tier 3 only.
///
/// All CUDA and ROCm backends implement this trait.
/// The concrete implementations live in `cuda_ffi.rs` and `rocm_ffi.rs`.
#[cfg(feature = "tier_3")]
pub trait GpuComputeBackend: Send + Sync {
    /// Dispatches a compute kernel.
    fn dispatch_kernel(
        &self,
        kernel: &ComputeKernel,
        args: &KernelArgs,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

// ── CUDA bridge ───────────────────────────────────────────────────────────────

/// CUDA compute backend — Tier 3, Linux or Windows.
///
/// # Safety
///
/// All unsafe blocks below interface with the CUDA runtime C API via FFI.
/// They are tagged [NEEDS_REVIEW: claude] and must be reviewed before
/// production deployment.
#[cfg(all(feature = "tier_3", any(target_os = "linux", target_os = "windows")))]
pub mod cuda_ffi {
    // [NEEDS_REVIEW: claude]
    use super::{ComputeKernel, GpuComputeBackend, KernelArgs};

    /// CUDA compute backend stub.
    ///
    /// Production implementation requires linking `libcuda.so` (Linux) or
    /// `nvcuda.dll` (Windows) via `build.rs` and generating FFI bindings
    /// with `bindgen` from `cuda.h`.
    ///
    /// The stub returns `Ok(())` to allow compilation without a CUDA runtime.
    pub struct CudaBackend {
        /// CUDA device ordinal (0-indexed).
        pub device: i32,
    }

    impl GpuComputeBackend for CudaBackend {
        fn dispatch_kernel(
            &self,
            _kernel: &ComputeKernel,
            _args: &KernelArgs,
        ) -> Result<(), Box<dyn std::error::Error>> {
            // [NEEDS_REVIEW: claude]
            // TODO: Replace with actual cuLaunchKernel / cuMemcpyHtoD / cuMemcpyDtoH
            // when CUDA runtime FFI bindings are added to build.rs.
            //
            // Safety requirements for production implementation:
            // 1. All CUdeviceptr values must be checked for null before use.
            // 2. cuMemAlloc / cuMemFree must be paired; use RAII wrapper.
            // 3. cuCtxSetCurrent must be called per thread if using multiple threads.
            // 4. All CUDA error codes must be checked — do not use .unwrap().
            Err("CudaBackend: stub not connected to CUDA runtime".into())
        }
    }
}

// ── ROCm/HIP bridge ───────────────────────────────────────────────────────────

/// ROCm/HIP compute backend — Tier 3, Linux only.
///
/// # Safety
///
/// All unsafe blocks below interface with the ROCm HIP runtime C API via FFI.
/// They are tagged [NEEDS_REVIEW: claude] and must be reviewed before
/// production deployment.
#[cfg(all(feature = "tier_3", target_os = "linux"))]
pub mod rocm_ffi {
    // [NEEDS_REVIEW: claude]
    use super::{ComputeKernel, GpuComputeBackend, KernelArgs};

    /// ROCm/HIP compute backend stub.
    ///
    /// Production implementation requires linking `libamdhip64.so` via `build.rs`
    /// and generating FFI bindings with `bindgen` from `hip/hip_runtime_api.h`.
    ///
    /// The stub returns `Ok(())` to allow compilation without a ROCm runtime.
    pub struct RocmBackend {
        /// HIP device ordinal (0-indexed).
        pub device: i32,
    }

    impl GpuComputeBackend for RocmBackend {
        fn dispatch_kernel(
            &self,
            _kernel: &ComputeKernel,
            _args: &KernelArgs,
        ) -> Result<(), Box<dyn std::error::Error>> {
            // [NEEDS_REVIEW: claude]
            // TODO: Replace with actual hipLaunchKernelGGL / hipMemcpy
            // when ROCm HIP runtime FFI bindings are added to build.rs.
            //
            // Safety requirements for production implementation:
            // 1. All hipDeviceptr_t values must be checked before use.
            // 2. hipMalloc / hipFree must be paired; use RAII wrapper.
            // 3. hipSetDevice must be called before any device operation.
            // 4. All hipError_t codes must be checked — do not use .unwrap().
            Err("RocmBackend: stub not connected to ROCm HIP runtime".into())
        }
    }
}
