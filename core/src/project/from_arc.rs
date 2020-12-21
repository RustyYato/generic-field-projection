//! Projects through an `Arc`
//!
//! This clones the `Arc` and keeps it around to clean up the data, and also
//! holds on to a pointer to the field from the `Arc`'s allocation.

use type_list::map::{ListMap, Mapped};

use super::*;

use std::sync::Arc;

pub struct ProjectedArc<P: ?Sized, T: ?Sized> {
    _own:  Arc<P>,
    field: *const T,
}

unsafe impl<P: ?Sized, T: ?Sized> Send for ProjectedArc<P, T> where Arc<P>: Send
{
}
unsafe impl<P: ?Sized, T: ?Sized> Sync for ProjectedArc<P, T> where Arc<P>: Sync
{
}

impl<P: ?Sized, T: ?Sized> Deref for ProjectedArc<P, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.field }
    }
}

unsafe impl<F: ?Sized> PinnablePointer for Arc<F> {
}
impl<F: Field> ProjectTo<F> for Arc<F::Parent> {
    type Projection = ProjectedArc<F::Parent, F::Type>;

    fn project_to(self, field: F) -> Self::Projection {
        unsafe {
            let field = field.project_raw(&self as &_);
            ProjectedArc {
                _own: self,
                field,
            }
        }
    }
}

pub struct ProjectedArcSet<P: ?Sized, T: ?Sized> {
    _own:  Arc<P>,
    field: T,
}

unsafe impl<P: ?Sized, T: ?Sized> Send for ProjectedArcSet<P, T> where
    Arc<P>: Send
{
}
unsafe impl<P: ?Sized, T: ?Sized> Sync for ProjectedArcSet<P, T> where
    Arc<P>: Sync
{
}

pub struct Split<P: ?Sized>(Arc<P>);

call! {
    fn[P: ?Sized, T: ?Sized](&mut self: Split<P>, field: *const T) -> ProjectedArc<P, T> {
        ProjectedArc { _own: self.0.clone(), field }
    }
}

type_function! {
    for(P: ?Sized, T: ?Sized)
    fn(self: Split<P>, field: *const T) -> ProjectedArc<P, T> {
        ProjectedArc { _own: self.0.clone(), field }
    }
}

impl<P: ?Sized, T> ProjectedArcSet<P, T> {
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
    for Arc<Parent>
{
    type Projection = ProjectedArcSet<Parent, F::Type>;

    #[inline]
    fn project_all(self, field: F) -> Self::Projection {
        unsafe {
            ProjectedArcSet {
                field: field.project_raw(&self as &_),
                _own:  self,
            }
        }
    }
}

impl<'a, F: FieldSet> ProjectToSet<F> for Arc<F::Parent> {
    type Projection = ProjectedArcSet<F::Parent, F::TypeSet>;

    #[inline]
    fn project_set_to(self, field: F) -> Self::Projection {
        unsafe {
            ProjectedArcSet {
                field: field.project_raw(&self as &_),
                _own:  self,
            }
        }
    }
}
