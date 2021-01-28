use typsy::cmp::Any;

use super::*;

#[cfg(feature = "alloc")]
pub mod from_arc;
#[cfg(feature = "alloc")]
pub mod from_box;
pub mod from_mut;
pub mod from_pin;
#[cfg(feature = "alloc")]
pub mod from_rc;
pub mod from_ref;

use core::{marker::PhantomData, ops::Deref, pin::Pin};

use crate::pin::*;

impl<F: Field, T: ProjectTo<F>> ProjectTo<F> for Option<T> {
    type Projection = Option<T::Projection>;

    fn project_to(self, field: F) -> Self::Projection {
        match self {
            Some(value) => Some(value.project_to(field)),
            None => None,
        }
    }
}

pub struct PtrToRef<'a>(PhantomData<&'a ()>);

typsy::call! {
    fn['a, T: 'a + Sized](&mut self: PtrToRef<'a>, ptr: *const T) -> &'a T {
        unsafe { &*ptr }
    }
}

pub struct FindOverlap<S> {
    counter: u64,
    set: S,
}

impl<S> FindOverlap<S> {
    fn new(set: S) -> Self {
        FindOverlap { set, counter: 0 }
    }
}

pub struct FindOverlapInner<I> {
    id: u64,
    counter: u64,
    field: I,
}

typsy::call! {
    fn[S, F](&mut self: FindOverlap<S>, field: F) -> bool
    where(
        S: Copy + for<'b> Any<'b, FindOverlapInner<F>>,
        F: Field,
    ){
        self.counter += 1;

        self.set.any(FindOverlapInner {
            id: self.counter,
            counter: 0,
            field
        })
    }

    fn[I: Field, J: Field](&mut self: FindOverlapInner<I>, input: J) -> bool {
        self.counter += 1;

        if self.id <= self.counter {
            return false
        }

        let field = self.field.range();
        let input = input.range();

        is_overlapping(field, input)
    }
}

fn is_overlapping(a: Range<usize>, b: Range<usize>) -> bool {
    !b.is_empty()
        && !a.is_empty()
        && (a.contains(&b.start) || b.contains(&a.start))
}

#[cfg(test)]
mod tests {
    use super::is_overlapping;

    // empty ranges can't overlap, because they represent zero sized types
    // which can inhabit any address, and won't alias other values
    #[test]
    fn zst_no_overlap() {
        assert!(!is_overlapping(1..1, 0..2));
        assert!(!is_overlapping(1..1, 1..2));
        assert!(!is_overlapping(1..1, 1..1));
        assert!(!is_overlapping(1..1, 0..1));

        assert!(!is_overlapping(0..2, 1..1));
        assert!(!is_overlapping(1..2, 1..1));
        assert!(!is_overlapping(1..1, 1..1));
        assert!(!is_overlapping(0..1, 1..1));
    }

    #[test]
    fn does_overlap() {
        assert!(is_overlapping(1..4, 0..5));
        assert!(is_overlapping(1..4, 0..2));
        assert!(is_overlapping(1..4, 2..5));
        assert!(is_overlapping(1..4, 2..3));

        assert!(is_overlapping(0..5, 1..4));
        assert!(is_overlapping(0..2, 1..4));
        assert!(is_overlapping(2..5, 1..4));
        assert!(is_overlapping(2..3, 1..4));

        assert!(is_overlapping(1..4, 1..4));
        assert!(is_overlapping(1..4, 0..4));
        assert!(is_overlapping(0..4, 1..4));
    }

    #[test]
    fn is_disjoint() {
        assert!(!is_overlapping(1..4, 0..1));
        assert!(!is_overlapping(1..4, 4..5));
        assert!(!is_overlapping(1..4, 5..10));
        assert!(!is_overlapping(5..10, 1..4));
    }
}
