use std::cmp::Ordering;
use std::collections::Bound;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeftBound<T> {
    Unbounded,
    Included(T),
    Excluded(T),
}

impl<T> fmt::Display for LeftBound<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LeftBound::Unbounded => write!(f, "(-∞"),
            LeftBound::Included(t) => write!(f, "[{}", t),
            LeftBound::Excluded(t) => write!(f, "({}", t),
        }
    }
}

impl<T> From<Bound<T>> for LeftBound<T> {
    fn from(bound: Bound<T>) -> Self {
        match bound {
            Bound::Unbounded => LeftBound::Unbounded,
            Bound::Included(t) => LeftBound::Included(t),
            Bound::Excluded(t) => LeftBound::Excluded(t),
        }
    }
}

impl<T> From<LeftBound<T>> for Bound<T> {
    fn from(bound: LeftBound<T>) -> Self {
        match bound {
            LeftBound::Unbounded => Bound::Unbounded,
            LeftBound::Included(t) => Bound::Included(t),
            LeftBound::Excluded(t) => Bound::Excluded(t),
        }
    }
}

impl<T> PartialOrd for LeftBound<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use LeftBound::*;

        match self {
            Unbounded => match other {
                Unbounded => Some(Ordering::Equal),
                _ => Some(Ordering::Less),
            },
            Included(l) => match other {
                Unbounded => Some(Ordering::Greater),
                Included(r) => l.partial_cmp(r),
                Excluded(r) => {
                    if l == r {
                        Some(Ordering::Less)
                    } else {
                        l.partial_cmp(r)
                    }
                }
            },
            Excluded(l) => match other {
                Unbounded => Some(Ordering::Greater),
                Included(r) => {
                    if l == r {
                        Some(Ordering::Greater)
                    } else {
                        l.partial_cmp(r)
                    }
                }
                Excluded(r) => l.partial_cmp(r),
            },
        }
    }
}

