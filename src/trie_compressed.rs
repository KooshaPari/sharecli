pub struct CompressedTrie {
    root: Node,
}
struct Node {
    edges: Vec<(String, Box<Node>)>,
    word: bool,
}
impl Node {
    fn new() -> Self { Self { edges: Vec::new(), word: false } }
}
impl CompressedTrie {
    pub fn new() -> Self { Self { root: Node::new() } }
    pub fn insert(&mut self, word: &str) {
        if word.is_empty() { self.root.word = true; return; }
        Self::insert_into(&mut self.root, word);
    }
    fn insert_into(node: &mut Node, word: &str) {
        if word.is_empty() { node.word = true; return; }
        // find edge with longest common prefix
        let mut best: Option<usize> = None;
        let mut best_common = 0usize;
        for (i, (k, _)) in node.edges.iter().enumerate() {
            let c = k.chars().zip(word.chars()).take_while(|(a, b)| a == b).count();
            if c > best_common { best = Some(i); best_common = c; }
        }
        match best {
            None => {
                node.edges.push((word.to_string(), Box::new(Node { edges: Vec::new(), word: true })));
            }
            Some(i) => {
                let k_len = node.edges[i].0.chars().count();
                if best_common == k_len {
                    // full match on edge key, descend
                    let rest: String = word.chars().skip(best_common).collect();
                    Self::insert_into(&mut node.edges[i].1, &rest);
                } else {
                    // split edge: best_common < k_len and best_common < word.len
                    let k_tail: String = node.edges[i].0.chars().skip(best_common).collect();
                    let r_tail: String = word.chars().skip(best_common).collect();
                    // extract old child
                    let old_key = std::mem::replace(&mut node.edges[i].0, String::new());
                    let mut old_child = std::mem::replace(&mut node.edges[i].1, Box::new(Node::new()));
                    let old_word = old_child.word;
                    let old_edges = std::mem::take(&mut old_child.edges);
                    // build split node
                    let mut split = Node::new();
                    split.word = old_word;
                    let mut k_child = Node::new();
                    k_child.word = old_word;
                    k_child.edges = old_edges;
                    split.edges.push((k_tail, Box::new(k_child)));
                    let mut r_child = Node::new();
                    r_child.word = true;
                    split.edges.push((r_tail, Box::new(r_child)));
                    node.edges[i] = (old_key.chars().take(best_common).collect(), Box::new(split));
                }
            }
        }
    }
    pub fn contains(&self, word: &str) -> bool {
        Self::find(&self.root, word)
    }
    fn find(node: &Node, word: &str) -> bool {
        if word.is_empty() { return node.word; }
        for (k, child) in &node.edges {
            let k_len = k.chars().count();
            let w_len = word.chars().count();
            let common: usize = k.chars().zip(word.chars()).take_while(|(a, b)| a == b).count();
            if common == 0 { continue; }
            if common < k_len {
                // edge key diverges mid-way from query; query ends at a non-word prefix point.
                return false;
            }
            // common == k_len
            if common == w_len { return child.word; }
            let rest: String = word.chars().skip(common).collect();
            return Self::find(child.as_ref(), &rest);
        }
        false
    }
    pub fn len(&self) -> usize { Self::count(&self.root) }
    fn count(node: &Node) -> usize {
        let mut c = if node.word { 1 } else { 0 };
        for (_, ch) in &node.edges { c += Self::count(ch); }
        c
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn empty() {
        let t = CompressedTrie::new();
        assert_eq!(t.len(), 0);
        assert!(!t.contains("anything"));
    }
    #[test] fn insert_contains() {
        let mut t = CompressedTrie::new();
        t.insert("hello");
        t.insert("help");
        assert!(t.contains("hello"));
        assert!(t.contains("help"));
        assert!(!t.contains("hell"));
    }
    #[test] fn prefix_not_word() {
        let mut t = CompressedTrie::new();
        t.insert("test");
        assert!(t.contains("test"));
        assert!(!t.contains("te"));
    }
    #[test] fn shared_prefix() {
        let mut t = CompressedTrie::new();
        t.insert("foo");
        t.insert("foobar");
        assert!(t.contains("foo"));
        assert!(t.contains("foobar"));
        assert!(!t.contains("foob"));
    }
    #[test] fn count() {
        let mut t = CompressedTrie::new();
        t.insert("a");
        t.insert("ab");
        t.insert("abc");
        assert_eq!(t.len(), 3);
    }
    #[test] fn overwrite() {
        let mut t = CompressedTrie::new();
        t.insert("hello");
        t.insert("hello");
        assert_eq!(t.len(), 1);
        assert!(t.contains("hello"));
    }
}
