use super::*;

use type_list::{FieldList, ProjectRaw, Projected};
use typsy::map::{Map, Mapped};

unsafe impl<F: ?Sized> PinnablePointer for &F {
}
impl<'a, F: Field> ProjectTo<F> for &'a F::Parent
where
    F::Parent: 'a,
    F::Type: 'a,
{
    type Projection = &'a F::Type;

    fn project_to(self, field: F) -> Self::Projection {
        unsafe { &*field.project_raw(self) }
    }
}

impl<'a, Parent: ?Sized, F: FieldList<Parent>> ProjectAll<Parent, F>
    for &'a Parent
where
    Parent: 'a,
    Projected<Parent, F>: Map<PtrToRef<'a>>,
{
    type Projection = Mapped<Projected<Parent, F>, PtrToRef<'a>>;

    #[inline]
    fn project_all(self, field: F) -> Self::Projection {
        unsafe { field.map(ProjectRaw::new(self)).map(PtrToRef(PhantomData)) }
    }
}
