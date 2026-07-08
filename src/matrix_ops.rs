// Square matrix operations over `f64`. Operations: add, subtract, scalar
// multiply, matrix multiply, transpose, trace, determinant (LU).
// std-only Rust.

use std::fmt;

/// Row-major dense matrix.
#[derive(Clone, Debug, PartialEq)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

impl Matrix {
    /// Zero matrix of given shape.
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Matrix { rows, cols, data: vec![0.0; rows * cols] }
    }
    /// Identity matrix (rows == cols required).
    pub fn identity(n: usize) -> Self {
        let mut m = Matrix::zeros(n, n);
        for i in 0..n {
            m.set(i, i, 1.0);
        }
        m
    }
    /// Construct from row-major data.
    pub fn from_vec(rows: usize, cols: usize, data: Vec<f64>) -> Self {
        assert_eq!(data.len(), rows * cols, "data length mismatch");
        Matrix { rows, cols, data }
    }
    #[inline]
    fn idx(&self, r: usize, c: usize) -> usize {
        r * self.cols + c
    }
    #[inline]
    pub fn get(&self, r: usize, c: usize) -> f64 {
        self.data[self.idx(r, c)]
    }
    #[inline]
    pub fn set(&mut self, r: usize, c: usize, v: f64) {
        let i = self.idx(r, c);
        self.data[i] = v;
    }
    /// Element-wise add. Returns `None` on shape mismatch.
    pub fn add(&self, other: &Matrix) -> Option<Matrix> {
        if self.rows != other.rows || self.cols != other.cols {
            return None;
        }
        let mut out = Matrix::zeros(self.rows, self.cols);
        for i in 0..self.data.len() {
            out.data[i] = self.data[i] + other.data[i];
        }
        Some(out)
    }
    /// Element-wise subtract.
    pub fn sub(&self, other: &Matrix) -> Option<Matrix> {
        if self.rows != other.rows || self.cols != other.cols {
            return None;
        }
        let mut out = Matrix::zeros(self.rows, self.cols);
        for i in 0..self.data.len() {
            out.data[i] = self.data[i] - other.data[i];
        }
        Some(out)
    }
    /// Scalar multiply.
    pub fn scale(&self, s: f64) -> Matrix {
        let mut out = self.clone();
        for v in out.data.iter_mut() {
            *v *= s;
        }
        out
    }
    /// Standard matrix-matrix multiply. Returns `None` on shape mismatch.
    pub fn mul(&self, other: &Matrix) -> Option<Matrix> {
        if self.cols != other.rows {
            return None;
        }
        let mut out = Matrix::zeros(self.rows, other.cols);
        for i in 0..self.rows {
            for k in 0..self.cols {
                let a = self.data[i * self.cols + k];
                if a == 0.0 {
                    continue;
                }
                for j in 0..other.cols {
                    out.data[i * other.cols + j] += a * other.data[k * other.cols + j];
                }
            }
        }
        Some(out)
    }
    /// Transpose.
    pub fn transpose(&self) -> Matrix {
        let mut out = Matrix::zeros(self.cols, self.rows);
        for r in 0..self.rows {
            for c in 0..self.cols {
                out.data[c * self.rows + r] = self.data[r * self.cols + c];
            }
        }
        out
    }
    /// Trace of a square matrix.
    pub fn trace(&self) -> Option<f64> {
        if self.rows != self.cols {
            return None;
        }
        let mut t = 0.0;
        for i in 0..self.rows {
            t += self.data[i * self.cols + i];
        }
        Some(t)
    }
    /// Determinant via in-place LU with partial pivoting. Returns `None` for
    /// non-square.
    pub fn det(&self) -> Option<f64> {
        if self.rows != self.cols {
            return None;
        }
        let n = self.rows;
        let mut a = self.data.clone();
        let mut sign = 1.0_f64;
        for i in 0..n {
            // partial pivot
            let mut piv = i;
            let mut maxv = a[i * n + i].abs();
            for r in (i + 1)..n {
                let v = a[r * n + i].abs();
                if v > maxv {
                    maxv = v;
                    piv = r;
                }
            }
            if maxv == 0.0 {
                return Some(0.0);
            }
            if piv != i {
                for c in 0..n {
                    let t = a[i * n + c];
                    a[i * n + c] = a[piv * n + c];
                    a[piv * n + c] = t;
                }
                sign = -sign;
            }
            let pv = a[i * n + i];
            for r in (i + 1)..n {
                let factor = a[r * n + i] / pv;
                for c in i..n {
                    a[r * n + c] -= factor * a[i * n + c];
                }
            }
        }
        let mut d = sign;
        for i in 0..n {
            d *= a[i * n + i];
        }
        Some(d)
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for r in 0..self.rows {
            for c in 0..self.cols {
                write!(f, "{:>10.4} ", self.get(r, c))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn add_sub() {
        let a = Matrix::from_vec(2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        let b = Matrix::from_vec(2, 2, vec![5.0, 6.0, 7.0, 8.0]);
        let s = a.add(&b).unwrap();
        assert!(approx_eq(s.get(0, 0), 6.0));
        assert!(approx_eq(s.get(1, 1), 12.0));
        let d = b.sub(&a).unwrap();
        assert!(approx_eq(d.get(0, 0), 4.0));
    }

    #[test]
    fn mul_identity() {
        let a = Matrix::from_vec(2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        let i = Matrix::identity(2);
        let r = a.mul(&i).unwrap();
        assert!(approx_eq(r.get(0, 0), 1.0));
        assert!(approx_eq(r.get(1, 1), 4.0));
    }

    #[test]
    fn mul_general() {
        let a = Matrix::from_vec(2, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let b = Matrix::from_vec(3, 2, vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0]);
        let r = a.mul(&b).unwrap();
        assert_eq!(r.rows, 2);
        assert_eq!(r.cols, 2);
        assert!(approx_eq(r.get(0, 0), 58.0));  // 1*7+2*9+3*11
        assert!(approx_eq(r.get(0, 1), 64.0));  // 1*8+2*10+3*12
        assert!(approx_eq(r.get(1, 0), 139.0)); // 4*7+5*9+6*11
        assert!(approx_eq(r.get(1, 1), 154.0));
    }

    #[test]
    fn trace_transpose() {
        let a = Matrix::from_vec(3, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        assert!(approx_eq(a.trace().unwrap(), 15.0));
        let t = a.transpose();
        assert!(approx_eq(t.get(0, 1), 4.0));
        assert!(approx_eq(t.get(2, 0), 3.0));
    }

    #[test]
    fn det_3x3() {
        let a = Matrix::from_vec(
            3,
            3,
            vec![6.0, 1.0, 1.0, 4.0, -2.0, 5.0, 2.0, 8.0, 7.0],
        );
        // det = 6*(-2*7 - 5*8) - 1*(4*7 - 5*2) + 1*(4*8 - (-2)*2)
        //     = 6*(-54) - 1*18 + 1*36 = -324 - 18 + 36 = -306
        assert!(approx_eq(a.det().unwrap(), -306.0));
    }

    #[test]
    fn det_singular() {
        let a = Matrix::from_vec(2, 2, vec![1.0, 2.0, 2.0, 4.0]);
        assert!(approx_eq(a.det().unwrap(), 0.0));
    }
}
