use super::*;

use crate::type_list::map::{ListMap, Mapped};

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
    F::Type: ListMap<PtrToRef<'a>>,
{
    type Projection = Mapped<F::Type, PtrToRef<'a>>;

    #[inline]
    fn project_all(self, field: F) -> Self::Projection {
        unsafe {
            let type_set = field.project_raw(self);
            type_set.map(PtrToRef(PhantomData))
        }
    }
}
