use std::collections::BinaryHeap;
use std::cmp::Reverse;

pub struct PriorityQueue<T: Ord> { heap: BinaryHeap<Reverse<T>> }
impl<T: Ord> PriorityQueue<T> {
    pub fn new() -> Self { Self { heap: BinaryHeap::new() } }
    pub fn push(&mut self, v: T) { self.heap.push(Reverse(v)); }
    pub fn pop(&mut self) -> Option<T> { self.heap.pop().map(|r| r.0) }
    pub fn peek(&self) -> Option<&T> { self.heap.peek().map(|r| &r.0) }
    pub fn len(&self) -> usize { self.heap.len() }
    pub fn is_empty(&self) -> bool { self.heap.is_empty() }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn min_priority() { let mut pq = PriorityQueue::new(); pq.push(5); pq.push(1); pq.push(3); assert_eq!(pq.pop(), Some(1)); assert_eq!(pq.pop(), Some(3)); }
    #[test] fn peek() { let mut pq = PriorityQueue::new(); pq.push(42); assert_eq!(pq.peek(), Some(&42)); assert_eq!(pq.len(), 1); }
    #[test] fn empty() { let mut pq: PriorityQueue<i32> = PriorityQueue::new(); assert_eq!(pq.pop(), None); }
    #[test] fn strings() { let mut pq = PriorityQueue::new(); pq.push("banana".to_string()); pq.push("apple".to_string()); assert_eq!(pq.pop(), Some("apple".to_string())); }
}