impl<T> Ord for LeftBound<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        use LeftBound::*;

        match self {
            Unbounded => match other {
                Unbounded => Ordering::Equal,
                _ => Ordering::Less,
            },
            Included(l) => match other {
                Unbounded => Ordering::Greater,
                Included(r) => l.cmp(r),
                Excluded(r) => {
                    if l == r {
                        Ordering::Less
                    } else {
                        l.cmp(r)
                    }
                }
            },
            Excluded(l) => match other {
                Unbounded => Ordering::Greater,
                Included(r) => {
                    if l == r {
                        Ordering::Greater
                    } else {
                        l.cmp(r)
                    }
                }
                Excluded(r) => l.cmp(r),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::*;

    #[test]
    fn test_left_bound_partial_ord() {
        // Test Unbounded comparisons
        assert_eq!(
            LeftBound::<f64>::Unbounded.partial_cmp(&LeftBound::Unbounded),
            Some(Ordering::Equal)
        );
        assert_eq!(
            LeftBound::<f64>::Unbounded.partial_cmp(&LeftBound::Included(0.0)),
            Some(Ordering::Less)
        );
        assert_eq!(
            LeftBound::<f64>::Unbounded.partial_cmp(&LeftBound::Excluded(0.0)),
            Some(Ordering::Less)
        );

        // Test Included comparisons
        assert_eq!(
            LeftBound::Included(0.0).partial_cmp(&LeftBound::Included(0.0)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            LeftBound::Included(0.0).partial_cmp(&LeftBound::Included(1.0)),
            Some(Ordering::Less)
        );
        assert_eq!(
            LeftBound::Included(1.0).partial_cmp(&LeftBound::Included(0.0)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            LeftBound::Included(0.0).partial_cmp(&LeftBound::Unbounded),
            Some(Ordering::Greater)
        );

        // Test Included vs Excluded with same value
        assert_eq!(
            LeftBound::Included(5.0).partial_cmp(&LeftBound::Excluded(5.0)),
            Some(Ordering::Less)
        );
        assert_eq!(
            LeftBound::Included(5.0).partial_cmp(&LeftBound::Excluded(4.0)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            LeftBound::Included(5.0).partial_cmp(&LeftBound::Excluded(6.0)),
            Some(Ordering::Less)
        );

        // Test Excluded comparisons
        assert_eq!(
            LeftBound::Excluded(0.0).partial_cmp(&LeftBound::Excluded(0.0)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            LeftBound::Excluded(0.0).partial_cmp(&LeftBound::Excluded(1.0)),
            Some(Ordering::Less)
        );
        assert_eq!(
            LeftBound::Excluded(1.0).partial_cmp(&LeftBound::Excluded(0.0)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            LeftBound::Excluded(0.0).partial_cmp(&LeftBound::Unbounded),
            Some(Ordering::Greater)
        );

        // Test Excluded vs Included with same value
        assert_eq!(
            LeftBound::Excluded(5.0).partial_cmp(&LeftBound::Included(5.0)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            LeftBound::Excluded(5.0).partial_cmp(&LeftBound::Included(4.0)),
            Some(Ordering::Greater)
        );
        assert_eq!(
            LeftBound::Excluded(5.0).partial_cmp(&LeftBound::Included(6.0)),
            Some(Ordering::Less)
        );
    }

    #[test]
    fn test_left_bound_ord() {
        // Test Unbounded comparisons
        assert_eq!(
            LeftBound::<i32>::Unbounded.cmp(&LeftBound::Unbounded),
            Ordering::Equal
        );
        assert_eq!(
            LeftBound::<i32>::Unbounded.cmp(&LeftBound::Included(0)),
            Ordering::Less
        );
        assert_eq!(
            LeftBound::<i32>::Unbounded.cmp(&LeftBound::Excluded(0)),
            Ordering::Less
        );

        // Test Included comparisons
        assert_eq!(
            LeftBound::Included(0).cmp(&LeftBound::Unbounded),
            Ordering::Greater
        );
        assert_eq!(
            LeftBound::Included(0).cmp(&LeftBound::Included(0)),
            Ordering::Equal
        );
        assert_eq!(
            LeftBound::Included(0).cmp(&LeftBound::Included(1)),
            Ordering::Less
        );
        assert_eq!(
            LeftBound::Included(1).cmp(&LeftBound::Included(0)),
            Ordering::Greater
        );

        // Special case: Included vs Excluded with same value
        assert_eq!(
            LeftBound::Included(5).cmp(&LeftBound::Excluded(5)),
            Ordering::Less
        );
        assert_eq!(
            LeftBound::Included(5).cmp(&LeftBound::Excluded(4)),
            Ordering::Greater
        );
        assert_eq!(
            LeftBound::Included(5).cmp(&LeftBound::Excluded(6)),
            Ordering::Less
        );

        // Test Excluded comparisons
        assert_eq!(
            LeftBound::Excluded(0).cmp(&LeftBound::Unbounded),
            Ordering::Greater
        );
        assert_eq!(
            LeftBound::Excluded(0).cmp(&LeftBound::Excluded(0)),
            Ordering::Equal
        );
        assert_eq!(
            LeftBound::Excluded(0).cmp(&LeftBound::Excluded(1)),
            Ordering::Less
        );
        assert_eq!(
            LeftBound::Excluded(1).cmp(&LeftBound::Excluded(0)),
            Ordering::Greater
        );

        // Special case: Excluded vs Included with same value
        assert_eq!(
            LeftBound::Excluded(5).cmp(&LeftBound::Included(5)),
            Ordering::Greater
        );
        assert_eq!(
            LeftBound::Excluded(5).cmp(&LeftBound::Included(4)),
            Ordering::Greater
        );
        assert_eq!(
            LeftBound::Excluded(5).cmp(&LeftBound::Included(6)),
            Ordering::Less
        );
    }
}
