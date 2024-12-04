mod left;
mod right;

use std::cmp::Ordering;

pub use left::LeftBound;
pub use right::RightBound;

impl<T> PartialEq<RightBound<T>> for LeftBound<T>
where
    T: Ord,
{
    fn eq(&self, other: &RightBound<T>) -> bool {
        match (self, other) {
            (LeftBound::Included(a), RightBound::Included(b)) => a == b,
            _ => false,
        }
    }
}

impl<T> PartialEq<LeftBound<T>> for RightBound<T>
where
    T: Ord,
{
    fn eq(&self, other: &LeftBound<T>) -> bool {
        other.eq(self)
    }
}

impl<T> PartialOrd<RightBound<T>> for LeftBound<T>
where
    T: Ord,
{
    fn partial_cmp(&self, other: &RightBound<T>) -> Option<Ordering> {
        match self {
            LeftBound::Unbounded => Some(Ordering::Less),
            LeftBound::Included(l) => match other {
                RightBound::Excluded(r) => {
                    if l == r {
                        Some(Ordering::Greater)
                    } else {
                        l.partial_cmp(r)
                    }
                }
                RightBound::Included(r) => l.partial_cmp(r),
                RightBound::Unbounded => Some(Ordering::Less),
            },
            LeftBound::Excluded(l) => match other {
                RightBound::Excluded(r) => {
                    if l == r {
                        Some(Ordering::Greater)
                    } else {
                        l.partial_cmp(r)
                    }
                }
                RightBound::Included(r) => {
                    if l == r {
                        Some(Ordering::Greater)
                    } else {
                        l.partial_cmp(r)
                    }
                }
                RightBound::Unbounded => Some(Ordering::Less),
            },
        }
    }
}

impl<T> PartialOrd<LeftBound<T>> for RightBound<T>
where
    T: Ord,
{
    fn partial_cmp(&self, other: &LeftBound<T>) -> Option<Ordering> {
        let res = other.partial_cmp(self);
        res.map(|o| o.reverse())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_left_right_equality() {
        assert_eq!(LeftBound::Included(5), RightBound::Included(5));
        assert_ne!(LeftBound::Included(5), RightBound::Included(6));
        assert_ne!(LeftBound::Excluded(5), RightBound::Included(5));
        assert_ne!(LeftBound::<usize>::Unbounded, RightBound::Unbounded);
    }

    #[test]
    fn test_left_bound_ordering() {
        // Test Unbounded
        assert!(LeftBound::Unbounded < RightBound::Excluded(0));
        assert!(LeftBound::Unbounded < RightBound::Included(0));
        assert!(LeftBound::<usize>::Unbounded < RightBound::Unbounded);

        // Test Included
        assert!(LeftBound::Included(5) > RightBound::Excluded(4));
        assert!(LeftBound::Included(5) > RightBound::Included(4));
        assert!(LeftBound::Included(5) > RightBound::Excluded(5));
        assert!(LeftBound::Included(5) == RightBound::Included(5));
        assert!(LeftBound::Included(5) < RightBound::Excluded(6));
        assert!(LeftBound::Included(5) < RightBound::Included(6));
        assert!(LeftBound::Included(5) < RightBound::Unbounded);

        // Test Excluded
        assert!(LeftBound::Excluded(5) > RightBound::Excluded(4));
        assert!(LeftBound::Excluded(5) > RightBound::Included(4));
        assert!(LeftBound::Excluded(5) > RightBound::Excluded(5));
        assert!(LeftBound::Excluded(5) > RightBound::Included(5));
        assert!(LeftBound::Excluded(5) < RightBound::Excluded(6));
        assert!(LeftBound::Excluded(5) < RightBound::Included(6));
        assert!(LeftBound::Excluded(5) < RightBound::Unbounded);
    }

    #[test]
    fn test_right_bound_ordering() {
        // Test Unbounded
        assert!(RightBound::<usize>::Unbounded > LeftBound::Excluded(0));
        assert!(RightBound::Unbounded > LeftBound::Included(0));
        assert!(RightBound::<usize>::Unbounded > LeftBound::Unbounded);

        // Test Included
        assert!(RightBound::Included(5) > LeftBound::Excluded(4));
        assert!(RightBound::Included(5) > LeftBound::Included(4));
        assert!(RightBound::Included(5) < LeftBound::Excluded(5));
        assert!(RightBound::Included(5) == LeftBound::Included(5));
        assert!(RightBound::Included(5) < LeftBound::Excluded(6));
        assert!(RightBound::Included(5) < LeftBound::Included(6));
        assert!(RightBound::Included(5) > LeftBound::Unbounded);

        // Test Excluded
        assert!(RightBound::Excluded(5) > LeftBound::Excluded(4));
        assert!(RightBound::Excluded(5) > LeftBound::Included(4));
        assert!(RightBound::Excluded(5) < LeftBound::Excluded(5));
        assert!(RightBound::Excluded(5) < LeftBound::Included(5));
        assert!(RightBound::Excluded(5) < LeftBound::Excluded(6));
        assert!(RightBound::Excluded(5) < LeftBound::Included(6));
        assert!(RightBound::Excluded(5) > LeftBound::Unbounded);
    }
}
