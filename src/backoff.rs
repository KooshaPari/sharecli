use std::time::Duration;

#[derive(Debug,Clone,Copy)]
pub enum BackoffStrategy { Fixed, Linear, Exponential }

pub struct Backoff { strategy: BackoffStrategy, base: Duration, max: Duration }
impl Backoff {
    pub fn new(strategy: BackoffStrategy, base: Duration, max: Duration) -> Self { Self { strategy, base, max } }
    pub fn delay_for(&self, attempt: u32) -> Duration {
        let ms = match self.strategy {
            BackoffStrategy::Fixed => self.base.as_millis() as u64,
            BackoffStrategy::Linear => self.base.as_millis() as u64 * (attempt as u64 + 1),
            BackoffStrategy::Exponential => self.base.as_millis() as u64 * 2u64.saturating_pow(attempt).min(u64::MAX/2),
        };
        Duration::from_millis(ms.min(self.max.as_millis() as u64))
    }
    pub fn strategy(&self) -> BackoffStrategy { self.strategy }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn fixed_constant() { let b = Backoff::new(BackoffStrategy::Fixed, Duration::from_millis(100), Duration::from_secs(10)); assert_eq!(b.delay_for(0), b.delay_for(5)); }
    #[test] fn linear_grows() { let b = Backoff::new(BackoffStrategy::Linear, Duration::from_millis(100), Duration::from_secs(10)); assert!(b.delay_for(2) > b.delay_for(0)); }
    #[test] fn exponential_doubles() { let b = Backoff::new(BackoffStrategy::Exponential, Duration::from_millis(100), Duration::from_secs(10)); assert_eq!(b.delay_for(0).as_millis(), 100); assert_eq!(b.delay_for(1).as_millis(), 200); assert_eq!(b.delay_for(2).as_millis(), 400); }
    #[test] fn capped_at_max() { let b = Backoff::new(BackoffStrategy::Exponential, Duration::from_millis(100), Duration::from_millis(500)); assert_eq!(b.delay_for(10).as_millis(), 500); }
    #[test] fn strategy_clone() { assert_eq!(Backoff::new(BackoffStrategy::Fixed, Duration::from_millis(1), Duration::from_millis(1)).strategy(), BackoffStrategy::Fixed); }
}
