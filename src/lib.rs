//! `SpanMap` is a data structure that efficiently maps spans (ranges) to sets of values.
//!
//! `SpanMap` allows you to associate sets of values with potentially overlapping spans,
//! where a span represents a continuous range with well-defined boundaries.
//! It's particularly useful for scenarios where you need to:
//!
//! * Track multiple values across ranges
//! * Efficiently query values at any given point
//! * Manage overlapping ranges with set operations
//!
//! # Example
//! ```
//! # use span_map::SpanMap;
//!
//! let mut map = SpanMap::new();
//! map.insert(0..10, "value1");
//! map.insert(5..15, "value2");
//!
//! // Point 7 has both values
//! let values: Vec<_> = map.get(&7).copied().collect();
//! assert_eq!(values, vec!["value1", "value2"]);
//! ```

#[doc(hidden)]
pub mod bounds;
#[doc(hidden)]
pub mod span;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ops::RangeBounds;

use bounds::LeftBound;
use span::Span;

/// A map that associates spans (ranges) with sets of values.
///
/// `SpanMap` maintains a mapping between spans and sets of values, where:
/// * Each span represents a continuous range with well-defined boundaries
/// * Multiple values can be associated with the same span
/// * Spans can overlap, resulting in points that contain multiple values
/// * Queries at any point return all values associated with spans containing that point
///
/// The implementation uses a B-tree based data structure for efficient operations.
///
/// # Type Parameters
///
/// * `K`: The type of the keys defining span boundaries. Must implement `Clone` and `Ord`.
/// * `V`: The type of values stored in the sets. Must implement `Clone` and `Ord`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanMap<K, V>
where
    K: Clone + Ord,
    V: Clone + Ord,
{
    m: BTreeMap<LeftBound<K>, BTreeSet<V>>,
}

impl<K, V> Default for SpanMap<K, V>
where
    K: Clone + Ord,
    V: Clone + Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> SpanMap<K, V>
where
    K: Clone + Ord,
    V: Clone + Ord,
{
    /// Creates a new, empty `SpanMap`.
    ///
    /// The new map is initialized with an unbounded span containing an empty set.
    pub fn new() -> Self {
        let mut m = BTreeMap::new();
        m.insert(LeftBound::Unbounded, BTreeSet::new());
        Self { m }
    }
}

