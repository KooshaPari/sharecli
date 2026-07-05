use std::time::{Duration, Instant};

pub struct Stopwatch { start: Instant, laps: Vec<Duration> }
impl Stopwatch {
    pub fn new() -> Self { Self { start: Instant::now(), laps: Vec::new() } }
    pub fn start_at(at: Instant) -> Self { Self { start: at, laps: Vec::new() } }
    pub fn lap(&mut self) -> Duration {
        let now = Instant::now();
        let lap = now.duration_since(self.start);
        self.laps.push(lap);
        self.start = now;
        lap
    }
    pub fn elapsed(&self) -> Duration { Instant::now().duration_since(self.start) + self.laps.iter().sum::<Duration>() }
    pub fn lap_count(&self) -> usize { self.laps.len() }
    pub fn best_lap(&self) -> Option<Duration> { self.laps.iter().min().copied() }
    pub fn reset(&mut self) { self.start = Instant::now(); self.laps.clear(); }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    #[test] fn new_lap_zero() { let mut s = Stopwatch::new(); let lap = s.lap(); assert!(lap < Duration::from_millis(50)); }
    #[test] fn lap_records() { let mut s = Stopwatch::new(); s.lap(); sleep(Duration::from_millis(5)); s.lap(); assert_eq!(s.lap_count(), 2); }
    #[test] fn best_lap() { let mut s = Stopwatch::new(); sleep(Duration::from_millis(5)); s.lap(); sleep(Duration::from_millis(20)); s.lap(); assert!(s.best_lap().unwrap() < Duration::from_millis(15)); }
    #[test] fn reset() { let mut s = Stopwatch::new(); s.lap(); s.reset(); assert_eq!(s.lap_count(), 0); }
}
