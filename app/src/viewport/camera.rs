//! Camera for the 3D viewport.
//!
//! Orbit / pan / zoom model (spherical coordinates around a target).
//! - Left-mouse drag → orbit (azimuth + elevation)
//! - Right-mouse drag → pan (target translation)
//! - Scroll wheel → zoom (radius change)

use glam::{Mat4, Vec3};

// ── Camera ────────────────────────────────────────────────────────────────────

/// Orbit camera state.
///
/// Internally uses spherical coordinates: `radius`, `azimuth` (horizontal
/// angle), `elevation` (vertical angle). `position` is derived on demand.
#[derive(Debug, Clone)]
pub struct Camera {
    /// World-space point the camera orbits around.
    pub target: Vec3,
    /// Distance from target to eye.
    pub radius: f32,
    /// Horizontal orbit angle, radians.
    pub azimuth: f32,
    /// Vertical orbit angle, radians (clamped to avoid gimbal lock).
    pub elevation: f32,
    /// Vertical field of view, degrees.
    pub fov_deg: f32,
    /// Near clip plane.
    pub near: f32,
    /// Far clip plane.
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target:    Vec3::ZERO,
            radius:    6.0,
            azimuth:   std::f32::consts::FRAC_PI_4,       // 45°
            elevation: std::f32::consts::FRAC_PI_4 * 0.6, // ~27°
            fov_deg:   60.0,
            near:      0.01,
            far:       1000.0,
        }
    }
}

impl Camera {
    /// Returns the eye position in world space.
    pub fn eye(&self) -> Vec3 {
        let x = self.radius * self.elevation.cos() * self.azimuth.sin();
        let y = self.radius * self.elevation.sin();
        let z = self.radius * self.elevation.cos() * self.azimuth.cos();
        self.target + Vec3::new(x, y, z)
    }

    /// Returns the view-projection matrix for the given viewport dimensions.
    ///
    /// Uses a right-handed coordinate system with Z pointing towards the viewer.
    /// `width` and `height` are the viewport pixel dimensions.
    pub fn view_proj(&self, width: f32, height: f32) -> Mat4 {
        let aspect = if height > 0.0 { width / height } else { 1.0 };
        let eye = self.eye();
        let view = Mat4::look_at_rh(eye, self.target, Vec3::Y);
        let proj = Mat4::perspective_rh(
            self.fov_deg.to_radians(),
            aspect,
            self.near,
            self.far,
        );
        proj * view
    }

    // ── Orbit controls ────────────────────────────────────────────────────

    /// Apply a mouse-drag delta to orbit the camera.
    ///
    /// `dx` / `dy` are pixel deltas. Sensitivity is ~0.01 rad/px.
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        const SENSITIVITY: f32 = 0.01;
        self.azimuth -= dx * SENSITIVITY;
        self.elevation = (self.elevation + dy * SENSITIVITY)
            .clamp(-std::f32::consts::FRAC_PI_2 + 0.05,
                    std::f32::consts::FRAC_PI_2 - 0.05);
    }

    /// Apply a mouse-drag delta to pan the camera target.
    ///
    /// `dx` / `dy` are pixel deltas. Pan speed scales with radius.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let sensitivity = self.radius * 0.002;
        // Right and up vectors in world space.
        let forward = (self.target - self.eye()).normalize();
        let right   = forward.cross(Vec3::Y).normalize();
        let up      = right.cross(forward).normalize();
        self.target -= right * (dx * sensitivity) + up * (dy * sensitivity);
    }

    /// Apply a scroll delta to zoom (change radius).
    pub fn zoom(&mut self, delta: f32) {
        self.radius = (self.radius - delta * 0.5).clamp(0.1, 500.0);
    }
}
