use super::*;

use crate::set::tuple::*;

#[cfg(any(not(feature = "no_std"), feature = "alloc"))]
pub mod from_arc;
#[cfg(any(not(feature = "no_std"), feature = "alloc"))]
pub mod from_box;
pub mod from_mut;
pub mod from_pin;
#[cfg(any(not(feature = "no_std"), feature = "alloc"))]
pub mod from_rc;
pub mod from_ref;

use std::{marker::PhantomData, ops::Deref, pin::Pin};

use crate::pin::*;

pub struct PtrToRef<'a>(PhantomData<&'a ()>);

type_function! {
    for('a, T: 'a + ?Sized)
    fn(self: PtrToRef<'a>, ptr: *const T) -> &'a T {
        unsafe { &*ptr }
    }
}

pub struct FindOverlap<S> {
    counter: u64,
    set:     S,
}

impl<S> FindOverlap<S> {
    fn new(set: S) -> Self {
        FindOverlap {
            set,
            counter: 0,
        }
    }
}

pub struct FindOverlapInner<I> {
    id:      u64,
    counter: u64,
    field:   I,
}

type_function! {
    for(S: Copy + TupleAny<FindOverlapInner<I>>, I: Field)
    fn(self: FindOverlap<S>, input: I) -> bool {
        self.counter += 1;

        self.set.tup_any(FindOverlapInner {
            id: self.counter,
            counter: 0,
            field: input
        })
    }

    for(I: Field, J: Field)
    fn(self: FindOverlapInner<I>, input: J) -> bool {
        self.counter += 1;

        if self.id <= self.counter {
            return false
        }

        self.field.name().zip(input.name())
            .all(|(i, j)| i == j)
    }
}
