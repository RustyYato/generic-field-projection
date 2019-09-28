use super::*;

use crate::set::{
    tuple::TypeFunction,
    func::{PtrToRef, PtrToRefMut}
};

impl<'a, F: FieldSet> ProjectToSet<F> for &'a F::Parent
where F::Parent: 'a,
      F::TypeSet: TupleMap<PtrToRef<'a>> {
    type Projection = <F::TypeSet as TupleMap<PtrToRef<'a>>>::Output;

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

impl<'a, S: TupleAny<FindOverlapInner<&'a I>>, I: Field> TypeFunction<&'a I> for FindOverlap<&S> {
    type Output = bool;

    default fn call(&mut self, input: &'a I) -> bool {
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

impl<I: Field, J: Field> TypeFunction<&J> for FindOverlapInner<&I> {
    type Output = bool;

    fn call(&mut self, input: &J) -> bool {
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
      
      for<'b> F: TupleAny<FindOverlap<&'b F>> {
    type Projection = <F::TypeSetMut as TupleMap<PtrToRefMut<'a>>>::Output;

    fn project_set_to(self, field: F) -> Self::Projection {
        unsafe {
            if field.tup_any(FindOverlap {
                    counter: 0,
                    set: &field
                }) {
                panic!("Found overlapping fields")
            } else {
                let type_set = field.project_raw_mut(self);
                type_set.tup_map(PtrToRefMut::new())
            }
        }
    }
}