impl<K, V> SpanMap<K, V>
where
    K: Clone + Ord,
    V: Clone + Ord,
{
    /// Returns an iterator over all values associated with spans containing the given key.
    pub fn get(&self, key: &K) -> impl Iterator<Item = &V> {
        // Safe unwrap(): Unbounded is always present
        let last_less_equal = self
            .m
            .range(..=LeftBound::Included(key.clone()))
            .next_back()
            .unwrap();

        let (_bound, set) = last_less_equal;

        set.iter()
    }

    /// Inserts a value into all sets associated with spans overlapping the given range.
    ///
    /// Adjacent ranges with the same value are merged into a single range.
    pub fn insert<R>(&mut self, range: R, value: V)
    where
        R: RangeBounds<K>,
    {
        self.insert_span(Span::from_range(range), value);
    }

    /// Removes a value from all sets associated with spans overlapping the given range.
    ///
    /// Adjacent ranges with the same value are merged into a single range.
    pub fn remove<R>(&mut self, range: R, value: V)
    where
        R: RangeBounds<K>,
    {
        self.remove_span(Span::from_range(range), value);
    }

    #[doc(hidden)]
    pub fn insert_span(&mut self, range: Span<K>, value: V) {
        self.update_set_in_span(range, |set| {
            set.insert(value.clone());
        });
    }

    #[doc(hidden)]
    pub fn remove_span(&mut self, range: Span<K>, value: V) {
        self.update_set_in_span(range, |set| {
            set.remove(&value);
        });
    }

    fn update_set_in_span(&mut self, span: Span<K>, f: impl Fn(&mut BTreeSet<V>)) {
        let start = span.left.clone();
        self.ensure_boundary(start.clone());

        let end = span.right.adjacent_left();
        if let Some(end) = end.clone() {
            self.ensure_boundary(end);
        }

        // At this point, `range.left` and `range.right` are ensured to be in the map

        for (b, set) in self.m.range_mut(span.left..) {
            if span.right < *b {
                break;
            }
            f(set);
        }

        self.merge_adjacent_left(start);
        if let Some(end) = end {
            self.merge_adjacent_left(end);
        }
    }

    /// Splits a range at the specified boundary point and ensures the boundary exists in the map.
    fn ensure_boundary(&mut self, bound: LeftBound<K>) {
        let last_less_equal = self.m.range(..=bound.clone()).next_back();
        if let Some((b, set)) = last_less_equal {
            if *b == bound {
                // no need to split
            } else {
                self.m.insert(bound, set.clone());
            }
        } else {
            // No bound <= bound, insert
            self.m.insert(bound, BTreeSet::new());
        }
    }

    /// Attempts to merge adjacent ranges by removing redundant boundaries.
    ///
    /// If the range to the left and the given one have identical value sets,
    /// the boundary between them is removed to create a single continuous range.
    fn merge_adjacent_left(&mut self, bound: LeftBound<K>) {
        let mut it = self.m.range(..=bound.clone()).rev();

        let Some((right_bound, right_set)) = it.next() else {
            return;
        };

        let Some((_left_bound, left_set)) = it.next() else {
            return;
        };

        if left_set == right_set {
            let right_bound = right_bound.clone();
            self.m.remove(&right_bound);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bounds::RightBound;

    // ===================== get

    #[test]
    fn test_get_empty_map() {
        let map = SpanMap::<i32, i32>::new();

        // Empty map should return empty iterator for any key
        assert_eq!(map.get(&5).count(), 0);
    }

    #[test]
    fn test_get_single_range() {
        let mut map = SpanMap::<i32, i32>::new();

        // [1, 5] -> {10}
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            10,
        );

        // Before range
        assert_eq!(map.get(&0).count(), 0);

        // Start of range
        let values: Vec<_> = map.get(&1).collect();
        assert_eq!(values, vec![&10]);

        // Middle of range
        let values: Vec<_> = map.get(&3).collect();
        assert_eq!(values, vec![&10]);

        // End of range
        let values: Vec<_> = map.get(&5).collect();
        assert_eq!(values, vec![&10]);

        // After range
        assert_eq!(map.get(&6).count(), 0);
    }

    #[test]
    fn test_get_overlapping_ranges() {
        let mut map = SpanMap::<i32, i32>::new();

        // [1,   5] -> {10}
        //    [3,   7] -> {20}
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            10,
        );
        map.insert_span(
            Span::new(LeftBound::Included(3), RightBound::Included(7)),
            20,
        );

        // Before first range
        assert_eq!(map.get(&0).count(), 0);

        // First range only
        let values: Vec<_> = map.get(&2).collect();
        assert_eq!(values, vec![&10]);

        // Overlapping section
        let mut values: Vec<_> = map.get(&4).collect();
        values.sort(); // Order not guaranteed for BTreeSet iterator
        assert_eq!(values, vec![&10, &20]);

        let mut values: Vec<_> = map.get(&5).collect();
        values.sort(); // Order not guaranteed for BTreeSet iterator
        assert_eq!(values, vec![&10, &20]);

        // Second range only
        let values: Vec<_> = map.get(&6).collect();
        assert_eq!(values, vec![&20]);

        // After all ranges
        assert_eq!(map.get(&8).count(), 0);
    }

    #[test]
    fn test_get_multiple_values() {
        let mut map = SpanMap::<i32, i32>::new();

        // [1, 5] -> {10, 20, 30}
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            10,
        );
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            20,
        );
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            30,
        );

        let mut values: Vec<_> = map.get(&3).collect();
        values.sort();
        assert_eq!(values, vec![&10, &20, &30]);
    }

    #[test]
    fn test_get_with_excluded_bounds() {
        let mut map = SpanMap::<i32, i32>::new();

        // (1, 5) -> {10}
        map.insert_span(
            Span::new(LeftBound::Excluded(1), RightBound::Excluded(5)),
            10,
        );

        // At excluded bounds
        assert_eq!(map.get(&1).count(), 0);
        assert_eq!(map.get(&5).count(), 0);

        // Inside range
        let values: Vec<_> = map.get(&3).collect();
        assert_eq!(values, vec![&10]);
    }

    #[test]
    fn test_get_with_unbounded() {
        let mut map = SpanMap::<i32, i32>::new();

        // (-∞, 5] -> {10}
        map.insert_span(Span::new(LeftBound::Unbounded, RightBound::Included(5)), 10);

        // Values in unbounded region
        let values: Vec<_> = map.get(&i32::MIN).collect();
        assert_eq!(values, vec![&10]);

        let values: Vec<_> = map.get(&0).collect();
        assert_eq!(values, vec![&10]);

        // After range
        assert_eq!(map.get(&6).count(), 0);
    }

    #[test]
    fn test_get_point_range() {
        let mut map = SpanMap::<i32, i32>::new();

        // [5, 5] -> {10}
        map.insert_span(
            Span::new(LeftBound::Included(5), RightBound::Included(5)),
            10,
        );

        // Before point
        assert_eq!(map.get(&4).count(), 0);

        // At point
        let values: Vec<_> = map.get(&5).collect();
        assert_eq!(values, vec![&10]);

        // After point
        assert_eq!(map.get(&6).count(), 0);
    }

    // ===================== insert

    #[test]
    fn test_insert_std_range() {
        let mut map = SpanMap::<i32, &str>::new();

        // Test with standard ranges
        map.insert(1..=5, "a");
        map.insert(3..7, "b");

        assert_eq!(map.get(&0).count(), 0);
        assert_eq!(map.get(&1).copied().collect::<Vec<_>>(), vec!["a"]);
        assert_eq!(map.get(&3).copied().collect::<Vec<_>>(), vec!["a", "b"]);
        assert_eq!(map.get(&6).copied().collect::<Vec<_>>(), vec!["b"]);
        assert_eq!(map.get(&7).count(), 0);
    }

    #[test]
    fn test_insert_empty_map() {
        let mut map = SpanMap::<i32, i32>::new();

        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            10,
        );

        // Check boundaries exist
        assert_eq!(map.m.len(), 3);
        assert!(map.m.contains_key(&LeftBound::Included(1)));
        assert!(map.m.contains_key(&LeftBound::Excluded(5))); // adjacent_left of Included(5)

        // Check value is present in range
        assert_eq!(
            map.m.get(&LeftBound::Included(1)).unwrap(),
            &BTreeSet::from([10])
        );
    }

    #[test]
    fn test_insert_overlapping_ranges() {
        let mut map = SpanMap::<i32, i32>::new();

        // [1,   5]
        //    [3,   7]

        // Insert first range
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            10,
        );

        // Insert overlapping range
        map.insert_span(
            Span::new(LeftBound::Included(3), RightBound::Included(7)),
            20,
        );

        // Check values in different segments
        assert_eq!(
            map.m.get(&LeftBound::Included(1)).unwrap(),
            &BTreeSet::from([10])
        );
        assert_eq!(
            map.m.get(&LeftBound::Included(3)).unwrap(),
            &BTreeSet::from([10, 20])
        );
        assert_eq!(
            map.m.get(&LeftBound::Excluded(5)).unwrap(),
            &BTreeSet::from([20])
        );
        assert_eq!(
            map.m.get(&LeftBound::Excluded(7)).unwrap(),
            &BTreeSet::from([])
        );
    }

    #[test]
    fn test_insert_adjacent_ranges() {
        let mut map = SpanMap::<i32, i32>::new();

        // Insert first range
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            10,
        );

        // Insert adjacent range with same value
        map.insert_span(
            Span::new(LeftBound::Excluded(5), RightBound::Included(10)),
            10,
        );

        // Should merge into single range due to adjacent ranges with same value
        assert_eq!(map.m.len(), 3); // Start and end boundaries
        assert_eq!(
            map.m.get(&LeftBound::Included(1)).unwrap(),
            &BTreeSet::from([10])
        );
    }

    #[test]
    fn test_insert_with_excluded_bounds() {
        let mut map = SpanMap::<i32, i32>::new();

        map.insert_span(
            Span::new(LeftBound::Excluded(1), RightBound::Excluded(5)),
            10,
        );

        assert!(map.m.contains_key(&LeftBound::Excluded(1)));
        assert!(map.m.contains_key(&LeftBound::Included(5)));
    }

    #[test]
    fn test_insert_with_unbounded() {
        let mut map = SpanMap::<i32, i32>::new();

        // Test unbounded left
        map.insert_span(Span::new(LeftBound::Unbounded, RightBound::Included(5)), 10);

        assert!(map.m.contains_key(&LeftBound::Unbounded));
        assert!(map.m.contains_key(&LeftBound::Excluded(5)));

        // Test unbounded right
        map.insert_span(
            Span::new(LeftBound::Included(10), RightBound::Unbounded),
            20,
        );

        assert!(map.m.contains_key(&LeftBound::Included(10)));
    }

    #[test]
    fn test_insert_multiple_values() {
        let mut map = SpanMap::<i32, i32>::new();

        // Insert overlapping ranges with different values
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(10)),
            10,
        );
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(10)),
            20,
        );
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(10)),
            30,
        );

        // Check all values present
        assert_eq!(
            map.m.get(&LeftBound::Included(1)).unwrap(),
            &BTreeSet::from([10, 20, 30])
        );
    }

    #[test]
    fn test_insert_point_range() {
        let mut map = SpanMap::<i32, i32>::new();

        // Insert range with same start and end point
        map.insert_span(
            Span::new(LeftBound::Included(5), RightBound::Included(5)),
            10,
        );

        assert!(map.m.contains_key(&LeftBound::Included(5)));
        assert!(map.m.contains_key(&LeftBound::Excluded(5)));
        assert_eq!(
            map.m.get(&LeftBound::Included(5)).unwrap(),
            &BTreeSet::from([10])
        );
    }

    #[test]
    fn test_insert_with_string_keys() {
        let mut map = SpanMap::<String, i32>::new();

        map.insert_span(
            Span::new(
                LeftBound::Included("a".to_string()),
                RightBound::Included("c".to_string()),
            ),
            10,
        );

        assert!(map.m.contains_key(&LeftBound::Included("a".to_string())));
        assert!(map.m.contains_key(&LeftBound::Excluded("c".to_string()))); // adjacent to "c"
        assert_eq!(
            map.m.get(&LeftBound::Included("a".to_string())).unwrap(),
            &BTreeSet::from([10])
        );
    }

    // ===================== remove

    #[test]
    fn test_remove_std_range() {
        let mut map = SpanMap::<i32, &str>::new();

        // Setup
        map.insert(1..=10, "a");
        map.insert(1..=10, "b");

        // Remove one value from a range
        map.remove(3..=7, "a");

        assert_eq!(map.get(&2).copied().collect::<Vec<_>>(), vec!["a", "b"]);
        assert_eq!(map.get(&5).copied().collect::<Vec<_>>(), vec!["b"]);
        assert_eq!(map.get(&8).copied().collect::<Vec<_>>(), vec!["a", "b"]);
    }

    #[test]
    fn test_remove_empty_map() {
        let mut map = SpanMap::<i32, i32>::new();

        // Removing from empty map should not panic
        map.remove_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            10,
        );
        assert_eq!(map.m.len(), 1); // Only boundary markers
        assert_eq!(map.get(&1).collect::<Vec<_>>(), Vec::<&i32>::new());
    }

    #[test]
    fn test_remove_single_value() {
        let mut map = SpanMap::<i32, i32>::new();

        // [1, 5] -> {10}
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            10,
        );

        map.remove_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            10,
        );

        assert_eq!(map.m.get(&LeftBound::Included(1)), None);
        assert_eq!(map.m.get(&LeftBound::Excluded(5)), None);
    }

    #[test]
    fn test_remove_partial_range() {
        let mut map = SpanMap::<i32, i32>::new();

        // [1,     5] -> {10}
        //   [2, 4]   Remove 10 here
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            10,
        );

        map.remove_span(
            Span::new(LeftBound::Included(2), RightBound::Included(4)),
            10,
        );

        // Check values in different segments
        assert_eq!(
            map.m.get(&LeftBound::Included(1)).unwrap(),
            &BTreeSet::from([10])
        );
        assert_eq!(
            map.m.get(&LeftBound::Included(2)).unwrap(),
            &BTreeSet::new()
        );
        assert_eq!(
            map.m.get(&LeftBound::Excluded(4)).unwrap(),
            &BTreeSet::from([10])
        );
    }

    #[test]
    fn test_remove_from_multiple_values() {
        let mut map = SpanMap::<i32, i32>::new();

        // [1, 5] -> {10, 20, 30}
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            10,
        );
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            20,
        );
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            30,
        );

        // Remove one value
        map.remove_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            20,
        );

        assert_eq!(
            map.m.get(&LeftBound::Included(1)).unwrap(),
            &BTreeSet::from([10, 30])
        );
    }

    #[test]
    fn test_remove_overlapping_ranges() {
        let mut map = SpanMap::<i32, i32>::new();

        // [1,   5] -> {10}
        //    [3,   7] -> {20}
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            10,
        );
        map.insert_span(
            Span::new(LeftBound::Included(3), RightBound::Included(7)),
            20,
        );

        // Remove value from first range
        map.remove_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            10,
        );

        assert_eq!(map.m.get(&LeftBound::Included(1)), None);
        assert_eq!(
            map.m.get(&LeftBound::Included(3)).unwrap(),
            &BTreeSet::from([20])
        );
    }

    #[test]
    fn test_remove_with_excluded_bounds() {
        let mut map = SpanMap::<i32, i32>::new();

        // (1, 5) -> {10}
        map.insert_span(
            Span::new(LeftBound::Excluded(1), RightBound::Excluded(5)),
            10,
        );

        map.remove_span(
            Span::new(LeftBound::Excluded(1), RightBound::Excluded(5)),
            10,
        );

        assert_eq!(map.m.get(&LeftBound::Excluded(1)), None);
        assert_eq!(map.m.get(&LeftBound::Included(5)), None);
    }

    #[test]
    fn test_remove_with_unbounded() {
        let mut map = SpanMap::<i32, i32>::new();

        // (-∞, 5] -> {10}
        map.insert_span(Span::new(LeftBound::Unbounded, RightBound::Included(5)), 10);

        map.remove_span(Span::new(LeftBound::Unbounded, RightBound::Included(5)), 10);

        assert_eq!(map.m.get(&LeftBound::Unbounded).unwrap(), &BTreeSet::new());
        assert_eq!(map.m.get(&LeftBound::Excluded(5)), None);
    }

    #[test]
    fn test_remove_nonexistent_value() {
        let mut map = SpanMap::<i32, i32>::new();

        // [1, 5] -> {10}
        map.insert_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            10,
        );

        // Try to remove value that doesn't exist
        map.remove_span(
            Span::new(LeftBound::Included(1), RightBound::Included(5)),
            20,
        );

        // Original value should still be present
        assert_eq!(
            map.m.get(&LeftBound::Included(1)).unwrap(),
            &BTreeSet::from([10])
        );
    }

    // ===================== ensure_boundary

    #[test]
    fn test_ensure_boundary_empty_map() {
        use LeftBound::*;
        let mut map = SpanMap::<i32, i32>::new();

        // Test with empty map
        map.ensure_boundary(Included(5));
        assert_eq!(map.m.len(), 2);
        assert!(map.m.contains_key(&Included(5)));
        assert!(map.m.get(&Included(5)).unwrap().is_empty());
    }

    #[test]
    fn test_ensure_boundary_existing_boundary() {
        use LeftBound::*;
        let mut map = SpanMap::<i32, i32>::new();

        // Insert initial boundary
        map.m.insert(Included(5), BTreeSet::from([1]));

        // Ensure same boundary
        map.ensure_boundary(Included(5));

        // Verify no changes
        assert_eq!(map.m.len(), 2);
        assert_eq!(map.m.get(&Included(5)).unwrap(), &BTreeSet::from([1]));
    }

    #[test]
    fn test_ensure_boundary_split_point() {
        use LeftBound::*;
        let mut map = SpanMap::<i32, i32>::new();

        // Insert initial boundary
        map.m.insert(Included(3), BTreeSet::from([1, 2]));

        // Split at a higher point
        map.ensure_boundary(Included(5));

        // Verify split result
        assert_eq!(map.m.len(), 3);
        assert_eq!(map.m.get(&Unbounded).unwrap(), &BTreeSet::from([]));
        assert_eq!(map.m.get(&Included(3)).unwrap(), &BTreeSet::from([1, 2]));
        assert_eq!(map.m.get(&Included(5)).unwrap(), &BTreeSet::from([1, 2]));
    }

    #[test]
    fn test_ensure_boundary_multiple_existing() {
        use LeftBound::*;
        let mut map = SpanMap::<i32, i32>::new();

        // Insert multiple boundaries
        map.m.insert(Included(1), BTreeSet::from([1]));
        map.m.insert(Included(3), BTreeSet::from([2]));
        map.m.insert(Included(5), BTreeSet::from([3]));

        // Ensure boundary between existing ones
        map.ensure_boundary(Included(2));

        // Verify result
        assert_eq!(map.m.len(), 5);
        assert_eq!(map.m.get(&Unbounded).unwrap(), &BTreeSet::from([]));
        assert_eq!(map.m.get(&Included(1)).unwrap(), &BTreeSet::from([1]));
        assert_eq!(map.m.get(&Included(2)).unwrap(), &BTreeSet::from([1]));
        assert_eq!(map.m.get(&Included(3)).unwrap(), &BTreeSet::from([2]));
        assert_eq!(map.m.get(&Included(5)).unwrap(), &BTreeSet::from([3]));
    }

    #[test]
    fn test_ensure_boundary_different_bound_types() {
        use LeftBound::*;
        let mut map = SpanMap::<i32, i32>::new();

        // Test with excluded bound
        map.ensure_boundary(Excluded(5));
        assert!(map.m.contains_key(&Excluded(5)));

        // Test with included bound after excluded
        map.ensure_boundary(Included(5));
        assert!(map.m.contains_key(&Included(5)));

        assert_eq!(map.m.len(), 3);
    }

    #[test]
    fn test_ensure_boundary_unbounded() {
        use LeftBound::*;
        let mut map = SpanMap::<i32, i32>::new();

        // Test with unbounded
        map.ensure_boundary(Unbounded);
        assert!(map.m.contains_key(&Unbounded));
        assert_eq!(map.m.len(), 1);

        // Add another boundary after unbounded
        map.ensure_boundary(Included(5));
        assert_eq!(map.m.len(), 2);
        assert!(map.m.contains_key(&Included(5)));
    }

    #[test]
    fn test_ensure_boundary_ordering() {
        use LeftBound::*;
        let mut map = SpanMap::<i32, i32>::new();

        // Insert boundaries in random order
        map.ensure_boundary(Included(5));
        map.ensure_boundary(Included(1));
        map.ensure_boundary(Included(3));

        // Verify correct ordering
        let keys: Vec<_> = map.m.keys().cloned().collect();
        assert_eq!(keys, vec![Unbounded, Included(1), Included(3), Included(5)]);
    }

    #[test]
    fn test_ensure_boundary_with_string_keys() {
        use LeftBound::*;
        let mut map = SpanMap::<String, i32>::new();

        map.ensure_boundary(Included("a".to_string()));
        assert!(map.m.contains_key(&Included("a".to_string())));

        map.ensure_boundary(Included("b".to_string()));
        assert_eq!(map.m.len(), 3);
        assert!(map.m.contains_key(&Included("b".to_string())));
    }

    // ===================== merge_adjacent_left

    #[test]
    fn test_merge_adjacent_left() {
        use LeftBound::*;

        let mut map = SpanMap::new();
        let set12: BTreeSet<i32> = vec![1, 2].into_iter().collect();
        let set23: BTreeSet<i32> = vec![2, 3].into_iter().collect();

        // Test case 1: Merge identical adjacent sets
        map.m.insert(Unbounded, set12.clone());
        map.m.insert(Included(5), set12.clone());
        map.merge_adjacent_left(Included(5));

        // Should only have one entry after merging
        assert_eq!(map.m.len(), 1);
        assert_eq!(map.m.get(&Unbounded), Some(&set12));

        // Test case 2: Don't merge different sets
        let mut map = SpanMap::new();
        map.m.insert(Unbounded, set12.clone());
        map.m.insert(Included(5), set23.clone());
        map.merge_adjacent_left(Included(5));

        // Should still have two entries
        assert_eq!(map.m.len(), 2);
        assert!(map.m.contains_key(&Included(5)));

        // Test case 3: Single entry (no left adjacent range)
        let mut map = SpanMap::new();
        map.m.insert(Included(5), set12.clone());
        map.merge_adjacent_left(Included(5));

        // Should remain unchanged
        assert_eq!(map.m.len(), 2);
        assert!(map.m.contains_key(&Included(5)));

        // Test case 4: Empty map
        let mut map: SpanMap<i32, i32> = SpanMap::new();
        map.merge_adjacent_left(Included(5));

        // Should remain empty
        assert_eq!(map.m.len(), 1);

        // Test case 5: Multiple boundaries, merge middle
        let mut map = SpanMap::new();
        map.m.insert(Unbounded, set12.clone());
        map.m.insert(Included(5), set12.clone());
        map.m.insert(Included(10), set23);
        map.merge_adjacent_left(Included(5));

        // Should have two entries after merging
        assert_eq!(map.m.len(), 2);
        assert!(!map.m.contains_key(&Included(5)));
        assert!(map.m.contains_key(&Included(10)));
    }
}
