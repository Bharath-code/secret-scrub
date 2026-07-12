//! Semantic placeholder allocation within one workspace scrub.

use std::collections::HashMap;

/// Allocates `[TYPE#N]` placeholders for distinct values of each type.
///
/// Each type gets a sequential counter: the first-seen value gets the next
/// index and indices never change afterwards, so the same value maps to the
/// same placeholder across every file in a workspace scrub. Maps are never
/// persisted.
#[derive(Debug, Default)]
pub struct PlaceholderAllocator {
    /// (detector_type, original_value) -> display index
    assigned: HashMap<(String, String), u32>,
    /// detector_type -> next index to hand out
    next: HashMap<String, u32>,
}

impl PlaceholderAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the placeholder string for this value, allocating if needed.
    pub fn placeholder_for(&mut self, detector_type: &str, value: &str) -> String {
        let key = (detector_type.to_string(), value.to_string());
        if let Some(&idx) = self.assigned.get(&key) {
            return format!("[{detector_type}#{idx}]");
        }
        let counter = self.next.entry(detector_type.to_string()).or_insert(1);
        let idx = *counter;
        *counter += 1;
        self.assigned.insert(key, idx);
        format!("[{detector_type}#{idx}]")
    }

    /// Occurrence counts per (type, value) are tracked externally; this only maps values.
    pub fn assigned_placeholder(&self, detector_type: &str, value: &str) -> Option<String> {
        self.assigned
            .get(&(detector_type.to_string(), value.to_string()))
            .map(|idx| format!("[{detector_type}#{idx}]"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_value_same_placeholder() {
        let mut a = PlaceholderAllocator::new();
        let p1 = a.placeholder_for("AWS_ACCESS_KEY", "AKIATEST1111111111");
        let p2 = a.placeholder_for("AWS_ACCESS_KEY", "AKIATEST1111111111");
        assert_eq!(p1, p2);
    }

    #[test]
    fn distinct_values_distinct_placeholders() {
        let mut a = PlaceholderAllocator::new();
        let p1 = a.placeholder_for("AWS_ACCESS_KEY", "AKIATEST1111111111");
        let p2 = a.placeholder_for("AWS_ACCESS_KEY", "AKIATEST2222222222");
        assert_ne!(p1, p2);
    }

    #[test]
    fn indices_are_first_seen_sequential_and_stable() {
        let mut a = PlaceholderAllocator::new();
        let p1 = a.placeholder_for("T", "v1");
        let p2 = a.placeholder_for("T", "v2");
        let p3 = a.placeholder_for("T", "v3");
        assert_eq!(p1, "[T#1]");
        assert_eq!(p2, "[T#2]");
        assert_eq!(p3, "[T#3]");
        // Earlier allocations never move when new values arrive.
        assert_eq!(a.placeholder_for("T", "v1"), "[T#1]");
        assert_eq!(a.assigned_placeholder("T", "v2").unwrap(), "[T#2]");
    }

    #[test]
    fn counters_are_per_type() {
        let mut a = PlaceholderAllocator::new();
        assert_eq!(a.placeholder_for("A", "x"), "[A#1]");
        assert_eq!(a.placeholder_for("B", "x"), "[B#1]");
    }
}
