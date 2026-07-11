//! Semantic placeholder allocation within one workspace scrub.

use std::collections::HashMap;

/// Allocates `[TYPE#N]` placeholders for distinct values of each type.
///
/// Indices are deterministic **within** a session for a fixed `session_seed` and
/// first-seen order. Different `session_seed` values permute type-local indices
/// so separate scrub sessions are not guaranteed to share the same numbers
/// (reduces cross-export correlation). Maps are never persisted.
#[derive(Debug, Default)]
pub struct PlaceholderAllocator {
    /// (detector_type, original_value) -> display index
    assigned: HashMap<(String, String), u32>,
    /// detector_type -> ordered unique values in first-seen order
    order: HashMap<String, Vec<String>>,
    session_seed: u64,
}

impl PlaceholderAllocator {
    pub fn new(session_seed: u64) -> Self {
        Self {
            assigned: HashMap::new(),
            order: HashMap::new(),
            session_seed,
        }
    }

    /// Returns the placeholder string for this value, allocating if needed.
    pub fn placeholder_for(&mut self, detector_type: &str, value: &str) -> String {
        let key = (detector_type.to_string(), value.to_string());
        if let Some(&idx) = self.assigned.get(&key) {
            return format!("[{detector_type}#{idx}]");
        }

        let list = self.order.entry(detector_type.to_string()).or_default();
        list.push(value.to_string());
        // Recompute indices for this type from first-seen list + seed permutation.
        self.reindex_type(detector_type);
        let idx = *self
            .assigned
            .get(&key)
            .expect("value must be indexed after reindex");
        format!("[{detector_type}#{idx}]")
    }

    fn reindex_type(&mut self, detector_type: &str) {
        let Some(values) = self.order.get(detector_type).cloned() else {
            return;
        };
        let n = values.len() as u32;
        // Permute display indices using session_seed so different sessions differ.
        // For n values, display index for first-seen position i is:
        //   1 + ((i + offset) % n)  where offset derived from seed + type name.
        let offset = self.offset_for(detector_type, n);
        for (i, value) in values.iter().enumerate() {
            let display = if n == 0 {
                1
            } else {
                1 + ((i as u32 + offset) % n)
            };
            self.assigned
                .insert((detector_type.to_string(), value.clone()), display);
        }
    }

    fn offset_for(&self, detector_type: &str, n: u32) -> u32 {
        if n <= 1 {
            return 0;
        }
        let mut h = self.session_seed;
        for b in detector_type.as_bytes() {
            h = h.wrapping_mul(31).wrapping_add(*b as u64);
        }
        (h % n as u64) as u32
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
        let mut a = PlaceholderAllocator::new(0);
        let p1 = a.placeholder_for("AWS_ACCESS_KEY", "AKIATEST1111111111");
        let p2 = a.placeholder_for("AWS_ACCESS_KEY", "AKIATEST1111111111");
        assert_eq!(p1, p2);
    }

    #[test]
    fn distinct_values_distinct_placeholders() {
        let mut a = PlaceholderAllocator::new(0);
        let p1 = a.placeholder_for("AWS_ACCESS_KEY", "AKIATEST1111111111");
        let p2 = a.placeholder_for("AWS_ACCESS_KEY", "AKIATEST2222222222");
        assert_ne!(p1, p2);
    }

    #[test]
    fn different_seeds_can_differ() {
        let mut a = PlaceholderAllocator::new(1);
        let mut b = PlaceholderAllocator::new(2);
        // Two values so offset can permute
        let _ = a.placeholder_for("T", "v1");
        let pa2 = a.placeholder_for("T", "v2");
        let _ = b.placeholder_for("T", "v1");
        let pb2 = b.placeholder_for("T", "v2");
        // With two values and different offsets, at least one pairing differs often;
        // seeds 1 and 2 produce different offsets mod 2.
        let pa1 = a.placeholder_for("T", "v1");
        let pb1 = b.placeholder_for("T", "v1");
        assert!(
            pa1 != pb1 || pa2 != pb2,
            "expected different seed to change index assignment: {pa1}/{pa2} vs {pb1}/{pb2}"
        );
    }
}
