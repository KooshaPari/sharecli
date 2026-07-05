use std::collections::HashMap;

pub struct Lru<K: std::hash::Hash + Eq + Clone, V> { map: HashMap<K, V>, order: Vec<K>, capacity: usize }
impl<K: std::hash::Hash + Eq + Clone, V> Lru<K, V> {
    pub fn new(capacity: usize) -> Self { Self { map: HashMap::new(), order: Vec::new(), capacity } }
    pub fn put(&mut self, k: K, v: V) {
        if self.map.contains_key(&k) {
            self.map.insert(k.clone(), v);
            self.order.retain(|x| x != &k);
            self.order.push(k);
        } else {
            if self.order.len() >= self.capacity {
                if let Some(evicted) = self.order.first().cloned() { self.order.remove(0); self.map.remove(&evicted); }
            }
            self.map.insert(k.clone(), v);
            self.order.push(k);
        }
    }
    pub fn get(&mut self, k: &K) -> Option<&V> {
        if self.map.contains_key(k) {
            self.order.retain(|x| x != k);
            self.order.push(k.clone());
            self.map.get(k)
        } else { None }
    }
    pub fn len(&self) -> usize { self.map.len() }
    pub fn capacity(&self) -> usize { self.capacity }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn put_get() { let mut l = Lru::new(3); l.put("a", 1); assert_eq!(l.get(&"a"), Some(&1)); }
    #[test] fn evicts_oldest() { let mut l = Lru::new(2); l.put("a",1); l.put("b",2); l.put("c",3); assert_eq!(l.get(&"a"), None); }
    #[test] fn get_refreshes() { let mut l = Lru::new(2); l.put("a",1); l.put("b",2); l.get(&"a"); l.put("c",3); assert_eq!(l.get(&"b"), None); }
    #[test] fn update() { let mut l = Lru::new(3); l.put("a", 1); l.put("a", 2); assert_eq!(l.get(&"a"), Some(&2)); }
    #[test] fn capacity_bounded() { let mut l = Lru::new(2); for i in 0..10 { l.put(format!("k{}",i), i); } assert_eq!(l.len(), 2); }
}
