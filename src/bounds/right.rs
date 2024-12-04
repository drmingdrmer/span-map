use std::cmp::Ordering;
use std::collections::Bound;
use std::fmt;

use crate::bounds::LeftBound;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RightBound<T> {
    Excluded(T),
    Included(T),
    Unbounded,
}

impl<T> fmt::Display for RightBound<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RightBound::Excluded(t) => write!(f, "{})", t),
            RightBound::Included(t) => write!(f, "{}]", t),
            RightBound::Unbounded => write!(f, "∞)"),
        }
    }
}

impl<T> RightBound<T> {
    /// Converts this right bound into a complementary left bound that would create
    /// adjacent non-overlapping ranges.
    ///
    /// For example, if we have two ranges:
    /// - Range1: (..., RightBound::Included(5))
    /// - Range2: (RightBound::Included(5).adjacent_left(), ...)
    ///
    /// # Examples
    /// ```
    /// # use span_map::bounds::{LeftBound, RightBound};
    ///
    /// let r1 = RightBound::Included(5);
    /// assert_eq!(r1.adjacent_left(), Some(LeftBound::Excluded(5)));
    ///
    /// let r2 = RightBound::Excluded(5);
    /// assert_eq!(r2.adjacent_left(), Some(LeftBound::Included(5)));
    ///
    /// let r3 = RightBound::<i32>::Unbounded;
    /// assert_eq!(r3.adjacent_left(), None);
    /// ```
    pub fn adjacent_left(&self) -> Option<LeftBound<T>>
    where
        T: Clone,
    {
        match self {
            RightBound::Unbounded => None,
            RightBound::Included(t) => Some(LeftBound::Excluded(t.clone())),
            RightBound::Excluded(t) => Some(LeftBound::Included(t.clone())),
        }
    }
}

impl<T> PartialOrd for RightBound<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use RightBound::*;

        match self {
            Excluded(r) => match other {
                Excluded(l) => r.partial_cmp(l),
                Included(l) => {
                    if r == l {
                        Some(Ordering::Less)
                    } else {
                        r.partial_cmp(l)
                    }
                }
                Unbounded => Some(Ordering::Less),
            },
            Included(r) => match other {
                Excluded(l) => {
                    if r == l {
                        Some(Ordering::Greater)
                    } else {
                        r.partial_cmp(l)
                    }
                }
                Included(l) => r.partial_cmp(l),
                Unbounded => Some(Ordering::Less),
            },
            Unbounded => match other {
                Unbounded => Some(Ordering::Equal),
                _ => Some(Ordering::Greater),
            },
        }
    }
}

impl<T> Ord for RightBound<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        use RightBound::*;

        match self {
            Excluded(r) => match other {
                Excluded(l) => r.cmp(l),
                Included(l) => {
                    if r == l {
                        Ordering::Less
                    } else {
                        r.cmp(l)
                    }
                }
                Unbounded => Ordering::Less,
            },
            Included(r) => match other {
                Excluded(l) => {
                    if r == l {
                        Ordering::Greater
                    } else {
                        r.cmp(l)
                    }
                }
                Included(l) => r.cmp(l),
                Unbounded => Ordering::Less,
            },
            Unbounded => match other {
                Unbounded => Ordering::Equal,
                _ => Ordering::Greater,
            },
        }
    }
}

impl<T> From<Bound<T>> for RightBound<T> {
    fn from(bound: Bound<T>) -> Self {
        match bound {
            Bound::Unbounded => RightBound::Unbounded,
            Bound::Included(t) => RightBound::Included(t),
            Bound::Excluded(t) => RightBound::Excluded(t),
        }
    }
}

