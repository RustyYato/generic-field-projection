//! Projects through an `Rc`
//!
//! This clones the `Rc` and keeps it around to clean up the data, and also
//! holds on to a pointer to the field from the `Rc`'s allocation.

use type_list::map::{ListMap, Mapped};

use super::*;

use std::rc::Rc;

pub struct ProjectedRc<P: ?Sized, T: ?Sized> {
    _own:  Rc<P>,
    field: *const T,
}

impl<P: ?Sized, T: ?Sized> Deref for ProjectedRc<P, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.field }
    }
}

unsafe impl<F: ?Sized> PinnablePointer for Rc<F> {
}
impl<F: Field> ProjectTo<F> for Rc<F::Parent> {
    type Projection = ProjectedRc<F::Parent, F::Type>;

    fn project_to(self, field: F) -> Self::Projection {
        unsafe {
            let field = field.project_raw(&self as &_);
            ProjectedRc {
                _own: self,
                field,
            }
        }
    }
}

pub struct ProjectedRcSet<P: ?Sized, T: ?Sized> {
    _own:  Rc<P>,
    field: T,
}

pub struct Split<P: ?Sized>(Rc<P>);

call! {
    fn[P: ?Sized, T: ?Sized](&mut self: Split<P>, field: *const T) -> ProjectedRc<P, T> {
        ProjectedRc { _own: self.0.clone(), field }
    }
}

type_function! {
    for(P: ?Sized, T: ?Sized)
    fn(self: Split<P>, field: *const T) -> ProjectedRc<P, T> {
        ProjectedRc { _own: self.0.clone(), field }
    }
}

impl<P: ?Sized, T> ProjectedRcSet<P, T> {
    pub fn get<'a>(&'a self) -> Mapped<T, PtrToRef<'a>>
    where
        T: Copy + ListMap<PtrToRef<'a>>,
    {
        self.field.list_map(PtrToRef(PhantomData))
    }

    pub fn split(self) -> Mapped<T, Split<P>>
    where
        T: Copy + ListMap<Split<P>>,
    {
        self.field.list_map(Split(self._own))
    }
}

impl<'a, Parent: ?Sized, F: FieldList<Parent>> ProjectAll<Parent, F>
    for Rc<Parent>
{
    type Projection = ProjectedRcSet<Parent, F::Type>;

    #[inline]
    fn project_all(self, field: F) -> Self::Projection {
        unsafe {
            ProjectedRcSet {
                field: field.project_raw(&self as &_),
                _own:  self,
            }
        }
    }
}

impl<'a, F: FieldSet> ProjectToSet<F> for Rc<F::Parent> {
    type Projection = ProjectedRcSet<F::Parent, F::TypeSet>;

    #[inline]
    fn project_set_to(self, field: F) -> Self::Projection {
        unsafe {
            ProjectedRcSet {
                field: field.project_raw(&self as &_),
                _own:  self,
            }
        }
    }
}
