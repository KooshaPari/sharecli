pub struct TextSlab { entries: Vec<Option<(u32, String)>>, free: Vec<u32>, next_id: u32 }
impl TextSlab {
    pub fn new() -> Self { Self { entries: Vec::new(), free: Vec::new(), next_id: 0 } }
    pub fn insert(&mut self, s: impl Into<String>) -> u32 {
        if let Some(id) = self.free.pop() {
            self.entries[id as usize] = Some((id, s.into()));
            return id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(Some((id, s.into())));
        id
    }
    pub fn get(&self, id: u32) -> Option<&str> { self.entries.get(id as usize).and_then(|e| e.as_ref()).map(|(_, s)| s.as_str()) }
    pub fn remove(&mut self, id: u32) -> bool {
        if self.entries.get(id as usize).is_some() {
            self.entries[id as usize] = None;
            self.free.push(id);
            true
        } else { false }
    }
    pub fn len(&self) -> usize { self.entries.iter().filter(|e| e.is_some()).count() }
    pub fn capacity(&self) -> usize { self.entries.len() }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_insert_get() { let mut s = TextSlab::new(); let id = s.insert("hello"); assert_eq!(s.get(id), Some("hello")); }
    #[test] fn test_recycle() { let mut s = TextSlab::new(); let id1 = s.insert("a"); let id2 = s.insert("b"); s.remove(id1); let id3 = s.insert("c"); assert_eq!(id3, id1); assert_eq!(s.get(id2), Some("b")); }
    #[test] fn test_remove_invalid() { let mut s = TextSlab::new(); assert!(!s.remove(99)); }
    #[test] fn test_len() { let mut s = TextSlab::new(); s.insert("x"); s.insert("y"); assert_eq!(s.len(), 2); }
}
