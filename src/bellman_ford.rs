// Bellman-Ford single-source shortest path with negative edge detection.
// Supports reaching any reachable node from `src`. Returns `Ok(distances)` if
// no negative cycle reachable from `src` is detectable; otherwise returns the
// list of nodes participating in (or reachable from) the cycle.
// std-only Rust.

use std::collections::VecDeque;

/// Edge representation: from `u` to `v` with cost `w`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    pub u: usize,
    pub v: usize,
    pub w: i64,
}

impl Edge {
    pub fn new(u: usize, v: usize, w: i64) -> Self {
        Edge { u, v, w }
    }
}

/// Result of Bellman-Ford.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BFResult {
    /// Shortest distance to each node. `i64::MAX` = unreachable sentinel.
    Distances(Vec<i64>),
    /// A negative cycle exists reachable from `src`. The cycle nodes are
    /// listed in arbitrary order.
    NegativeCycle(Vec<usize>),
}

/// Compute shortest paths from `src`. `n` is the number of nodes (0..n).
pub fn bellman_ford(n: usize, edges: &[Edge], src: usize) -> BFResult {
    let mut dist = vec![i64::MAX; n];
    dist[src] = 0;

    // Relax edges n-1 times.
    for _ in 1..n {
        let mut changed = false;
        for e in edges {
            if dist[e.u] != i64::MAX && dist[e.u] + e.w < dist[e.v] {
                dist[e.v] = dist[e.u] + e.w;
                changed = true;
            }
        }
        if !changed {
            return BFResult::Distances(dist);
        }
    }

    // n-th pass: find nodes that can still be relaxed (part of / reachable from
    // a negative cycle).
    let mut on_neg = vec![false; n];
    let mut queue = VecDeque::new();
    for e in edges {
        if dist[e.u] != i64::MAX && dist[e.u] + e.w < dist[e.v] {
            on_neg[e.v] = true;
            queue.push_back(e.v);
        }
    }
    while let Some(u) = queue.pop_front() {
        for e in edges {
            if e.u == u && !on_neg[e.v] {
                on_neg[e.v] = true;
                queue.push_back(e.v);
            }
        }
    }
    if on_neg.iter().any(|&b| b) {
        let cycle: Vec<usize> = (0..n).filter(|&i| on_neg[i]).collect();
        BFResult::NegativeCycle(cycle)
    } else {
        BFResult::Distances(dist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_shortest() {
        // 0 -> 1 (w=4), 0 -> 2 (w=5), 1 -> 2 (w=-3), 2 -> 3 (w=2)
        let edges = [
            Edge::new(0, 1, 4),
            Edge::new(0, 2, 5),
            Edge::new(1, 2, -3),
            Edge::new(2, 3, 2),
        ];
        let result = bellman_ford(4, &edges, 0);
        let dists = match result {
            BFResult::Distances(d) => d,
            other => panic!("expected distances, got {:?}", other),
        };
        assert_eq!(dists[0], 0);
        assert_eq!(dists[1], 4);
        assert_eq!(dists[2], 1);
        assert_eq!(dists[3], 3);
    }

    #[test]
    fn unreachable() {
        let edges = [Edge::new(0, 1, 1)];
        let dists = match bellman_ford(4, &edges, 0) {
            BFResult::Distances(d) => d,
            _ => panic!(),
        };
        assert_eq!(dists[1], 1);
        assert_eq!(dists[2], i64::MAX);
        assert_eq!(dists[3], i64::MAX);
    }

    #[test]
    fn negative_cycle_detection() {
        // 0 -> 1 (1), 1 -> 2 (-1), 2 -> 1 (-1)  -> cycle 1-2 weight -2
        let edges = [Edge::new(0, 1, 1), Edge::new(1, 2, -1), Edge::new(2, 1, -1)];
        match bellman_ford(3, &edges, 0) {
            BFResult::NegativeCycle(nodes) => {
                assert!(nodes.contains(&1) || nodes.contains(&2));
            }
            other => panic!("expected cycle, got {:?}", other),
        }
    }

    #[test]
    fn no_negative_cycle_unreachable() {
        // 0 -> 1, 2 -> 3 -> 2 negative cycle isolated from 0
        let edges = [Edge::new(0, 1, 1), Edge::new(2, 3, 1), Edge::new(3, 2, -3)];
        let dists = match bellman_ford(4, &edges, 0) {
            BFResult::Distances(d) => d,
            other => panic!("expected distances (cycle not reachable from src), got {:?}", other),
        };
        assert_eq!(dists[1], 1);
    }
}
