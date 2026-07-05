use std::cell::OnceCell;

pub struct Lazy<T> { cell: OnceCell<T>, init: fn() -> T }
impl<T> Lazy<T> {
    pub const fn new(init: fn() -> T) -> Self { Self { cell: OnceCell::new(), init } }
    pub fn get(&self) -> &T { self.cell.get_or_init(|| (self.init)()) }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn lazy_once() { let l = Lazy::new(|| 42); assert_eq!(*l.get(), 42); assert_eq!(*l.get(), 42); }
    #[test] fn lazy_init_fn() { let mut calls = 0; let l = Lazy::new(|| { calls += 1; 100 }); l.get(); l.get(); assert_eq!(calls, 1); }
    #[test] fn lazy_string() { let l = Lazy::new(|| String::from("hello")); assert_eq!(l.get(), "hello"); }
}
