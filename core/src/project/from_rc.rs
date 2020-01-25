//! Projects through an `Rc`
//! 
//! This clones the `Rc` and keeps it around to clean up the data, and also
//! holds on to a pointer to the field from the `Rc`'s allocation.

use super::*;

use crate::alloc::Rc;

pub struct ProjectedRc<P: ?Sized, T: ?Sized> {
    // to clean up the allocation once all projections are dropped
    _own: Rc<P>,
    // The field being projected onto
    field: *const T,
}

impl<P: ?Sized, T: ?Sized> Deref for ProjectedRc<P, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.field }
    }
}

unsafe impl<F: ?Sized> PinnablePointer for Rc<F> {}
impl<F: Field> ProjectTo<F> for Rc<F::Parent> {
    type Projection = ProjectedRc<F::Parent, F::Type>;

    fn project_to(self, field: F) -> Self::Projection {
        unsafe {
            let field = field.project_raw(&self as &_);
            ProjectedRc { _own: self, field }
        }
    }
}

pub struct ProjectedRcSet<P: ?Sized, T: ?Sized> {
    // to clean up the allocation once all projections are dropped
    _own: Rc<P>,
    // The fields being projected onto
    field: T,
}

pub struct Split<P: ?Sized>(Rc<P>);

type_function! {
    for(P: ?Sized, T: ?Sized)
    fn(self: Split<P>, field: *const T) -> ProjectedRc<P, T> {
        ProjectedRc { _own: self.0.clone(), field }
    }
}

impl<P: ?Sized, T> ProjectedRcSet<P, T> {
    pub fn get<'a>(&'a self) -> TMap<T, PtrToRef<'a>>
    where
        T: Copy + TupleMap<PtrToRef<'a>>,
    {
        self.field.tup_map(PtrToRef(PhantomData))
    }

    pub fn split(self) -> TMap<T, Split<P>>
    where
        T: Copy + TupleMap<Split<P>>,
    {
        self.field.tup_map(Split(self._own))
    }
}

impl<'a, F: FieldSet> ProjectToSet<F> for Rc<F::Parent> {
    type Projection = ProjectedRcSet<F::Parent, F::TypeSet>;

    #[inline]
    fn project_set_to(self, field: F) -> Self::Projection {
        unsafe {
            ProjectedRcSet {
                field: field.project_raw(&self as &_),
                _own: self,
            }
        }
    }
}
