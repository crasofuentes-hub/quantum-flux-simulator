use num_complex::Complex64;
use serde::Serialize;

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

    pub fn normalize_trace(&self) -> Self {
        let tr = self.trace().re;
        let safe = if tr.abs() < 1e-12 { 1.0 } else { tr };
        self.mul_scalar(Complex64::new(1.0 / safe, 0.0))
    }

    pub fn stabilize(&self) -> Self {
        let mut out = self.hermitianize();
        if out.a00.re < 1e-9 {
            out.a00 = Complex64::new(1e-9, 0.0);
        }
        if out.a11.re < 1e-9 {
            out.a11 = Complex64::new(1e-9, 0.0);
        }
        out.normalize_trace()
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
    let tr = (rho.a00 + rho.a11).re;
    let det = (rho.a00 * rho.a11 - rho.a01 * rho.a10).re;
    let disc = (tr * tr - 4.0 * det).max(0.0).sqrt();

    let l1 = ((tr + disc) * 0.5).max(1e-12);
    let l2 = ((tr - disc) * 0.5).max(1e-12);

    -(l1 * l1.ln() + l2 * l2.ln())
}
