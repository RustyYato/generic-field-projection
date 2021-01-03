use super::*;
use type_list::{FieldList, ProjectRawMut, ProjectedMut};
use typsy::{
    cmp::Any,
    map::{Map, Mapped},
};

pub struct PtrToRefMut<'a>(PhantomData<&'a ()>);

typsy::call! {
    fn['a, T: 'a](&mut self: PtrToRefMut<'a>, ptr: *mut T) -> &'a mut T {
        unsafe { &mut *ptr }
    }
}

unsafe impl<T: ?Sized> PinnablePointer for &mut T {
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
    F: FieldList<Parent>,
    ProjectedMut<Parent, F>: Map<PtrToRefMut<'a>>,
    F: Copy + for<'b> Any<'b, FindOverlap<F>>,
{
    /// The projection of the type, can be used to directly access the field
    type Projection = Mapped<ProjectedMut<Parent, F>, PtrToRefMut<'a>>;

    /// projects to the given field
    fn project_all(self, field: F) -> Self::Projection {
        assert!(
            !field.any(FindOverlap::new(field)),
            "Found overlapping fields"
        );

        unsafe {
            field
                .map(ProjectRawMut::new(self))
                .map(PtrToRefMut(PhantomData))
        }
    }
}
