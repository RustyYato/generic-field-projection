use super::*;

use core::pin::Pin;
use core::ops::Deref;

use crate::pin::*;

use crate::set::{
    tuple::TypeFunction,
    func::{PtrToRef, PtrToRefMut}
};

impl<'a, F: FieldSet> ProjectToSet<F> for &'a F::Parent
where F::Parent: 'a,
      F::TypeSet: TupleMap<PtrToRef<'a>> {
    type Projection = TMap<F::TypeSet, PtrToRef<'a>>;

    #[inline]
    fn project_set_to(self, field: F) -> Self::Projection {
        unsafe {
            let type_set = field.project_raw(self);
            type_set.tup_map(PtrToRef::new())
        }
    }
}

pub struct FindOverlap<S> {
    counter: u64,
    set: S
}

impl<S, I: Field> TypeFunction<I> for FindOverlap<S>
where S: Copy + TupleAny<FindOverlapInner<I>> {
    type Output = bool;

    #[inline]
    fn call(&mut self, input: I) -> bool {
        self.counter += 1;
        self.set.tup_any(FindOverlapInner {
            id: self.counter,
            counter: 0,
            field: input
        })
    }
}

pub struct FindOverlapInner<I> {
    id: u64,
    counter: u64,
    field: I
}

impl<I: Field, J: Field> TypeFunction<J> for FindOverlapInner<I> {
    type Output = bool;

    #[inline]
    fn call(&mut self, input: J) -> bool {
        self.counter += 1;

        if self.id <= self.counter {
            return false
        }

        self.field.name().zip(input.name())
            .all(|(i, j)| i == j)
    }
}

impl<'a, F: FieldSet> ProjectToSet<F> for &'a mut F::Parent
where F::Parent: 'a,
      F::TypeSetMut: TupleMap<PtrToRefMut<'a>>,
      
      F: Copy + TupleAny<FindOverlap<F>> {
    type Projection = TMap<F::TypeSetMut, PtrToRefMut<'a>>;

    #[inline]
    fn project_set_to(self, field: F) -> Self::Projection {
        unsafe {
            if field.tup_any(FindOverlap {
                    counter: 0,
                    set: field
                }) {
                panic!("Found overlapping fields")
            } else {
                let type_set = field.project_raw_mut(self);
                type_set.tup_map(PtrToRefMut::new())
            }
        }
    }
}

pub struct MakePin;
pub struct MakeRef;

pub struct CheckMake;

impl<F: Field> TypeFunction<PPF<F>> for CheckMake {
    type Output = MakePin;

    #[inline]
    fn call(&mut self, _: PPF<F>) -> Self::Output { MakePin }
}

impl<F: Field> TypeFunction<PTR<F>> for CheckMake {
    type Output = MakeRef;

    #[inline]
    fn call(&mut self, _: PTR<F>) -> Self::Output { MakeRef }
}

pub struct PinCombine;

impl<T: Deref> TypeFunction<(MakePin, T)> for PinCombine {
    type Output = Pin<T>;

    #[inline]
    fn call(&mut self, (MakePin, value): (MakePin, T)) -> Self::Output {
        unsafe { Pin::new_unchecked(value) }
    }
}

impl<T> TypeFunction<(MakeRef, T)> for PinCombine {
    type Output = T;

    #[inline]
    fn call(&mut self, (MakeRef, value): (MakeRef, T)) -> Self::Output {
        value
    }
}

impl<F: Copy + FieldSet, P: ProjectToSet<F> + Deref> ProjectToSet<F> for Pin<P>
where
    F: TupleMap<CheckMake>,
    TMap<F, CheckMake>: TupleZip<P::Projection, PinCombine>
{
    type Projection = TZip<TMap<F, CheckMake>, P::Projection, PinCombine>;

    #[inline]
    fn project_set_to(self, field: F) -> Self::Projection {
        let check_make = field.tup_map(CheckMake);
        unsafe {
            let project = Pin::into_inner_unchecked(self)
                .project_set_to(field);
            
            check_make.tup_zip(project, PinCombine)
        }
    }
}
