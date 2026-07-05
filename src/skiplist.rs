use std::cell::RefCell;
use std::rc::Rc;

const MAX_LEVEL: usize = 16;
const P: f64 = 0.5;

struct Node<K, V> {
    key: Option<K>,
    value: Option<V>,
    forward: Vec<Option<Rc<RefCell<Node<K, V>>>>>,
}

pub struct SkipList<K: Ord + Clone, V: Clone> {
    head: Rc<RefCell<Node<K, V>>>,
    level: usize,
    rng_state: RefCell<u64>,
    len: usize,
}

fn next_rand(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

impl<K: Ord + Clone, V: Clone> SkipList<K, V> {
    pub fn new() -> Self {
        let head = Rc::new(RefCell::new(Node {
            key: None,
            value: None,
            forward: vec![None; MAX_LEVEL],
        }));
        Self { head, level: 0, rng_state: RefCell::new(0xdeadbeef_cafebabe), len: 0 }
    }
    fn random_level(&self) -> usize {
        let mut lvl = 0;
        let mut s = self.rng_state.borrow_mut();
        while lvl < MAX_LEVEL - 1 {
            let r = next_rand(&mut s);
            let u = (r >> 11) as f64 / (1u64 << 53) as f64;
            if u < P { lvl += 1; } else { break; }
        }
        lvl
    }
    pub fn insert(&mut self, key: K, value: V) {
        let lvl = self.random_level();
        let new_node = Rc::new(RefCell::new(Node {
            key: Some(key.clone()),
            value: Some(value.clone()),
            forward: vec![None; lvl + 1],
        }));
        let mut update: Vec<Rc<RefCell<Node<K, V>>>> = (0..MAX_LEVEL).map(|_| self.head.clone()).collect();
        {
            let mut cur = self.head.clone();
            for i in (0..=self.level).rev() {
                loop {
                    let next = cur.borrow().forward[i].clone();
                    if let Some(n) = next {
                        if n.borrow().key.as_ref().unwrap() < &key { cur = n; continue; }
                    }
                    break;
                }
                update[i] = cur.clone();
            }
        }
        for i in 0..=lvl {
            let next = update[i].borrow().forward[i].clone();
            new_node.borrow_mut().forward[i] = next;
            update[i].borrow_mut().forward[i] = Some(new_node.clone());
        }
        if lvl > self.level { self.level = lvl; }
        self.len += 1;
    }
    pub fn get(&self, key: &K) -> Option<V> {
        let mut cur = self.head.clone();
        for i in (0..=self.level).rev() {
            loop {
                let next = cur.borrow().forward[i].clone();
                if let Some(n) = next {
                    if n.borrow().key.as_ref().unwrap() < key { cur = n; continue; }
                }
                break;
            }
        }
        let next = cur.borrow().forward[0].clone();
        if let Some(n) = next {
            if n.borrow().key.as_ref().unwrap() == key {
                return n.borrow().value.clone();
            }
        }
        None
    }
    pub fn contains(&self, key: &K) -> bool { self.get(key).is_some() }
    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn empty() {
        let sl: SkipList<i32, &str> = SkipList::new();
        assert!(sl.is_empty());
        assert_eq!(sl.len(), 0);
        assert!(!sl.contains(&1));
    }
    #[test] fn insert_get() {
        let mut sl = SkipList::new();
        sl.insert(1, "a");
        sl.insert(2, "b");
        sl.insert(3, "c");
        assert_eq!(sl.get(&1), Some("a"));
        assert_eq!(sl.get(&2), Some("b"));
        assert_eq!(sl.get(&3), Some("c"));
        assert_eq!(sl.len(), 3);
        assert!(!sl.is_empty());
    }
    #[test] fn contains() {
        let mut sl = SkipList::new();
        sl.insert(10, 100);
        assert!(sl.contains(&10));
        assert!(!sl.contains(&20));
    }
    #[test] fn replace() {
        let mut sl = SkipList::new();
        sl.insert(1, "a");
        sl.insert(1, "b");
        assert_eq!(sl.get(&1), Some("b"));
        assert_eq!(sl.len(), 2);
    }
    #[test] fn many_keys() {
        let mut sl = SkipList::new();
        for i in 0..100 { sl.insert(i, i * 10); }
        assert_eq!(sl.len(), 100);
        for i in 0..100 { assert_eq!(sl.get(&i), Some(i * 10)); }
    }
}
