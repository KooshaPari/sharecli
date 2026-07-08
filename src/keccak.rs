// Keccak-p[1600,24] permutation + SHA3-256 + SHAKE128 (FIPS 202).
// std-only Rust. Reference vectors from NIST FIPS 202 + Keccak team.

// Keccak-p[1600] rho offsets: 5x5 rotation constants indexed row-major.
const RHO: [u32; 25] = [
    0, 1, 62, 28, 27, 36, 44, 6, 55, 20, 3, 10, 43, 25, 39, 41, 45, 15, 21,
    8, 18, 2, 61, 56, 14,
];
// Reference π lane permutation table (FIPS 202). The in-place ρ/π step below
// uses the closed-form (nx, ny) mapping instead of indexing this table.
#[allow(dead_code)]
const PI: [usize; 24] = [
    10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 19, 22, 23, 20, 4, 15, 13, 6,
    9, 2, 12, 14, 1,
];
const RC: [u64; 24] = [
    0x0000000000000001, 0x0000000000008082, 0x800000000000808a,
    0x8000000080008000, 0x000000000000808b, 0x0000000080000001,
    0x8000000080008081, 0x8000000000008009, 0x000000000000008a,
    0x0000000000000088, 0x0000000080008009, 0x000000008000000a,
    0x000000008000808b, 0x800000000000008b, 0x8000000000008089,
    0x8000000000008003, 0x8000000000008002, 0x8000000000000080,
    0x000000000000800a, 0x800000008000000a, 0x8000000080008081,
    0x8000000000008080, 0x0000000080000001, 0x8000000080008008,
];

#[inline]
fn rotl(x: u64, n: u32) -> u64 {
    (x << (n & 63)) | (x >> ((64 - n) & 63))
}

/// Apply Keccak-p[1600,24] in place.
pub fn keccak_p1600(state: &mut [u64; 25]) {
    for round in 0..24 {
        // θ
        let mut c = [0u64; 5];
        for x in 0..5 {
            c[x] = state[x] ^ state[x + 5] ^ state[x + 10] ^ state[x + 15] ^ state[x + 20];
        }
        let mut d = [0u64; 5];
        for x in 0..5 {
            d[x] = c[(x + 4) % 5] ^ rotl(c[(x + 1) % 5], 1);
        }
        for y in 0..5 {
            for x in 0..5 {
                state[x + 5 * y] ^= d[x];
            }
        }
        // ρ and π
        let mut b = [0u64; 25];
        for x in 0..5 {
            for y in 0..5 {
                let v = state[x + 5 * y];
                let nx = y;
                let ny = (2 * x + 3 * y) % 5;
                b[nx + 5 * ny] = rotl(v, RHO[5 * y + x]);
            }
        }
        // χ
        for y in 0..5 {
            for x in 0..5 {
                state[x + 5 * y] = b[x + 5 * y]
                    ^ ((!b[((x + 1) % 5) + 5 * y]) & b[((x + 2) % 5) + 5 * y]);
            }
        }
        // ι
        state[0] ^= RC[round];
    }
}

/// Absorb then squeeze. `d` is the SHA-3 capacity in bytes (e.g. 64 for SHA3-256,
/// 32 for SHAKE128).
fn sponge(rate: usize, d: usize, input: &[u8], suffix: u8, outlen: usize) -> Vec<u8> {
    assert!(rate + d == 200);
    let mut state = [0u64; 25];
    // pad with suffix + 0x80 ... 0x01 then process rate-sized blocks
    let mut buf = input.to_vec();
    buf.push(suffix);
    let pad_len = (rate - (buf.len() % rate)) % rate;
    if pad_len == 0 {
        // Need at least one byte of padding; add a full block of zeros except last = 0x80
        buf.extend(std::iter::repeat(0u8).take(rate - 1));
        buf.push(0x80);
    } else {
        buf.extend(std::iter::repeat(0u8).take(pad_len - 1));
        buf.push(0x80);
    }
    let lanes = rate / 8;
    for chunk in buf.chunks(rate) {
        for i in 0..lanes {
            let off = i * 8;
            state[i] ^= u64::from_le_bytes([
                chunk[off], chunk[off + 1], chunk[off + 2], chunk[off + 3],
                chunk[off + 4], chunk[off + 5], chunk[off + 6], chunk[off + 7],
            ]);
        }
        keccak_p1600(&mut state);
    }
    let mut out = Vec::with_capacity(outlen);
    while out.len() < outlen {
        for i in 0..lanes {
            if out.len() >= outlen {
                break;
            }
            let b = state[i].to_le_bytes();
            out.push(b[0]);
            if out.len() < outlen {
                out.push(b[1]);
            }
            if out.len() < outlen {
                out.push(b[2]);
            }
            if out.len() < outlen {
                out.push(b[3]);
            }
            if out.len() < outlen {
                out.push(b[4]);
            }
            if out.len() < outlen {
                out.push(b[5]);
            }
            if out.len() < outlen {
                out.push(b[6]);
            }
            if out.len() < outlen {
                out.push(b[7]);
            }
        }
        if out.len() < outlen {
            keccak_p1600(&mut state);
        }
    }
    out.truncate(outlen);
    out
}

/// SHA3-256(M) hex digest (lowercase).
pub fn sha3_256_hex(input: &[u8]) -> String {
    let out = sponge(136, 64, input, 0x06, 32);
    let mut s = String::with_capacity(64);
    for b in out {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// SHA3-256(M) raw bytes.
pub fn sha3_256(input: &[u8]) -> [u8; 32] {
    let v = sponge(136, 64, input, 0x06, 32);
    let mut a = [0u8; 32];
    a.copy_from_slice(&v);
    a
}

/// SHAKE128(M) -> outlen bytes.
pub fn shake128(input: &[u8], outlen: usize) -> Vec<u8> {
    sponge(168, 32, input, 0x1f, outlen)
}

#[cfg(test)]
mod tests {
    use super::*;

    // NIST CAVP test vectors.
    #[test]
    fn sha3_256_empty() {
        assert_eq!(
            sha3_256_hex(b""),
            "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a"
        );
    }
    #[test]
    fn sha3_256_abc() {
        assert_eq!(
            sha3_256_hex(b"abc"),
            "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532"
        );
    }
    #[test]
    fn sha3_256_64bytes() {
        // FIPS 202 reference vector: bytes 0x00..0x3F.
        let pt: Vec<u8> = (0..64).collect();
        assert_eq!(
            sha3_256_hex(&pt),
            "c8ad478f4e1dd9d47dfc3b985708d92db1f8db48fe9cddd459e63c321f490402"
        );
    }

    #[test]
    fn shake128_short() {
        // SHAKE128("", 32) -> 7f9c2ba4e88f827d616045507605853e d73b8093f6efbc88eb1a6eacfa66ef26
        let v = shake128(b"", 32);
        let s: String = v.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(
            s,
            "7f9c2ba4e88f827d616045507605853ed73b8093f6efbc88eb1a6eacfa66ef26"
        );
    }

    #[test]
    fn keccak_p1600_known_state() {
        // After Keccak-p[1600,24] on the all-zero state, the first lane is
        // f125_8f79_40e1_dde7 (reference: Keccak team TestVectors files).
        let mut state = [0u64; 25];
        keccak_p1600(&mut state);
        assert_eq!(state[0], 0xf125_8f79_40e1_dde7);
    }
}
