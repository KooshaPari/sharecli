#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Rational { pub num: i64, pub den: i64 }
impl Rational {
    pub fn new(num: i64, den: i64) -> Self {
        if den == 0 { return Self { num: 0, den: 1 }; }
        Self { num, den }
    }
    pub fn reduce(self) -> Self {
        let g = gcd(self.num.abs(), self.den.abs());
        if g == 0 { return self; }
        Self { num: self.num / g, den: self.den / g }
    }
    pub fn add(self, other: Self) -> Self {
        Self { num: self.num * other.den + other.num * self.den, den: self.den * other.den }.reduce()
    }
    pub fn to_f64(self) -> f64 { self.num as f64 / self.den as f64 }
}
fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a, b);
    while b != 0 { let t = b; b = a % b; a = t; }
    a
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn add() { let r = Rational::new(1, 2).add(Rational::new(1, 3)); assert_eq!(r, Rational::new(5, 6)); }
    #[test] fn add_overflow() { let r = Rational::new(1, 4).add(Rational::new(3, 4)); assert_eq!(r, Rational::new(1, 1)); }
    #[test] fn reduce() { assert_eq!(Rational::new(4, 8).reduce(), Rational::new(1, 2)); }
    #[test] fn zero_den() { assert_eq!(Rational::new(1, 0), Rational { num: 0, den: 1 }); }
    #[test] fn to_float() { assert!((Rational::new(1, 2).to_f64() - 0.5).abs() < 1e-9); }
    #[test] fn gcd_zero() { assert_eq!(gcd(0, 5), 5); }
}
