// [NEEDS_REVIEW: claude]
//! EPA (Expanding Polytope Algorithm) — penetration depth and contact normal.
//!
//! Called after GJK confirms intersection.
//! Returns penetration depth ([`Meters`]) and contact normal (unit `Vec3`).
//!
//! # Algorithm source
//!
//! van den Bergen, G. (2001). "Proximity Queries and Penetration Depth
//! Computation on 3D Game Objects." Game Developers Conference proceedings.
//! <https://www.dtecta.com/papers/gdc2001depth.pdf>
//!
//! Ericson, C. (2004). *Real-Time Collision Detection.* Morgan Kaufmann.
//! Chapter 9.5 (EPA). ISBN 978-1558607323.
//!
//! EPA starts from the GJK simplex (tetrahedron enclosing origin) and
//! expands outward until convergence on the closest boundary face.

use glam::Vec3;
use core::units::Meters;
use crate::collision::traits::ConvexShape;
use crate::collision::gjk::gjk_intersect;

const EPA_MAX_ITERATIONS: usize = 64;
const EPA_TOLERANCE: f32 = 1e-4;

// ── Simplex input (re-exported from gjk) ─────────────────────────────────────

use crate::collision::gjk::Simplex;

// ── Polytope ──────────────────────────────────────────────────────────────────

struct Polytope {
    vertices: Vec<Vec3>,
    faces: Vec<[usize; 3]>,
}

impl Polytope {
    fn from_simplex(simplex: &Simplex) -> Self {
        assert!(simplex.size == 4);
        let v: Vec<Vec3> = (0..4).map(|i| simplex.points[i]).collect();
        let mut faces = vec![[0,1,2],[0,3,1],[0,2,3],[1,3,2]];
        for face in &mut faces {
            let a = v[face[0]]; let b = v[face[1]]; let c = v[face[2]];
            if (b-a).cross(c-a).dot(a) < 0.0 { face.swap(1,2); }
        }
        Polytope { vertices: v, faces }
    }

    fn face_normal_dist(&self, face: &[usize;3]) -> (Vec3, f32) {
        let a = self.vertices[face[0]]; let b = self.vertices[face[1]]; let c = self.vertices[face[2]];
        let n = (b-a).cross(c-a); let len = n.length();
        if len < f32::EPSILON { return (Vec3::Z, f32::MAX); }
        let n = n / len;
        (n, n.dot(a).abs())
    }

    fn closest_face(&self) -> (usize, Vec3, f32) {
        let mut min_d = f32::MAX; let mut idx = 0; let mut norm = Vec3::Z;
        for (i, face) in self.faces.iter().enumerate() {
            let (n, d) = self.face_normal_dist(face);
            if d < min_d { min_d = d; idx = i; norm = n; }
        }
        (idx, norm, min_d)
    }

    fn add_vertex(&mut self, p: Vec3) {
        let ni = self.vertices.len();
        self.vertices.push(p);
        let mut edges: Vec<(usize,usize)> = Vec::new();
        self.faces.retain(|face| {
            let a = self.vertices[face[0]]; let b = self.vertices[face[1]]; let c = self.vertices[face[2]];
            if (b-a).cross(c-a).dot(p-a) > 0.0 {
                Self::edge(&mut edges, face[0], face[1]);
                Self::edge(&mut edges, face[1], face[2]);
                Self::edge(&mut edges, face[2], face[0]);
                false
            } else { true }
        });
        for (a, b) in edges {
            let mut f = [a, b, ni];
            let na = self.vertices[f[0]]; let nb = self.vertices[f[1]]; let nc = self.vertices[f[2]];
            if (nb-na).cross(nc-na).dot(na) < 0.0 { f.swap(0,1); }
            self.faces.push(f);
        }
    }

    fn edge(edges: &mut Vec<(usize,usize)>, a: usize, b: usize) {
        if let Some(p) = edges.iter().position(|&(x,y)| x==b && y==a) { edges.remove(p); }
        else { edges.push((a,b)); }
    }
}

// ── Public result type ────────────────────────────────────────────────────────

/// Result of an EPA penetration query.
#[derive(Debug, Clone)]
pub struct EpaResult {
    /// Penetration depth in SI metres.
    pub depth: Meters,
    /// Contact normal (unit vector; direction from B toward A).
    pub normal: Vec3,
}

// ── EPA entry point ───────────────────────────────────────────────────────────

/// Computes penetration depth and contact normal for two intersecting shapes.
///
/// Returns `None` if GJK reports no intersection.
pub fn epa_penetration(a: &dyn ConvexShape, b: &dyn ConvexShape) -> Option<EpaResult> {
    let (intersects, mut simplex) = gjk_intersect(a, b);
    if !intersects { return None; }

    // Ensure we have a tetrahedron.
    for &dir in &[Vec3::X, Vec3::NEG_X, Vec3::Y, Vec3::NEG_Y, Vec3::Z, Vec3::NEG_Z] {
        if simplex.size >= 4 { break; }
        let s = a.support(dir) - b.support(-dir);
        let dup = (0..simplex.size).any(|i| (simplex.points[i]-s).length_squared() < 1e-8);
        if !dup { simplex.push(s); }
    }

    if simplex.size < 4 {
        return Some(EpaResult { depth: Meters(0.0), normal: Vec3::Y });
    }

    let mut poly = Polytope::from_simplex(&simplex);

    for _ in 0..EPA_MAX_ITERATIONS {
        let (_, normal, dist) = poly.closest_face();
        let support = a.support(normal) - b.support(-normal);
        let new_dist = support.dot(normal);
        if (new_dist - dist).abs() < EPA_TOLERANCE {
            return Some(EpaResult { depth: Meters(dist as f64), normal });
        }
        poly.add_vertex(support);
    }

    let (_, normal, dist) = poly.closest_face();
    Some(EpaResult { depth: Meters(dist as f64), normal })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    struct Sphere { centre: Vec3, radius: f32 }
    impl ConvexShape for Sphere {
        fn support(&self, d: Vec3) -> Vec3 {
            let l = d.length();
            if l < f32::EPSILON { return self.centre; }
            self.centre + d * (self.radius / l)
        }
    }

    /// Two spheres overlapping by 1 m: depth ≈ 1.0.
    #[test]
    fn sphere_penetration_depth_accurate() {
        let a = Sphere { centre: Vec3::ZERO, radius: 1.0 };
        let b = Sphere { centre: Vec3::new(1.0, 0.0, 0.0), radius: 1.0 };
        let r = epa_penetration(&a, &b).expect("overlapping spheres must have EPA result");
        let d = r.depth.value() as f32;
        assert!((d - 1.0).abs() < 0.05, "depth: expected ≈1.0, got {d}");
    }

    /// Normal must be roughly along X.
    #[test]
    fn sphere_contact_normal_direction() {
        let a = Sphere { centre: Vec3::ZERO, radius: 1.0 };
        let b = Sphere { centre: Vec3::new(1.0, 0.0, 0.0), radius: 1.0 };
        let r = epa_penetration(&a, &b).expect("should intersect");
        assert!(r.normal.x.abs() > 0.8, "normal should be ≈X, got {:?}", r.normal);
    }

    /// Non-overlapping → None.
    #[test]
    fn non_intersecting_returns_none() {
        let a = Sphere { centre: Vec3::ZERO, radius: 1.0 };
        let b = Sphere { centre: Vec3::new(5.0, 0.0, 0.0), radius: 1.0 };
        assert!(epa_penetration(&a, &b).is_none());
    }
}
