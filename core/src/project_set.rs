use super::*;

use crate::set::{
    tuple::TypeFunction,
    func::{PtrToRef, PtrToRefMut, Enumerate}
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

pub struct FindOverlap<'a, S>(&'a S);

impl<'a, S, I: Field> TypeFunction<(u64, I)> for FindOverlap<'a, S>
where S: FieldSet + TupleRef<'a>,
      S::Ref: TupleMap<Enumerate>,
      <S::Ref as TupleMap<Enumerate>>::Output: TupleAny<FindOverlapInner<I>> {
    type Output = bool;

    fn call(&mut self, (id, input): (u64, I)) -> bool {
        self.0.as_tup_ref()
            .tup_map(Enumerate::start_at(0))
            .tup_any(FindOverlapInner(id, input))
    }
}

pub struct FindOverlapInner<I>(u64, I);

impl<I: Field, J: Field> TypeFunction<(u64, &J)> for FindOverlapInner<&I> {
    type Output = bool;

    fn call(&mut self, (id, input): (u64, &J)) -> bool {
        if self.0 <= id {
            return false
        }

        let mut i = self.1.name();
        let mut j = input.name();
        
        loop {
            match (i.next(), j.next()) {
                (Some(x), Some(y)) => {
                    if x != y {
                        return false
                    }
                },
                _ => return true
            }
        }
    }
}

impl<'a, 'b, F: FieldSet + TupleRef<'b>> ProjectToSet<&'b F> for &'a mut F::Parent
where F::Parent: 'a,
      F::TypeSetMut: TupleMap<PtrToRefMut<'a>>,
      
      F::Ref: TupleMap<Enumerate>,
      <F::Ref as TupleMap<Enumerate>>::Output: TupleAny<FindOverlap<'b, F>> {
    type Projection = <F::TypeSetMut as TupleMap<PtrToRefMut<'a>>>::Output;

    fn project_set_to(self, field: &'b F) -> Self::Projection {
        unsafe {
            if field.as_tup_ref()
                .tup_map(Enumerate::start_at(0))
                .tup_any(FindOverlap(field)) {
                panic!("Found overlapping fields")
            } else {
                let type_set = field.project_raw_mut(self);
                type_set.tup_map(PtrToRefMut::new())
            }
        }
    }
}
