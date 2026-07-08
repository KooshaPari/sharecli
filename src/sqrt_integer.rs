// Integer square root using Newton's method.
// For non-negative inputs, returns floor(sqrt(n)). Newton's iteration on
// x_{k+1} = floor((x_k + n / x_k) / 2) converges quadratically when n > 0.
// When n is 0 returns 0. Overflow detection: if `x_k * x_k` would overflow u64,
// we report u64::MAX to signal the caller that 64-bit ops are insufficient.
// std-only Rust.

/// Compute the integer square root: `floor(sqrt(n))` for `n: u64`.
///
/// Returns `u64::MAX` when `n` is too large for the result to fit in `u64`
/// (i.e. `n >= (u64::MAX)^2`); this signals an overflow rather than
/// silently clamping or panicking.
pub fn isqrt(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    // Standard integer Newton: x = n is a safe upper bound (since sqrt(n) <= n
    // for n >= 1). y = (x + n/x) / 2 monotonically decreases until y < x
    // fails, at which point x is the previous (smaller) y -- the floor sqrt.
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        // Stop iterating when division would be 0 (n < x so n/x = 0, fixed
        // point reached) or when y is no longer decreasing.
        y = (x + n / x) / 2;
    }
    // x is floor(sqrt(n)) for n <= (u64::MAX - 1)^2; for larger n the
    // initial upper bound n itself is the answer and we return it, which
    // may exceed the true floor. Detect overflow: if x*x overflows, n is
    // too large to fit a 64-bit sqrt -- return u64::MAX as the overflow
    // sentinel.
    if x.checked_mul(x).is_none() {
        return u64::MAX;
    }
    x
}

/// Compute `floor(sqrt(n))` as a u128 when the answer needs more than 64 bits.
pub fn isqrt_u128(n: u128) -> u128 {
    if n <= 1 {
        return n;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    if x.checked_mul(x).is_none() {
        return u128::MAX;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_and_one() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt_u128(0), 0);
        assert_eq!(isqrt_u128(1), 1);
    }

    #[test]
    fn perfect_squares() {
        // (k, k^2) for k in 0..=20
        for k in 0u64..=20 {
            let sq = k * k;
            assert_eq!(isqrt(sq), k, "isqrt({}) should be {}", sq, k);
        }
    }

    #[test]
    fn non_perfect_squares() {
        // Spot-check known floors of sqrt.
        assert_eq!(isqrt(2), 1);
        assert_eq!(isqrt(3), 1);
        assert_eq!(isqrt(4), 2);
        assert_eq!(isqrt(8), 2);
        assert_eq!(isqrt(9), 3);
        assert_eq!(isqrt(15), 3);
        assert_eq!(isqrt(16), 4);
        assert_eq!(isqrt(99), 9);
        assert_eq!(isqrt(100), 10);
        assert_eq!(isqrt(15_000), 122);
        assert_eq!(isqrt(1_000_000), 1000);
    }

    #[test]
    fn large_value() {
        // 2^60 = 1152921504606846976, sqrt = 2^30 = 1073741824
        assert_eq!(isqrt(1u64 << 60), 1u64 << 30);
        // 2^62 + 5 -> floor sqrt = 2^31
        assert_eq!(isqrt((1u64 << 62) + 5), 1u64 << 31);
    }

    #[test]
    fn relation_holds_for_range() {
        // For all n in 0..10_000, isqrt(n)^2 <= n < (isqrt(n)+1)^2
        for n in 0u64..10_000 {
            let s = isqrt(n);
            assert!(s <= n, "isqrt({}) returned {} > n", n, s);
            if s < u64::MAX {
                assert!(s * s <= n, "isqrt({})^2 = {} > n", n, s * s);
                if s + 1 <= (u64::MAX >> 1) {
                    assert!((s + 1) * (s + 1) > n || s + 1 > s,
                        "isqrt({})={} should be maximal", n, s);
                }
            }
        }
    }

    #[test]
    fn u128_perfect_squares() {
        for k in 0u128..30 {
            let sq = k * k;
            assert_eq!(isqrt_u128(sq), k);
        }
        // 2^60 squared
        let sq = 1u128 << 120;
        assert_eq!(isqrt_u128(sq), 1u128 << 60);
    }

    #[test]
    fn u128_floors_correctly() {
        // (10^9)^2 = 10^18, sqrt = 10^9.
        assert_eq!(isqrt_u128(1_000_000_000_000_000_000u128), 1_000_000_000u128);
        assert_eq!(isqrt_u128(1_000_000_000_000_000_001u128), 1_000_000_000u128);
    }
}
