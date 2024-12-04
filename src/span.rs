use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;
use std::ops::RangeBounds;

use crate::bounds::LeftBound;
use crate::bounds::RightBound;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span<T>
where
    T: Ord,
{
    pub(crate) left: LeftBound<T>,
    pub(crate) right: RightBound<T>,
}

impl<T> fmt::Display for Span<T>
where
    T: Ord,
    T: fmt::Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.left, self.right)
    }
}

impl<T> Span<T>
where
    T: Ord,
{
    pub fn new(left: LeftBound<T>, right: RightBound<T>) -> Self {
        Self { left, right }
    }

    pub fn from_range<R>(range: R) -> Self
    where
        T: Clone,
        R: RangeBounds<T>,
    {
        Self::new(
            range.start_bound().cloned().into(),
            range.end_bound().cloned().into(),
        )
    }
}

impl<T> PartialOrd for Span<T>
where
    T: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        #[allow(clippy::collapsible_else_if)]
        if self.right < other.left {
            Some(Ordering::Less)
        } else if self.left > other.right {
            Some(Ordering::Greater)
        } else {
            if self == other {
                Some(Ordering::Equal)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;
    use std::ops::RangeFrom;
    use std::ops::RangeInclusive;
    use std::ops::RangeTo;
    use std::ops::RangeToInclusive;

    use super::*;

    #[test]
    fn test_display() {
        let rng = Span::new(LeftBound::<i32>::Included(1), RightBound::Excluded(5));
        assert_eq!(rng.to_string(), "[1, 5)");

        let rng = Span::new(LeftBound::<i32>::Excluded(3), RightBound::Included(5));
        assert_eq!(rng.to_string(), "(3, 5]");

        let rng = Span::new(LeftBound::<i32>::Unbounded, RightBound::Unbounded);
        assert_eq!(rng.to_string(), "(-∞, ∞)");
    }

    #[test]
    fn test_from_range() {
        // Test Range
        let range: Range<usize> = 1..5;
        let rng = Span::from_range(range);
        assert_eq!(rng.left, LeftBound::Included(1));
        assert_eq!(rng.right, RightBound::Excluded(5));

        // Test RangeInclusive
        let range: RangeInclusive<usize> = 1..=5;
        let rng = Span::from_range(range);
        assert_eq!(rng.left, LeftBound::Included(1));
        assert_eq!(rng.right, RightBound::Included(5));

        // Test RangeFrom
        let range: RangeFrom<usize> = 1..;
        let rng = Span::from_range(range);
        assert_eq!(rng.left, LeftBound::Included(1));
        assert_eq!(rng.right, RightBound::Unbounded);

        // Test RangeTo
        let range: RangeTo<usize> = ..5;
        let rng = Span::from_range(range);
        assert_eq!(rng.left, LeftBound::Unbounded);
        assert_eq!(rng.right, RightBound::Excluded(5));

        // Test RangeToInclusive
        let range: RangeToInclusive<usize> = ..=5;
        let rng = Span::from_range(range);
        assert_eq!(rng.left, LeftBound::Unbounded);
        assert_eq!(rng.right, RightBound::Included(5));
    }

    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    #[test]
    fn test_partial_ord() {
        // Test disjoint ranges
        let r1 = Span::new(LeftBound::Included(1), RightBound::Excluded(3));
        let r2 = Span::new(LeftBound::Included(4), RightBound::Excluded(6));
        assert!(r1 < r2);
        assert!(r2 > r1);

        // Test overlapping ranges
        let r1 = Span::new(LeftBound::Included(1), RightBound::Excluded(4));
        let r2 = Span::new(LeftBound::Included(2), RightBound::Excluded(5));
        assert_eq!(r1.partial_cmp(&r2), None);

        // Test equal ranges
        let r1 = Span::new(LeftBound::Included(1), RightBound::Excluded(3));
        let r2 = Span::new(LeftBound::Included(1), RightBound::Excluded(3));
        assert_eq!(r1, r2);
        assert!(!(r1 < r2));
        assert!(!(r1 > r2));

        // Test nested ranges
        let r1 = Span::new(LeftBound::Included(1), RightBound::Excluded(5));
        let r2 = Span::new(LeftBound::Included(2), RightBound::Excluded(4));
        assert_eq!(r1.partial_cmp(&r2), None);

        // Test touching ranges
        let r1 = Span::new(LeftBound::Included(1), RightBound::Excluded(3));
        let r2 = Span::new(LeftBound::Included(3), RightBound::Excluded(5));
        assert!(r1 < r2);
        assert!(r2 > r1);

        // Test unbounded ranges
        let r1 = Span::new(LeftBound::Unbounded, RightBound::Excluded(3));
        let r2 = Span::new(LeftBound::Included(3), RightBound::Unbounded);
        assert!(r1 < r2);
        assert!(r2 > r1);

        // Test point overlapping ranges
        let r1 = Span::new(LeftBound::Unbounded, RightBound::Included(3));
        let r2 = Span::new(LeftBound::Included(3), RightBound::Unbounded);
        assert_eq!(r1.partial_cmp(&r2), None);
    }
}
