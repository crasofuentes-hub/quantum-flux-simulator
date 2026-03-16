use num_complex::Complex64;
use serde::Serialize;

const NUMERIC_FLOOR: f64 = 1e-12;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Density2 {
    pub rho00_re: f64,
    pub rho00_im: f64,
    pub rho01_re: f64,
    pub rho01_im: f64,
    pub rho10_re: f64,
    pub rho10_im: f64,
    pub rho11_re: f64,
    pub rho11_im: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct ComplexMatrix2 {
    pub a00: Complex64,
    pub a01: Complex64,
    pub a10: Complex64,
    pub a11: Complex64,
}

impl ComplexMatrix2 {
    pub fn zero() -> Self {
        Self {
            a00: Complex64::new(0.0, 0.0),
            a01: Complex64::new(0.0, 0.0),
            a10: Complex64::new(0.0, 0.0),
            a11: Complex64::new(0.0, 0.0),
        }
    }

    pub fn identity() -> Self {
        Self {
            a00: Complex64::new(1.0, 0.0),
            a01: Complex64::new(0.0, 0.0),
            a10: Complex64::new(0.0, 0.0),
            a11: Complex64::new(1.0, 0.0),
        }
    }

    pub fn dagger(&self) -> Self {
        Self {
            a00: self.a00.conj(),
            a01: self.a10.conj(),
            a10: self.a01.conj(),
            a11: self.a11.conj(),
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            a00: self.a00 + other.a00,
            a01: self.a01 + other.a01,
            a10: self.a10 + other.a10,
            a11: self.a11 + other.a11,
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        Self {
            a00: self.a00 - other.a00,
            a01: self.a01 - other.a01,
            a10: self.a10 - other.a10,
            a11: self.a11 - other.a11,
        }
    }

    pub fn mul_scalar(&self, s: Complex64) -> Self {
        Self {
            a00: self.a00 * s,
            a01: self.a01 * s,
            a10: self.a10 * s,
            a11: self.a11 * s,
        }
    }

    pub fn mul(&self, other: &Self) -> Self {
        Self {
            a00: self.a00 * other.a00 + self.a01 * other.a10,
            a01: self.a00 * other.a01 + self.a01 * other.a11,
            a10: self.a10 * other.a00 + self.a11 * other.a10,
            a11: self.a10 * other.a01 + self.a11 * other.a11,
        }
    }

    pub fn trace(&self) -> Complex64 {
        self.a00 + self.a11
    }

    pub fn determinant(&self) -> Complex64 {
        self.a00 * self.a11 - self.a01 * self.a10
    }

    pub fn commutator(&self, other: &Self) -> Self {
        self.mul(other).sub(&other.mul(self))
    }

    pub fn anticommutator(&self, other: &Self) -> Self {
        self.mul(other).add(&other.mul(self))
    }

    pub fn hermitianize(&self) -> Self {
        let d = self.dagger();
        self.add(&d).mul_scalar(Complex64::new(0.5, 0.0))
    }

    pub fn sanitize_finite(&self) -> Self {
        Self {
            a00: sanitize_complex(self.a00),
            a01: sanitize_complex(self.a01),
            a10: sanitize_complex(self.a10),
            a11: sanitize_complex(self.a11),
        }
    }

    pub fn normalize_trace(&self) -> Self {
        let tr = self.trace().re;
        let safe = if tr.is_finite() && tr.abs() >= NUMERIC_FLOOR {
            tr
        } else {
            1.0
        };
        self.mul_scalar(Complex64::new(1.0 / safe, 0.0))
    }

    pub fn stabilize(&self) -> Self {
        let sanitized = self.sanitize_finite();
        let mut out = sanitized.hermitianize().sanitize_finite();

        let mut d00 = out.a00.re;
        let mut d11 = out.a11.re;

        if !d00.is_finite() {
            d00 = 0.5;
        }
        if !d11.is_finite() {
            d11 = 0.5;
        }

        d00 = d00.max(NUMERIC_FLOOR);
        d11 = d11.max(NUMERIC_FLOOR);

        out.a00 = Complex64::new(d00, 0.0);
        out.a11 = Complex64::new(d11, 0.0);

        let max_coherence = (d00 * d11).max(0.0).sqrt();
        let off = out.a01;
        let off_norm = off.norm();

        if !off_norm.is_finite() || off_norm <= NUMERIC_FLOOR {
            out.a01 = Complex64::new(0.0, 0.0);
            out.a10 = Complex64::new(0.0, 0.0);
        } else {
            let capped = off_norm.min(max_coherence);
            let scale = capped / off_norm;
            let bounded = off * scale;
            out.a01 = bounded;
            out.a10 = bounded.conj();
        }

        out = out.normalize_trace().sanitize_finite().hermitianize();

        let tr = out.trace().re;
        if !tr.is_finite() || tr <= NUMERIC_FLOOR {
            return Self::identity().mul_scalar(Complex64::new(0.5, 0.0));
        }

        out.normalize_trace().sanitize_finite()
    }

    pub fn to_density2(&self) -> Density2 {
        Density2 {
            rho00_re: self.a00.re,
            rho00_im: self.a00.im,
            rho01_re: self.a01.re,
            rho01_im: self.a01.im,
            rho10_re: self.a10.re,
            rho10_im: self.a10.im,
            rho11_re: self.a11.re,
            rho11_im: self.a11.im,
        }
    }
}

pub fn initial_density_from_information_density(info_density: f64) -> ComplexMatrix2 {
    let p = info_density.clamp(0.05, 0.95);
    ComplexMatrix2 {
        a00: Complex64::new(p, 0.0),
        a01: Complex64::new(0.0, 0.0),
        a10: Complex64::new(0.0, 0.0),
        a11: Complex64::new(1.0 - p, 0.0),
    }
}

pub fn entropy_von_neumann_2x2(rho: &ComplexMatrix2) -> f64 {
    let stabilized = rho.stabilize();
    let tr = stabilized.trace().re;
    let det = stabilized.determinant().re;
    let disc = (tr * tr - 4.0 * det).max(0.0).sqrt();

    let l1 = ((tr + disc) * 0.5).max(NUMERIC_FLOOR);
    let l2 = ((tr - disc) * 0.5).max(NUMERIC_FLOOR);

    -(l1 * l1.ln() + l2 * l2.ln())
}

pub fn trace_distance_from_one(rho: &ComplexMatrix2) -> f64 {
    (rho.trace().re - 1.0).abs() + rho.trace().im.abs()
}

pub fn hermiticity_residual(rho: &ComplexMatrix2) -> f64 {
    let delta = rho.sub(&rho.dagger());
    delta.a00.norm() + delta.a01.norm() + delta.a10.norm() + delta.a11.norm()
}

pub fn has_only_finite_entries(rho: &ComplexMatrix2) -> bool {
    [
        rho.a00.re, rho.a00.im, rho.a01.re, rho.a01.im, rho.a10.re, rho.a10.im, rho.a11.re,
        rho.a11.im,
    ]
    .into_iter()
    .all(f64::is_finite)
}

pub fn min_eigenvalue_2x2(rho: &ComplexMatrix2) -> f64 {
    let stabilized = rho.hermitianize();
    let tr = stabilized.trace().re;
    let det = stabilized.determinant().re;
    let disc = (tr * tr - 4.0 * det).max(0.0).sqrt();
    (tr - disc) * 0.5
}

fn sanitize_complex(value: Complex64) -> Complex64 {
    Complex64::new(sanitize_scalar(value.re), sanitize_scalar(value.im))
}

fn sanitize_scalar(value: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{
        entropy_von_neumann_2x2, has_only_finite_entries, hermiticity_residual,
        initial_density_from_information_density, min_eigenvalue_2x2, trace_distance_from_one,
        ComplexMatrix2,
    };
    use num_complex::Complex64;

    #[test]
    fn stabilize_normalizes_trace_and_preserves_hermiticity() {
        let rho = ComplexMatrix2 {
            a00: Complex64::new(0.7, 0.3),
            a01: Complex64::new(0.4, -0.8),
            a10: Complex64::new(-0.1, 0.6),
            a11: Complex64::new(0.1, -0.2),
        };

        let stabilized = rho.stabilize();

        assert!(has_only_finite_entries(&stabilized));
        assert!(trace_distance_from_one(&stabilized) < 1e-9);
        assert!(hermiticity_residual(&stabilized) < 1e-9);
    }

    #[test]
    fn stabilize_sanitizes_non_finite_entries() {
        let rho = ComplexMatrix2 {
            a00: Complex64::new(f64::NAN, 0.0),
            a01: Complex64::new(f64::INFINITY, 1.0),
            a10: Complex64::new(1.0, f64::NEG_INFINITY),
            a11: Complex64::new(0.0, 0.0),
        };

        let stabilized = rho.stabilize();

        assert!(has_only_finite_entries(&stabilized));
        assert!(trace_distance_from_one(&stabilized) < 1e-9);
        assert!(hermiticity_residual(&stabilized) < 1e-9);
    }

    #[test]
    fn entropy_of_stabilized_density_is_non_negative() {
        let rho = initial_density_from_information_density(0.73);
        let entropy = entropy_von_neumann_2x2(&rho);

        assert!(entropy.is_finite());
        assert!(entropy >= 0.0);
    }

    #[test]
    fn stabilize_enforces_reasonable_positive_semidefinite_bound() {
        let rho = ComplexMatrix2 {
            a00: Complex64::new(0.2, 0.0),
            a01: Complex64::new(3.0, 4.0),
            a10: Complex64::new(3.0, -4.0),
            a11: Complex64::new(0.3, 0.0),
        };

        let stabilized = rho.stabilize();

        assert!(min_eigenvalue_2x2(&stabilized) >= -1e-9);
    }
}
