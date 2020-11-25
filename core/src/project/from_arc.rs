use super::*;

use crate::alloc::Arc;

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

type_function! {
    for(P: ?Sized, T: ?Sized)
    fn(self: Split<P>, field: *const T) -> ProjectedArc<P, T> {
        ProjectedArc { _own: self.0.clone(), field }
    }
}

impl<P: ?Sized, T> ProjectedArcSet<P, T> {
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
