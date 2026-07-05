pub struct SortedSet<T: Ord + Clone> { items: Vec<T> }
impl<T: Ord + Clone> SortedSet<T> {
    pub fn new() -> Self { Self { items: Vec::new() } }
    pub fn insert(&mut self, v: T) -> bool {
        match self.items.binary_search(&v) { Ok(_) => false, Err(p) => { self.items.insert(p, v); true } }
    }
    pub fn contains(&self, v: &T) -> bool { self.items.binary_search(v).is_ok() }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn remove(&mut self, v: &T) -> bool {
        match self.items.binary_search(v) { Ok(p) => { self.items.remove(p); true }, Err(_) => false }
    }
    pub fn iter(&self) -> std::slice::Iter<T> { self.items.iter() }
    pub fn to_vec(&self) -> Vec<T> { self.items.clone() }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn insert_dedup() { let mut s = SortedSet::new(); assert!(s.insert(1)); assert!(!s.insert(1)); assert_eq!(s.len(), 1); }
    #[test] fn sorted_iteration() { let mut s = SortedSet::new(); s.insert(3); s.insert(1); s.insert(2); let v: Vec<i32> = s.iter().copied().collect(); assert_eq!(v, vec![1, 2, 3]); }
    #[test] fn contains() { let mut s = SortedSet::new(); s.insert(5); assert!(s.contains(&5)); assert!(!s.contains(&3)); }
    #[test] fn remove() { let mut s = SortedSet::new(); s.insert(1); s.insert(2); assert!(s.remove(&1)); assert_eq!(s.to_vec(), vec![2]); }
    #[test] fn remove_missing() { let mut s = SortedSet::new(); s.insert(1); assert!(!s.remove(&99)); }
}
