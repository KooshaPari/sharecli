// Catalan numbers C_n = (1 / (n+1)) * binomial(2n, n).
// First values: 1, 1, 2, 5, 14, 42, 132, 429, 1430, 4862, ...
// We compute them iteratively using the recurrence:
//     C_0 = 1
//     C_{n+1} = C_n * 2 * (2n + 1) / (n + 2)
// which stays in u64 until n = 35 (C_35 = 3116285494907301262).
// Beyond that, values are returned as `Option::None` (overflow).
// std-only Rust.

/// Return `Option<C_n>` for `n`. `None` signals an overflow at u64.
pub fn catalan_u64(n: u32) -> Option<u64> {
    if n == 0 {
        return Some(1);
    }
    // Compute in u128 so the intermediate `c * 2 * (2i + 1)` does not
    // overflow even when the final C_n fits in u64 (C_35 = 3116285494907301262
    // fits, but the intermediate at i = 34 is ~1.12e20 which exceeds u64).
    let mut c: u128 = 1;
    for i in 0..n {
        // c_{i+1} = c_i * 2 * (2i + 1) / (i + 2)
        let two_i_plus_1 = (2u128 * i as u128) + 1;
        let numerator = c.checked_mul(2)?.checked_mul(two_i_plus_1)?;
        let denom = i as u128 + 2;
        if numerator % denom != 0 {
            // The Catalan recurrence guarantees integer division, but be safe.
            return None;
        }
        c = numerator / denom;
    }
    u64::try_from(c).ok()
}

/// Return `Option<C_n>` as a u128. Catalan numbers fit in u128 up to n = 75
/// (C_75 = 4533804494341229440 < 2^62; C_76 > 2^65 so we can fit up to ~88).
pub fn catalan_u128(n: u32) -> Option<u128> {
    if n == 0 {
        return Some(1);
    }
    let mut c: u128 = 1;
    for i in 0..n {
        let two_i_plus_1 = (2u128 * i as u128) + 1;
        let numerator = c.checked_mul(2)?.checked_mul(two_i_plus_1)?;
        let denom = i as u128 + 2;
        if numerator % denom != 0 {
            return None;
        }
        c = numerator / denom;
    }
    Some(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_eleven() {
        // OEIS A000108: 1, 1, 2, 5, 14, 42, 132, 429, 1430, 4862, 16796
        let expected: [u64; 11] = [1, 1, 2, 5, 14, 42, 132, 429, 1430, 4862, 16796];
        for (i, &want) in expected.iter().enumerate() {
            let got = catalan_u64(i as u32).expect("small n");
            assert_eq!(got, want, "C_{} = {} expected {}", i, got, want);
        }
    }

    #[test]
    fn closed_form_agrees() {
        // For small n, cross-check against binomial formula.
        // C_n = binom(2n, n) - binom(2n, n+1).
        fn binom_u128(n: u32, k: u32) -> Option<u128> {
            // binom(n, k) = prod_{j=0..k} (n-j)/(j+1)
            let mut res: u128 = 1;
            let k = k.min(n - k);
            for j in 0..k {
                let num = (n - j) as u128;
                let den = (j + 1) as u128;
                // Multiply by num, divide by den after gcd-reduction would be ideal
                // but a direct approach is fine for small n.
                res = res.checked_mul(num)?;
                // We need exact division by den at each step to stay integral.
                if res % den != 0 {
                    // Factor out any gcd to keep the running product integer.
                    let g = gcd(res, den);
                    if g > 1 {
                        res /= g;
                        let den2 = den / g;
                        if res % den2 != 0 {
                            return None;
                        }
                        res /= den2;
                    } else {
                        return None;
                    }
                } else {
                    res /= den;
                }
            }
            Some(res)
        }
        fn gcd(a: u128, b: u128) -> u128 {
            let (mut a, mut b) = (a, b);
            while b != 0 {
                let t = b;
                b = a % b;
                a = t;
            }
            a
        }

        for n in 0u32..10 {
            let lhs = catalan_u128(n).expect("should fit");
            // C_n = binom(2n, n) / (n + 1) is the exact closed form for n >= 0.
            let total = binom_u128(2 * n, n).expect("binom2n,n fits");
            let denom = (n + 1) as u128;
            assert_eq!(total % denom, 0, "n={}: binom(2n,n) not divisible by n+1", n);
            let want = total / denom;
            assert_eq!(lhs, want, "n={}", n);
        }
    }

    #[test]
    fn large_u64_boundary() {
        // Largest n whose catalan fits in u64 is 35.
        // Verify some values we know.
        let c30 = catalan_u64(30).expect("C30");
        assert_eq!(c30, 3814986502092304);
        let c35 = catalan_u64(35).expect("C35");
        assert_eq!(c35, 3116285494907301262);
    }

    #[test]
    fn u128_extends_range() {
        // Up to ~88 fits in u128; verify a known value.
        // C_50 = 1978261657756160653623774456 (OEIS A000108).
        let c50 = catalan_u128(50).expect("C50");
        assert_eq!(c50, 1978261657756160653623774456u128);
    }
}
