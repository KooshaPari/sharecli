pub struct DSU { parent: Vec<usize>, rank: Vec<usize> }
impl DSU {
    pub fn new(n: usize) -> Self { Self { parent: (0..n).collect(), rank: vec![0; n] } }
    pub fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x { let r = self.find(self.parent[x]); self.parent[x] = r; } self.parent[x]
    }
    pub fn union(&mut self, x: usize, y: usize) {
        let rx = self.find(x); let ry = self.find(y);
        if rx == ry { return; }
        match self.rank[rx].cmp(&self.rank[ry]) {
            std::cmp::Ordering::Less => self.parent[rx] = ry,
            std::cmp::Ordering::Greater => self.parent[ry] = rx,
            std::cmp::Ordering::Equal => { self.parent[ry] = rx; self.rank[rx] += 1; }
        }
    }
    pub fn connected(&mut self, x: usize, y: usize) -> bool { self.find(x) == self.find(y) }
    pub fn size(&self) -> usize { self.parent.len() }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn new_size() { let d = DSU::new(10); assert_eq!(d.size(), 10); }
    #[test] fn union_connected() { let mut d = DSU::new(5); d.union(0, 1); assert!(d.connected(0, 1)); }
    #[test] fn not_connected() { let mut d = DSU::new(5); assert!(!d.connected(0, 1)); }
    #[test] fn chain_union() { let mut d = DSU::new(5); d.union(0, 1); d.union(1, 2); assert!(d.connected(0, 2)); }
    #[test] fn separate_components() { let mut d = DSU::new(5); d.union(0, 1); d.union(2, 3); assert!(!d.connected(0, 2)); }
}
