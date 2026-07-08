// 3D vector math over f64.
// Components are (x, y, z). Operations include arithmetic, dot/cross product,
// length (L2 norm), normalization, distance, lerp, and basic component-wise
// helpers. All operations are pure and total; no unsafe code.
// std-only Rust.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const X_AXIS: Vec3 = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
    pub const Y_AXIS: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    pub const Z_AXIS: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    pub fn splat(v: f64) -> Self {
        Vec3 { x: v, y: v, z: v }
    }

    pub fn add(self, o: Self) -> Self {
        Vec3 { x: self.x + o.x, y: self.y + o.y, z: self.z + o.z }
    }

    pub fn sub(self, o: Self) -> Self {
        Vec3 { x: self.x - o.x, y: self.y - o.y, z: self.z - o.z }
    }

    pub fn scale(self, s: f64) -> Self {
        Vec3 { x: self.x * s, y: self.y * s, z: self.z * s }
    }

    /// Component-wise multiplication.
    pub fn mul(self, o: Self) -> Self {
        Vec3 { x: self.x * o.x, y: self.y * o.y, z: self.z * o.z }
    }

    pub fn neg(self) -> Self {
        Vec3 { x: -self.x, y: -self.y, z: -self.z }
    }

    pub fn dot(self, o: Self) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    pub fn cross(self, o: Self) -> Self {
        Vec3 {
            x: self.y * o.z - self.z * o.y,
            y: self.z * o.x - self.x * o.z,
            z: self.x * o.y - self.y * o.x,
        }
    }

    pub fn length_sq(self) -> f64 {
        self.dot(self)
    }

    pub fn length(self) -> f64 {
        self.length_sq().sqrt()
    }

    /// Returns `Some(normalized)` if length > 0, else `None`.
    pub fn try_normalize(self) -> Option<Self> {
        let l = self.length();
        if l == 0.0 {
            None
        } else {
            Some(self.scale(1.0 / l))
        }
    }

    /// `normalize` panics on zero length; use `try_normalize` for safety.
    pub fn normalize(self) -> Self {
        self.try_normalize()
            .expect("Vec3::normalize called on zero vector")
    }

    pub fn distance(self, o: Self) -> f64 {
        self.sub(o).length()
    }

    pub fn distance_sq(self, o: Self) -> f64 {
        self.sub(o).length_sq()
    }

    /// Linear interpolation: `self * (1 - t) + o * t`. No clamping.
    pub fn lerp(self, o: Self, t: f64) -> Self {
        self.scale(1.0 - t).add(o.scale(t))
    }

    pub fn min(self, o: Self) -> Self {
        Vec3 { x: self.x.min(o.x), y: self.y.min(o.y), z: self.z.min(o.z) }
    }

    pub fn max(self, o: Self) -> Self {
        Vec3 { x: self.x.max(o.x), y: self.y.max(o.y), z: self.z.max(o.z) }
    }

    pub fn abs(self) -> Self {
        Vec3 { x: self.x.abs(), y: self.y.abs(), z: self.z.abs() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    fn vec_approx_eq(a: Vec3, b: Vec3, eps: f64) -> bool {
        approx_eq(a.x, b.x, eps)
            && approx_eq(a.y, b.y, eps)
            && approx_eq(a.z, b.z, eps)
    }

    #[test]
    fn add_sub() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(a.add(b), Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(b.sub(a), Vec3::new(3.0, 3.0, 3.0));
    }

    #[test]
    fn dot_and_cross_basis() {
        // Standard basis: orthogonal unit vectors.
        let x = Vec3::X_AXIS;
        let y = Vec3::Y_AXIS;
        let z = Vec3::Z_AXIS;
        assert_eq!(x.dot(y), 0.0);
        assert_eq!(y.dot(z), 0.0);
        assert_eq!(x.dot(z), 0.0);
        assert_eq!(x.dot(x), 1.0);

        // Right-handed: x cross y = z, y cross z = x, z cross x = y.
        let xy = x.cross(y);
        assert!(vec_approx_eq(xy, z, 1e-12), "x*y = {:?}", xy);
        let yz = y.cross(z);
        assert!(vec_approx_eq(yz, x, 1e-12));
        let zx = z.cross(x);
        assert!(vec_approx_eq(zx, y, 1e-12));

        // Anti-commutative.
        let yx = y.cross(x);
        assert!(vec_approx_eq(yx, z.neg(), 1e-12));
    }

    #[test]
    fn length_normalize() {
        let v = Vec3::new(3.0, 4.0, 12.0); // 3-4-12 triangle, length 13.
        assert_eq!(v.length(), 13.0);
        let n = v.normalize();
        assert!(approx_eq(n.length(), 1.0, 1e-12));
    }

    #[test]
    fn zero_normalize_is_none() {
        assert!(Vec3::ZERO.try_normalize().is_none());
    }

    #[test]
    fn distance() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 6.0, 3.0);
        // sqrt(9 + 16 + 0) = 5
        assert_eq!(a.distance(b), 5.0);
        assert_eq!(a.distance_sq(b), 25.0);
    }

    #[test]
    fn lerp_endpoints_and_midpoint() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(10.0, 20.0, 30.0);
        assert!(vec_approx_eq(a.lerp(b, 0.0), a, 1e-12));
        assert!(vec_approx_eq(a.lerp(b, 1.0), b, 1e-12));
        assert!(vec_approx_eq(a.lerp(b, 0.5), Vec3::new(5.0, 10.0, 15.0), 1e-12));
    }

    #[test]
    fn splat_and_component_arith() {
        let v = Vec3::splat(2.0);
        assert_eq!(v, Vec3::new(2.0, 2.0, 2.0));
        let v2 = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.mul(v2), Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(v2.neg(), Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(v2.abs(), v2);
        assert_eq!(Vec3::new(-1.0, 2.0, -3.0).abs(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn min_max() {
        let a = Vec3::new(1.0, 4.0, -2.0);
        let b = Vec3::new(3.0, 2.0, 5.0);
        assert_eq!(a.min(b), Vec3::new(1.0, 2.0, -2.0));
        assert_eq!(a.max(b), Vec3::new(3.0, 4.0, 5.0));
    }

    #[test]
    fn triangle_area_via_cross() {
        // Triangle with vertices (0,0,0), (1,0,0), (0,1,0). Area = 0.5.
        let a = Vec3::ZERO;
        let b = Vec3::X_AXIS;
        let c = Vec3::Y_AXIS;
        let n = b.sub(a).cross(c.sub(a));
        // |n|/2 = area.
        assert!(approx_eq(n.length() / 2.0, 0.5, 1e-12));
    }
}
