// Iterative point-update + range-sum Fenwick tree, and a sparse segment tree
// for arbitrary monoidal range-fold queries. std-only Rust, no unsafe.

/// Fenwick tree (Binary Indexed Tree) over `u64`. 1-indexed internally.
#[derive(Clone, Debug)]
pub struct Fenwick {
    bit: Vec<u64>,
    n: usize,
}

impl Fenwick {
    /// Build from a vector of values. Operates in O(n).
    pub fn from_vec(values: &[u64]) -> Self {
        let n = values.len();
        let mut bit = vec![0u64; n + 1];
        for (i, &v) in values.iter().enumerate() {
            let mut j = i + 1;
            while j <= n {
                bit[j] = bit[j].wrapping_add(v);
                j += j & (!j + 1); // j += lowest set bit
            }
        }
        Fenwick { bit, n }
    }
    /// Update position `idx` by adding `delta`. `idx` is 0-based.
    pub fn add(&mut self, idx: usize, delta: u64) {
        let mut j = idx + 1;
        while j <= self.n {
            self.bit[j] = self.bit[j].wrapping_add(delta);
            j += j & (!j + 1);
        }
    }
    /// Set position `idx` to `value`. Caller supplies the prior value.
    pub fn set(&mut self, idx: usize, prior: u64, value: u64) {
        self.add(idx, value.wrapping_sub(prior));
    }
    /// Prefix sum of indices `[0, idx]`. `idx` is 0-based; pass `usize::MAX` to sum all.
    pub fn prefix_sum(&self, idx: usize) -> u64 {
        let limit = if idx >= self.n { self.n } else { idx + 1 };
        let mut j = limit;
        let mut sum = 0u64;
        while j > 0 {
            sum = sum.wrapping_add(self.bit[j]);
            j -= j & (!j + 1);
        }
        sum
    }
    /// Range sum `[l, r]` inclusive. Both 0-based.
    pub fn range_sum(&self, l: usize, r: usize) -> u64 {
        if l == 0 {
            self.prefix_sum(r)
        } else if l > r {
            0
        } else {
            self.prefix_sum(r).wrapping_sub(self.prefix_sum(l - 1))
        }
    }
    /// Lower bound: smallest index `i` such that `prefix_sum(i) >= target`.
    /// If `target <= 0`, returns 0. If no such index exists, returns `n`.
    pub fn lower_bound(&self, target: u64) -> usize {
        if target == 0 {
            return 0;
        }
        let mut j = 0usize;
        let mut bit = if self.n == 0 { 0 } else { 1usize << (63 - self.n.leading_zeros()) };
        let mut sum = 0u64;
        while bit > 0 {
            let next = j + bit;
            if next <= self.n && sum.wrapping_add(self.bit[next]) < target {
                sum = sum.wrapping_add(self.bit[next]);
                j = next;
            }
            bit >>= 1;
        }
        j.min(self.n)
    }
    pub fn len(&self) -> usize {
        self.n
    }
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
}

/// Type-erased monoid segment tree. Supports arbitrary `T: Copy + Ord` and any
/// closed binary operation `op: T -> T -> T` together with an identity.
pub struct SegTree<T> {
    tree: Vec<Option<T>>,
    n: usize,
    op: Box<dyn Fn(T, T) -> T + Send + Sync>,
}

impl<T: Copy + Ord + Default + std::fmt::Debug> std::fmt::Debug for SegTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SegTree").field("n", &self.n).field("tree", &self.tree).finish()
    }
}

impl<T: Copy + Ord + Default> SegTree<T> {
    /// Build from initial values and the monoid op. Caller passes identity
    /// implicitly via `default()`.
    pub fn build<F>(values: &[T], op: F) -> Self
    where
        F: Fn(T, T) -> T + Send + Sync + 'static,
    {
        let n = if values.is_empty() { 1 } else { values.len().next_power_of_two() };
        let mut tree: Vec<Option<T>> = vec![None; 2 * n];
        for (i, v) in values.iter().enumerate() {
            tree[n + i] = Some(*v);
        }
        for i in (1..n).rev() {
            tree[i] = match (tree[2 * i], tree[2 * i + 1]) {
                (Some(a), Some(b)) => Some(op(a, b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };
        }
        SegTree { tree, n, op: Box::new(op) }
    }

    /// Range fold over `values[l..=r]`. Caller must pass the monoid identity
    /// (e.g. `i64::MAX` for `min`). Empty range returns the identity.
    pub fn query_with(&self, l: usize, r: usize, identity: T) -> T {
        let mut acc = identity;
        let mut l = l + self.n;
        let mut r = r + self.n + 1;
        while l < r {
            if l & 1 == 1 {
                if let Some(v) = self.tree[l] {
                    acc = (self.op)(acc, v);
                }
                l += 1;
            }
            if r & 1 == 1 {
                r -= 1;
                if let Some(v) = self.tree[r] {
                    acc = (self.op)(acc, v);
                }
            }
            l /= 2;
            r /= 2;
        }
        acc
    }

    /// Range fold using `T::default()` as the identity (correct for `sum`, may
    /// give nonsensical results for `min`/`max`).
    pub fn query(&self, l: usize, r: usize) -> T {
        let id: T = T::default();
        self.query_with(l, r, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fenwick_basic() {
        let ft = Fenwick::from_vec(&[1, 2, 3, 4, 5]);
        assert_eq!(ft.range_sum(0, 4), 15);
        assert_eq!(ft.range_sum(1, 3), 9);
        assert_eq!(ft.range_sum(2, 2), 3);
    }

    #[test]
    fn fenwick_update() {
        let mut ft = Fenwick::from_vec(&[1, 2, 3, 4, 5]);
        ft.add(2, 7); // [1,2,10,4,5]
        assert_eq!(ft.range_sum(0, 4), 22);
        assert_eq!(ft.range_sum(2, 2), 10);
    }

    #[test]
    fn fenwick_lower_bound() {
        let ft = Fenwick::from_vec(&[3, 1, 4, 1, 5, 9, 2, 6]);
        // prefix sums: 3, 4, 8, 9, 14, 23, 25, 31
        assert_eq!(ft.lower_bound(0), 0);
        assert_eq!(ft.lower_bound(1), 0);
        assert_eq!(ft.lower_bound(3), 0);
        assert_eq!(ft.lower_bound(4), 1);
        assert_eq!(ft.lower_bound(5), 2);
        assert_eq!(ft.lower_bound(14), 4);
        assert_eq!(ft.lower_bound(15), 5);
        assert_eq!(ft.lower_bound(31), 7);
        assert_eq!(ft.lower_bound(32), 8);
    }

    #[test]
    fn segtree_min() {
        let st: SegTree<i64> = SegTree::build(&[5, 1, 7, 3, 9, 2, 6, 4], |a, b| a.min(b));
        // min identity is i64::MAX, NOT i64::default (which is 0).
        assert_eq!(st.query_with(0, 7, i64::MAX), 1);
        assert_eq!(st.query_with(2, 4, i64::MAX), 3);
        assert_eq!(st.query_with(0, 0, i64::MAX), 5);
        assert_eq!(st.query_with(7, 7, i64::MAX), 4);
        assert_eq!(st.query_with(1, 1, i64::MAX), 1);
    }

    #[test]
    fn segtree_max() {
        let st: SegTree<i64> = SegTree::build(&[5, 1, 7, 3, 9, 2, 6, 4], |a, b| a.max(b));
        assert_eq!(st.query(0, 7), 9);
        assert_eq!(st.query(0, 2), 7);
    }
}
