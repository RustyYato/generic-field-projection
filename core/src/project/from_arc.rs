//! Projects through an `Arc`
//!
//! This clones the `Arc` and keeps it around to clean up the data, and also
//! holds on to a pointer to the field from the `Arc`'s allocation.

use type_list::{FieldList, ProjectRaw, Projected};
use typsy::map::{Map, Mapped};

use super::*;

use std::sync::Arc;

pub struct ProjectedArc<P, T> {
    _own:  Arc<P>,
    field: *const T,
}

unsafe impl<P, T> Send for ProjectedArc<P, T> where Arc<P>: Send
{
}
unsafe impl<P, T> Sync for ProjectedArc<P, T> where Arc<P>: Sync
{
}

impl<P, T> Deref for ProjectedArc<P, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.field }
    }
}

unsafe impl<T: ?Sized> PinnablePointer for Arc<T> {
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

pub struct ProjectedArcSet<P, T> {
    _own:  Arc<P>,
    field: T,
}

unsafe impl<P, T> Send for ProjectedArcSet<P, T> where Arc<P>: Send
{
}
unsafe impl<P, T> Sync for ProjectedArcSet<P, T> where Arc<P>: Sync
{
}

pub struct Split<P>(Arc<P>);

typsy::call! {
    fn[P, T](&mut self: Split<P>, field: *const T) -> ProjectedArc<P, T> {
        ProjectedArc { _own: self.0.clone(), field }
    }
}

impl<P, T> ProjectedArcSet<P, T> {
    pub fn get<'a>(&'a self) -> Mapped<T, PtrToRef<'a>>
    where
        T: Copy + Map<PtrToRef<'a>>,
    {
        self.field.map(PtrToRef(PhantomData))
    }

    pub fn split(self) -> Mapped<T, Split<P>>
    where
        T: Copy + Map<Split<P>>,
    {
        self.field.map(Split(self._own))
    }
}

impl<'a, Parent, F: FieldList<Parent>> ProjectAll<Parent, F> for Arc<Parent> {
    type Projection = ProjectedArcSet<Parent, Projected<Parent, F>>;

    #[inline]
    fn project_all(self, field: F) -> Self::Projection {
        unsafe {
            ProjectedArcSet {
                field: field.map(ProjectRaw::new(&self as &_)),
                _own:  self,
            }
        }
    }
}
