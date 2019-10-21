use super::*;

pub mod tuple;

use tuple::Tuple;

pub unsafe trait FieldSet: Tuple {
    /// The type that the field comes from
    type Parent: ?Sized;

    /// The type of the field itself
    type TypeSet: Tuple;

    /// The type of the field itself
    type TypeSetMut: Tuple;

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
