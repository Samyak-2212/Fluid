//! SI newtype wrappers for dimensional correctness at the type level.
//!
//! All physical quantities crossing module boundaries must use these types.
//! Mixing raw `f64` for physical quantities is a bug — see physics_contract.md.
//!
//! Dimensional products (e.g. `Meters * Meters -> MetersSquared`) are
//! [UNRESOLVED: dimensional algebra]. Do not implement `Mul<Meters> for Meters`
//! without Tier A review.

use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Generates a SI newtype wrapper with standard arithmetic and formatting.
///
/// Each generated type implements:
/// `Debug`, `Clone`, `Copy`, `PartialEq`, `PartialOrd`,
/// `Add`, `Sub`, `Mul<f64>`, `Div<f64>`, `Neg`, `Display`.
macro_rules! si_unit {
    ($(#[$meta:meta])* $name:ident, $symbol:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        pub struct $name(pub f64);

        impl $name {
            /// Returns the raw `f64` value.
            #[inline]
            pub fn value(self) -> f64 {
                self.0
            }
        }

        impl Add for $name {
            type Output = $name;
            #[inline]
            fn add(self, rhs: $name) -> $name {
                $name(self.0 + rhs.0)
            }
        }

        impl Sub for $name {
            type Output = $name;
            #[inline]
            fn sub(self, rhs: $name) -> $name {
                $name(self.0 - rhs.0)
            }
        }

        impl Mul<f64> for $name {
            type Output = $name;
            #[inline]
            fn mul(self, rhs: f64) -> $name {
                $name(self.0 * rhs)
            }
        }

        impl Mul<$name> for f64 {
            type Output = $name;
            #[inline]
            fn mul(self, rhs: $name) -> $name {
                $name(self * rhs.0)
            }
        }

        impl Div<f64> for $name {
            type Output = $name;
            #[inline]
            fn div(self, rhs: f64) -> $name {
                $name(self.0 / rhs)
            }
        }

        impl Neg for $name {
            type Output = $name;
            #[inline]
            fn neg(self) -> $name {
                $name(-self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{} {}", self.0, $symbol)
            }
        }
    };
}

// ── SI base and derived units ────────────────────────────────────────────────

si_unit!(
    /// Length in metres (m).
    Meters, "m"
);

si_unit!(
    /// Mass in kilograms (kg).
    Kilograms, "kg"
);

si_unit!(
    /// Time in seconds (s).
    Seconds, "s"
);

si_unit!(
    /// Force in newtons (N = kg·m·s⁻²).
    Newtons, "N"
);

si_unit!(
    /// Pressure in pascals (Pa = N·m⁻²).
    Pascals, "Pa"
);

si_unit!(
    /// Mass density in kilograms per cubic metre (kg·m⁻³).
    KilogramsPerCubicMeter, "kg/m³"
);

si_unit!(
    /// Linear velocity in metres per second (m·s⁻¹).
    MetersPerSecond, "m/s"
);

si_unit!(
    /// Linear acceleration in metres per second squared (m·s⁻²).
    MetersPerSecondSquared, "m/s²"
);

si_unit!(
    /// Energy in joules (J = N·m).
    Joules, "J"
);

si_unit!(
    /// Power in watts (W = J·s⁻¹).
    Watts, "W"
);

si_unit!(
    /// Plane angle in radians (rad).
    Radians, "rad"
);

si_unit!(
    /// Angular velocity in radians per second (rad·s⁻¹).
    RadiansPerSecond, "rad/s"
);

si_unit!(
    /// Thermodynamic temperature in kelvins (K).
    Kelvin, "K"
);

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_meters() {
        assert_eq!(Meters(1.0) + Meters(2.0), Meters(3.0));
    }

    #[test]
    fn sub_meters() {
        assert_eq!(Meters(5.0) - Meters(3.0), Meters(2.0));
    }

    #[test]
    fn scale_meters() {
        assert_eq!(Meters(2.0) * 3.0, Meters(6.0));
        assert_eq!(3.0 * Meters(2.0), Meters(6.0));
    }

    #[test]
    fn div_meters() {
        assert_eq!(Meters(6.0) / 2.0, Meters(3.0));
    }

    #[test]
    fn neg_meters() {
        assert_eq!(-Meters(1.0), Meters(-1.0));
    }

    #[test]
    fn display_seconds() {
        assert_eq!(format!("{}", Seconds(1.5)), "1.5 s");
    }

    #[test]
    fn display_kelvin() {
        assert_eq!(format!("{}", Kelvin(273.15)), "273.15 K");
    }

    #[test]
    fn value_accessor() {
        assert_eq!(Kilograms(9.81).value(), 9.81_f64);
    }
}
