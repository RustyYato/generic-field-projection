use super::*;

pub mod tuple;

/// Represents a set of fields
pub unsafe trait FieldSet {
    /// The type that the field comes from
    type Parent: ?Sized;

    /// The type of the field itself
    type TypeSet;

    /// The type of the field itself
    type TypeSetMut;

    /// projects the raw pointer from the `Parent` type to the field `Type`
    ///
    /// # Safety
    ///
    /// * `ptr` must point to a valid, initialized allocation of `Parent`
    /// * the projection is not safe to write to
    unsafe fn project_raw(&self, ptr: *const Self::Parent) -> Self::TypeSet;

    /// projects the raw pointer from the `Parent` type to the field `Type`
    ///
    /// # Safety
    ///
    /// * `ptr` must point to a valid, initialized allocation of `Parent`
    /// * the projection is not safe to write to
    unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> Self::TypeSetMut;
}

/// This munches through each identifier in the list,
/// and creates a tuple using all of the identifiers in the list
///
/// This just makes it easier to implement `FieldSet` for a large number of tuples
///
/// Just provide `impl_tuple` with the required number of identifiers, and it will automatically
/// generate all of the required impls for all tuples up to the given size
macro_rules! impl_tuple {
    () => {};
    ($first:ident $($T:ident)*) => {
        impl_tuple! { $($T)* }

        unsafe impl<$first: Field, $($T: Field<Parent = $first::Parent>),*> FieldSet for ($first, $($T),*) {
            type Parent = $first::Parent;

            type TypeSet = (*const $first::Type, $(*const $T::Type),*);
            type TypeSetMut = (*mut $first::Type, $(*mut $T::Type),*);

            #[inline]
            #[allow(non_snake_case)]
            unsafe fn project_raw(&self, ptr: *const Self::Parent) -> Self::TypeSet {
                let ($first, $($T),*) = self;
                (
                    $first.project_raw(ptr),
                    $($T.project_raw(ptr)),*
                )
            }

            #[inline]
            #[allow(non_snake_case)]
            unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> Self::TypeSetMut {
                let ($first, $($T),*) = self;
                (
                    $first.project_raw_mut(ptr),
                    $($T.project_raw_mut(ptr)),*
                )
            }
        }

    };
}

impl_tuple! {
    A B C D
    E F G H
    I J K L
    M N O P
}
