use super::*;
use type_list::{FieldList, ProjectRawMut, ProjectedMut};
use typsy::{
    call::Simple,
    cmp::Any,
    map::{Map, Mapped},
};

pub struct PtrToRefMut<'a>(PhantomData<&'a ()>);

typsy::call! {
    fn['a, T: 'a + ?Sized](&mut self: PtrToRefMut<'a>, ptr: *mut T) -> &'a mut T {
        unsafe { &mut *ptr }
    }
}

unsafe impl<F: ?Sized> PinnablePointer for &mut F {
}
impl<'a, F: Field> ProjectTo<F> for &'a mut F::Parent
where
    F::Parent: 'a,
    F::Type: 'a,
{
    type Projection = &'a mut F::Type;

    fn project_to(self, field: F) -> Self::Projection {
        unsafe { &mut *field.project_raw_mut(self) }
    }
}

impl<'a, F, Parent> ProjectAll<Parent, F> for &'a mut Parent
where
    Parent: ?Sized,
    F: FieldList<Parent>,
    ProjectedMut<Parent, F>: Map<Simple<PtrToRefMut<'a>>>,
    F: Copy + for<'b> Any<'b, Simple<FindOverlap<F>>>,
{
    /// The projection of the type, can be used to directly access the field
    type Projection = Mapped<ProjectedMut<Parent, F>, Simple<PtrToRefMut<'a>>>;

    /// projects to the given field
    fn project_all(self, field: F) -> Self::Projection {
        assert!(
            !field.any(Simple(FindOverlap::new(field))),
            "Found overlapping fields"
        );

        unsafe {
            field
                .map(Simple(ProjectRawMut::new(self)))
                .map(Simple(PtrToRefMut(PhantomData)))
        }
    }
}
