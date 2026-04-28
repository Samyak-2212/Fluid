//! rendering/src/camera.rs
//!
//! Camera holds view and projection matrices plus position in SI Meters.
//! Used by SceneRenderer to produce the view-projection transform.

use fluid_core::math::{Mat4, Vec3};
use fluid_core::units::Meters;

/// Perspective camera.
///
/// `position` is in SI Meters (from `core::units`).
/// `view` and `projection` are column-major matrices (glam convention).
#[derive(Debug, Clone)]
pub struct Camera {
    /// World-space position in SI Meters.
    pub position: [Meters; 3],
    /// View matrix (world → camera space).
    pub view: Mat4,
    /// Projection matrix (camera → clip space).
    pub projection: Mat4,
    /// Horizontal field of view in radians.
    pub fov_radians: f32,
    /// Near clip plane distance (metres).
    pub near: f32,
    /// Far clip plane distance (metres).
    pub far: f32,
}

impl Camera {
    /// Constructs a default perspective camera at the origin looking along -Z.
    pub fn default_perspective(aspect: f32) -> Self {
        let fov = std::f32::consts::FRAC_PI_4; // 45°
        let near = 0.1_f32;
        let far = 1000.0_f32;
        Self {
            position: [Meters(0.0), Meters(0.0), Meters(0.0)],
            view: Mat4::IDENTITY,
            projection: Mat4::perspective_rh(fov, aspect, near, far),
            fov_radians: fov,
            near,
            far,
        }
    }

    /// Returns the combined view-projection matrix.
    pub fn view_projection(&self) -> Mat4 {
        self.projection * self.view
    }
}