impl<T> From<RightBound<T>> for Bound<T> {
    fn from(bound: RightBound<T>) -> Self {
        match bound {
            RightBound::Unbounded => Bound::Unbounded,
            RightBound::Included(t) => Bound::Included(t),
            RightBound::Excluded(t) => Bound::Excluded(t),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::*;

    #[test]
    fn test_right_bound_partial_ord() {
        // Test Excluded comparisons
        assert_eq!(
            RightBound::Excluded(0).partial_cmp(&RightBound::Excluded(0)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            RightBound::Excluded(0).partial_cmp(&RightBound::Excluded(1)),
            Some(Ordering::Less)
        );
        assert_eq!(
            RightBound::Excluded(1).partial_cmp(&RightBound::Excluded(0)),
            Some(Ordering::Greater)
        );

        // Test Excluded vs Included with same value
        assert_eq!(
            RightBound::Excluded(5).partial_cmp(&RightBound::Included(5)),
            Some(Ordering::Less)
        );
        assert_eq!(
            RightBound::Excluded(5).partial_cmp(&RightBound::Included(4)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            RightBound::Excluded(5).partial_cmp(&RightBound::Included(6)),
            Some(Ordering::Less)
        );

        // Test Included comparisons
        assert_eq!(
            RightBound::Included(0).partial_cmp(&RightBound::Included(0)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            RightBound::Included(0).partial_cmp(&RightBound::Included(1)),
            Some(Ordering::Less)
        );
        assert_eq!(
            RightBound::Included(1).partial_cmp(&RightBound::Included(0)),
            Some(Ordering::Greater)
        );

        // Test Included vs Excluded with same value
        assert_eq!(
            RightBound::Included(5).partial_cmp(&RightBound::Excluded(5)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            RightBound::Included(5).partial_cmp(&RightBound::Excluded(4)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            RightBound::Included(5).partial_cmp(&RightBound::Excluded(6)),
            Some(Ordering::Less)
        );

        // Test Unbounded comparisons
        assert_eq!(
            RightBound::<i32>::Unbounded.partial_cmp(&RightBound::Unbounded),
            Some(Ordering::Equal)
        );
        assert_eq!(
            RightBound::<i32>::Unbounded.partial_cmp(&RightBound::Included(0)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            RightBound::<i32>::Unbounded.partial_cmp(&RightBound::Excluded(0)),
            Some(Ordering::Greater)
        );

        // Test comparisons with Unbounded
        assert_eq!(
            RightBound::Included(0).partial_cmp(&RightBound::Unbounded),
            Some(Ordering::Less)
        );
        assert_eq!(
            RightBound::Excluded(0).partial_cmp(&RightBound::Unbounded),
            Some(Ordering::Less)
        );

        // Test with floating point numbers
        assert_eq!(
            RightBound::Included(1.0).partial_cmp(&RightBound::Included(1.0)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            RightBound::Excluded(1.0).partial_cmp(&RightBound::Excluded(1.0)),
            Some(Ordering::Equal)
        );

        // Test transitivity
        let bounds = [
            RightBound::Excluded(0),
            RightBound::Included(0),
            RightBound::Excluded(1),
            RightBound::Included(1),
            RightBound::Unbounded,
        ];

        for a in bounds.iter() {
            for b in bounds.iter() {
                for c in bounds.iter() {
                    if let (Some(ord1), Some(ord2)) = (a.partial_cmp(b), b.partial_cmp(c)) {
                        if ord1 == ord2 && ord1 != Ordering::Equal {
                            assert_eq!(a.partial_cmp(c), Some(ord1));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_right_bound_ordering() {
        // Test Excluded comparisons
        assert_eq!(
            RightBound::Excluded(0).cmp(&RightBound::Excluded(0)),
            Ordering::Equal
        );
        assert_eq!(
            RightBound::Excluded(0).cmp(&RightBound::Excluded(1)),
            Ordering::Less
        );
        assert_eq!(
            RightBound::Excluded(1).cmp(&RightBound::Excluded(0)),
            Ordering::Greater
        );

        // Test Excluded vs Included with same value
        assert_eq!(
            RightBound::Excluded(5).cmp(&RightBound::Included(5)),
            Ordering::Less
        );
        assert_eq!(
            RightBound::Excluded(5).cmp(&RightBound::Included(4)),
            Ordering::Greater
        );
        assert_eq!(
            RightBound::Excluded(5).cmp(&RightBound::Included(6)),
            Ordering::Less
        );

        // Test Included comparisons
        assert_eq!(
            RightBound::Included(0).cmp(&RightBound::Included(0)),
            Ordering::Equal
        );
        assert_eq!(
            RightBound::Included(0).cmp(&RightBound::Included(1)),
            Ordering::Less
        );
        assert_eq!(
            RightBound::Included(1).cmp(&RightBound::Included(0)),
            Ordering::Greater
        );

        // Test Included vs Excluded with same value
        assert_eq!(
            RightBound::Included(5).cmp(&RightBound::Excluded(5)),
            Ordering::Greater
        );
        assert_eq!(
            RightBound::Included(5).cmp(&RightBound::Excluded(4)),
            Ordering::Greater
        );
        assert_eq!(
            RightBound::Included(5).cmp(&RightBound::Excluded(6)),
            Ordering::Less
        );

        // Test Unbounded comparisons
        assert_eq!(
            RightBound::<usize>::Unbounded.cmp(&RightBound::Unbounded),
            Ordering::Equal
        );
        assert_eq!(
            RightBound::Unbounded.cmp(&RightBound::Included(0)),
            Ordering::Greater
        );
        assert_eq!(
            RightBound::Unbounded.cmp(&RightBound::Excluded(0)),
            Ordering::Greater
        );

        // Test comparisons with Unbounded
        assert_eq!(
            RightBound::Included(0).cmp(&RightBound::Unbounded),
            Ordering::Less
        );
        assert_eq!(
            RightBound::Excluded(0).cmp(&RightBound::Unbounded),
            Ordering::Less
        );
    }

    #[test]
    fn test_next_left() {
        // Test Unbounded case
        assert_eq!(RightBound::<usize>::Unbounded.adjacent_left(), None);

        // Test Included case
        assert_eq!(
            RightBound::Included(5).adjacent_left(),
            Some(LeftBound::Excluded(5))
        );

        // Test Excluded case
        assert_eq!(
            RightBound::Excluded(5).adjacent_left(),
            Some(LeftBound::Included(5))
        );
    }
}
