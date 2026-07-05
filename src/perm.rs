pub fn next_permutation(arr: &mut [u32]) -> bool {
    if arr.len() < 2 { return false; }
    let mut i = arr.len() - 2;
    while i > 0 && arr[i] >= arr[i + 1] { i -= 1; }
    if i == 0 && arr[0] >= arr[1] { return false; }
    let mut j = arr.len() - 1;
    while arr[j] <= arr[i] { j -= 1; }
    arr.swap(i, j);
    arr[i + 1..].reverse();
    true
}
pub fn permutations<T: Clone>(items: &[T]) -> Vec<Vec<T>> {
    let mut result = Vec::new();
    if items.is_empty() { result.push(Vec::new()); return result; }
    let mut pool: Vec<T> = items.to_vec();
    fn permute<T: Clone>(pool: &mut Vec<T>, k: usize, result: &mut Vec<Vec<T>>) {
        if k == pool.len() { result.push(pool.clone()); return; }
        for i in k..pool.len() {
            pool.swap(k, i);
            permute(pool, k + 1, result);
            pool.swap(k, i);
        }
    }
    permute(&mut pool, 0, &mut result);
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn next_perm_basic() { let mut a = vec![1, 2, 3]; assert!(next_permutation(&mut a)); assert_eq!(a, vec![1, 3, 2]); }
    #[test] fn next_perm_last() { let mut a = vec![3, 2, 1]; assert!(!next_permutation(&mut a)); }
    #[test] fn perms_3() { let p = permutations(&[1, 2, 3]); assert_eq!(p.len(), 6); }
    #[test] fn perms_empty() { let p = permutations::<i32>(&[]); assert_eq!(p, vec![Vec::<i32>::new()]); }
    #[test] fn perms_single() { let p = permutations(&[42]); assert_eq!(p, vec![vec![42]]); }
}
