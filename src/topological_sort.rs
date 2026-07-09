// Topological sort on a directed acyclic graph (DAG).
// Nodes are addressed by usize (0..n). Edges go from `from` -> `to`.
// Two algorithms are exposed:
//   - `kahn_sort`     : BFS-based Kahn's algorithm using in-degree counts.
//   - `dfs_sort`      : DFS-based order (reverse of post-order finish times).
// Both return `Some(order)` on success and `None` if a cycle is present.
// On success the returned Vec lists nodes in a valid topological order:
// for every edge `u -> v`, `u` appears before `v` in the order.
// std-only Rust.

/// Build a directed adjacency list from a slice of (from, to) edges.
pub fn build_adj(n: usize, edges: &[(usize, usize)]) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::<usize>::new(); n];
    for &(u, v) in edges {
        if u < n && v < n && u != v {
            adj[u].push(v);
        }
    }
    adj
}

/// Kahn's algorithm: process nodes in order of in-degree 0, decrementing
/// in-degrees of their neighbours.
///
/// Returns `Some(order)` if a topological ordering exists, `None` if a cycle
/// (or self-loop in the edge list) prevents one.
pub fn kahn_sort(n: usize, edges: &[(usize, usize)]) -> Option<Vec<usize>> {
    if n == 0 {
        return Some(Vec::new());
    }
    let mut indeg = vec![0usize; n];
    for &(u, v) in edges {
        if u >= n || v >= n {
            return None;
        }
        indeg[v] += 1;
    }
    let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
    for i in 0..n {
        if indeg[i] == 0 {
            queue.push_back(i);
        }
    }
    let mut order = Vec::with_capacity(n);
    while let Some(u) = queue.pop_front() {
        order.push(u);
        for &v in &build_adj(n, edges)[u] {
            // Self-loops create a cycle; skip safely.
            if v == u {
                return None;
            }
            indeg[v] -= 1;
            if indeg[v] == 0 {
                queue.push_back(v);
            }
        }
    }
    if order.len() == n {
        Some(order)
    } else {
        None
    }
}

