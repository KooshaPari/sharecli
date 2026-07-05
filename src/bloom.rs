pub struct BloomFilter { bits: Vec<bool>, size: usize, hash_count: u32 }
impl BloomFilter {
    pub fn new(size: usize, hash_count: u32) -> Self { Self { bits: vec![false; size], size, hash_count } }
    fn hashes(&self, item: &str) -> Vec<usize> {
        let primary = simple_hash(item);
        (0..self.hash_count).map(|i| ((primary.wrapping_add(i as u64 * 31)) % self.size as u64) as usize).collect()
    }
    pub fn insert(&mut self, item: &str) { for h in self.hashes(item) { self.bits[h] = true; } }
    pub fn contains(&self, item: &str) -> bool { self.hashes(item).iter().all(|&h| self.bits[h]) }
    pub fn popcount(&self) -> usize { self.bits.iter().filter(|&&b| b).count() }
    pub fn size(&self) -> usize { self.size }
}
fn simple_hash(s: &str) -> u64 { let mut h: u64 = 0xcbf29ce484222325; for b in s.bytes() { h = h.wrapping_mul(0x100000001b3) ^ b as u64; } h }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_insert_contains() { let mut b = BloomFilter::new(64, 3); b.insert("hello"); assert!(b.contains("hello")); }
    #[test] fn test_miss() { let mut b = BloomFilter::new(64, 3); b.insert("hello"); assert!(!b.contains("world")); }
    #[test] fn test_popcount() { let mut b = BloomFilter::new(64, 3); b.insert("a"); b.insert("b"); assert!(b.popcount() > 0); }
    #[test] fn test_size() { let b = BloomFilter::new(128, 4); assert_eq!(b.size(), 128); }
}
