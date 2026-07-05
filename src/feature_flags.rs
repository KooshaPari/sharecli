use std::collections::HashMap;

pub struct FeatureFlags { flags: HashMap<String, bool> }
impl FeatureFlags {
    pub fn new() -> Self { Self { flags: HashMap::new() } }
    pub fn enable(&mut self, name: impl Into<String>) { self.flags.insert(name.into(), true); }
    pub fn disable(&mut self, name: impl Into<String>) { self.flags.insert(name.into(), false); }
    pub fn is_enabled(&self, name: &str) -> bool { self.flags.get(name).copied().unwrap_or(false) }
    pub fn toggle(&mut self, name: &str) -> bool {
        let cur = self.is_enabled(name);
        self.flags.insert(name.to_string(), !cur);
        !cur
    }
    pub fn enabled_count(&self) -> usize { self.flags.values().filter(|v| **v).count() }
    pub fn total(&self) -> usize { self.flags.len() }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn initially_disabled() { assert!(!FeatureFlags::new().is_enabled("x")); }
    #[test] fn enable_works() { let mut f = FeatureFlags::new(); f.enable("x"); assert!(f.is_enabled("x")); }
    #[test] fn disable_works() { let mut f = FeatureFlags::new(); f.enable("x"); f.disable("x"); assert!(!f.is_enabled("x")); }
    #[test] fn toggle() { let mut f = FeatureFlags::new(); assert!(f.toggle("x")); assert!(!f.toggle("x")); }
    #[test] fn counts() { let mut f = FeatureFlags::new(); f.enable("a"); f.enable("b"); assert_eq!(f.enabled_count(), 2); assert_eq!(f.total(), 2); }
}