/// DFS-based topological sort. Pushes nodes onto `order` in post-order, then
/// reverses to get the topological ordering. Avoids recursion to keep the
/// implementation robust for large DAGs (and free of stack-overflow risk).
pub fn dfs_sort(n: usize, edges: &[(usize, usize)]) -> Option<Vec<usize>> {
    if n == 0 {
        return Some(Vec::new());
    }
    // Self-loops are cycles; `build_adj` drops them, so detect here.
    for &(u, v) in edges {
        if u == v {
            return None;
        }
    }
    let adj = build_adj(n, edges);

    // 0 = unvisited, 1 = on stack, 2 = done.
    #[derive(Clone)]
    enum Color {
        White,
        Gray,
        Black,
    }
    let mut color = vec![Color::White; n];
    let mut order = Vec::with_capacity(n);

    // Iterative DFS using our own stack of (node, child-index).
    for start in 0..n {
        if matches!(color[start], Color::White) {
            let mut stack: Vec<(usize, usize)> = Vec::new();
            stack.push((start, 0));
            color[start] = Color::Gray;
            'outer: while let Some(top) = stack.last_mut() {
                let (u, ref mut idx) = *top;
                let adj_u = &adj[u];
                while *idx < adj_u.len() {
                    let v = adj_u[*idx];
                    *idx += 1;
                    match color[v] {
                        Color::White => {
                            color[v] = Color::Gray;
                            stack.push((v, 0));
                            continue 'outer;
                        }
                        Color::Gray => return None,
                        Color::Black => {}
                    }
                }
                // All neighbours processed; mark done and emit post-order.
                color[u] = Color::Black;
                order.push(u);
                stack.pop();
            }
        }
    }

    order.reverse();
    Some(order)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validate_topo(n: usize, edges: &[(usize, usize)], order: &[usize]) -> bool {
        let mut pos = vec![0usize; n];
        for (i, &u) in order.iter().enumerate() {
            if u >= n {
                return false;
            }
            pos[u] = i;
        }
        for &(u, v) in edges {
            if pos[u] >= pos[v] {
                return false;
            }
        }
        true
    }

    #[test]
    fn kahn_linear_chain() {
        // 0 -> 1 -> 2 -> 3
        let edges = [(0, 1), (1, 2), (2, 3)];
        let order = kahn_sort(4, &edges).expect("must succeed");
        assert_eq!(order, vec![0, 1, 2, 3]);
        assert!(validate_topo(4, &edges, &order));
    }

    #[test]
    fn dfs_linear_chain() {
        let edges = [(0, 1), (1, 2), (2, 3)];
        let order = dfs_sort(4, &edges).expect("must succeed");
        assert_eq!(order, vec![0, 1, 2, 3]);
    }

    #[test]
    fn diamond_graph() {
        // 0 -> 1, 0 -> 2, 1 -> 3, 2 -> 3
        let edges = [(0, 1), (0, 2), (1, 3), (2, 3)];
        let order_k = kahn_sort(4, &edges).expect("kahn ok");
        let order_d = dfs_sort(4, &edges).expect("dfs ok");
        assert!(validate_topo(4, &edges, &order_k));
        assert!(validate_topo(4, &edges, &order_d));
        // 0 must come first, 3 last.
        assert_eq!(order_k[0], 0);
        assert_eq!(order_k[3], 3);
        assert_eq!(order_d[0], 0);
        assert_eq!(order_d[3], 3);
    }

    #[allow(unused_imports)]
    use std::collections::VecDeque;

    #[test]
    fn empty_graph() {
        let order = kahn_sort(5, &[]).expect("must succeed");
        assert_eq!(order.len(), 5);
        let order_d = dfs_sort(5, &[]).expect("must succeed");
        assert_eq!(order_d.len(), 5);
    }

    #[test]
    fn cycle_detection_kahn() {
        // 0 -> 1 -> 2 -> 0
        let edges = [(0, 1), (1, 2), (2, 0)];
        assert!(kahn_sort(3, &edges).is_none());
    }

    #[test]
    fn cycle_detection_dfs() {
        // 0 -> 1 -> 2 -> 1 (cycle 1-2)
        let edges = [(0, 1), (1, 2), (2, 1)];
        assert!(dfs_sort(3, &edges).is_none());
    }

    #[test]
    fn self_loop_is_cycle() {
        let edges = [(0, 0)];
        assert!(kahn_sort(1, &edges).is_none());
        assert!(dfs_sort(1, &edges).is_none());
    }

    #[test]
    fn multiple_zero_indegree() {
        // 0 and 2 are sources, both feed 3.
        let edges = [(0, 3), (2, 3)];
        let order = kahn_sort(4, &edges).expect("must succeed");
        assert!(validate_topo(4, &edges, &order));
        // 3 must be last.
        assert_eq!(order[3], 3);
    }

    #[test]
    fn partial_order_respected() {
        // Two disconnected DAGs: {0,1} and {2,3,4}.
        let edges = [(0, 1), (2, 3), (3, 4)];
        let order = kahn_sort(5, &edges).expect("must succeed");
        assert!(validate_topo(5, &edges, &order));
        let pos0 = order.iter().position(|&x| x == 0).unwrap();
        let pos1 = order.iter().position(|&x| x == 1).unwrap();
        let pos2 = order.iter().position(|&x| x == 2).unwrap();
        let pos3 = order.iter().position(|&x| x == 3).unwrap();
        let pos4 = order.iter().position(|&x| x == 4).unwrap();
        assert!(pos0 < pos1, "0 before 1");
        assert!(pos2 < pos3, "2 before 3");
        assert!(pos3 < pos4, "3 before 4");
    }

    #[test]
    fn kahn_and_dfs_agree_on_acyclic() {
        let edges = [(0, 2), (0, 3), (1, 3), (1, 4), (2, 5), (3, 5), (4, 5)];
        let k = kahn_sort(6, &edges).unwrap();
        let d = dfs_sort(6, &edges).unwrap();
        assert!(validate_topo(6, &edges, &k));
        assert!(validate_topo(6, &edges, &d));
    }
}
