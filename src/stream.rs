pub struct Stream<T> {
    chunks: Vec<Vec<T>>,
    pos: usize,
    off: usize,
}
impl<T: Clone> Stream<T> {
    pub fn new() -> Self {
        Self { chunks: vec![Vec::new()], pos: 0, off: 0 }
    }
    pub fn write(&mut self, item: T) {
        self.chunks.last_mut().unwrap().push(item);
    }
    pub fn next(&mut self) -> Option<T> {
        loop {
            if self.pos >= self.chunks.len() {
                return None;
            }
            if self.off < self.chunks[self.pos].len() {
                let v = self.chunks[self.pos][self.off].clone();
                self.off += 1;
                return Some(v);
            }
            self.pos += 1;
            self.off = 0;
        }
    }
    pub fn chunk_size(&mut self, n: usize) {
        if n == 0 {
            return;
        }
        let cur = std::mem::take(&mut self.chunks);
        let mut new_chunks = Vec::new();
        let mut cur_chunk = Vec::new();
        for chunk in cur.into_iter() {
            for item in chunk {
                cur_chunk.push(item);
                if cur_chunk.len() == n {
                    new_chunks.push(std::mem::take(&mut cur_chunk));
                }
            }
        }
        if !cur_chunk.is_empty() {
            new_chunks.push(cur_chunk);
        }
        self.chunks = new_chunks;
        self.pos = 0;
        self.off = 0;
    }
    pub fn collected(&mut self) -> Vec<T> {
        let mut out = Vec::new();
        while let Some(v) = self.next() {
            out.push(v);
        }
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let mut s: Stream<i32> = Stream::new();
        s.write(1);
        s.write(2);
        assert_eq!(s.next(), Some(1));
        assert_eq!(s.next(), Some(2));
        assert_eq!(s.next(), None);
    }
    #[test]
    fn chunks() {
        let mut s: Stream<i32> = Stream::new();
        for i in 0..5 {
            s.write(i);
        }
        s.chunk_size(2);
        assert_eq!(s.collected(), vec![0, 1, 2, 3, 4]);
    }
    #[test]
    fn empty() {
        let mut s: Stream<i32> = Stream::new();
        assert_eq!(s.collected().len(), 0);
    }
}
