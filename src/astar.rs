use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

pub fn shortest_path<N, FN, FH>(
    start: N,
    goal: N,
    mut neighbors: FN,
    mut heuristic: FH,
) -> Option<Vec<N>>
where
    N: Copy + Eq + Ord + std::hash::Hash,
    FN: FnMut(N) -> Vec<(N, u32)>,
    FH: FnMut(N) -> u32,
{
    let mut open: BinaryHeap<Reverse<(u32, N)>> = BinaryHeap::new();
    let mut g_score: HashMap<N, u32> = HashMap::new();
    let mut came_from: HashMap<N, N> = HashMap::new();
    open.push(Reverse((heuristic(start), start)));
    g_score.insert(start, 0);
    while let Some(Reverse((_, current))) = open.pop() {
        if current == goal {
            let mut path = vec![current];
            let mut c = current;
            while let Some(&p) = came_from.get(&c) {
                path.push(p);
                c = p;
            }
            path.reverse();
            return Some(path);
        }
        for (next, cost) in neighbors(current) {
            let tentative = g_score[&current] + cost;
            let existing = g_score.get(&next).copied().unwrap_or(u32::MAX);
            if tentative < existing {
                g_score.insert(next, tentative);
                came_from.insert(next, current);
                open.push(Reverse((tentative + heuristic(next), next)));
            }
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn straight_line() {
        let path = shortest_path(0u32, 5u32, |x| vec![(x + 1, 1)], |x| (5 - x) * 10).unwrap();
        assert_eq!(path, vec![0, 1, 2, 3, 4, 5]);
    }
    #[test]
    fn unreachable() {
        let path = shortest_path(0u32, 5u32, |_| vec![], |_| 0);
        assert!(path.is_none());
    }
    #[test]
    fn with_branch() {
        let path = shortest_path(
            0u32,
            5u32,
            |x| vec![(x + 1, 1), (x + 2, 3)],
            |x| 5u32.saturating_sub(x) * 10,
        )
        .unwrap();
        assert!(!path.is_empty());
    }
}
