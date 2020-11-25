use super::*;

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

impl<'a, F: FieldSet> ProjectToSet<F> for &'a F::Parent
where
    F::Parent: 'a,
    F::TypeSet: TupleMap<PtrToRef<'a>>,
{
    type Projection = TMap<F::TypeSet, PtrToRef<'a>>;

    #[inline]
    fn project_set_to(self, field: F) -> Self::Projection {
        unsafe {
            let type_set = field.project_raw(self);
            type_set.tup_map(PtrToRef(PhantomData))
        }
    }
}
