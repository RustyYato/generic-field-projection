pub use core::iter::{once, Once};

/// Create a new compile-time field type for the given field
#[macro_export]
macro_rules! field {
    ($field_ty_name:ident ($parent:ty => $field_ty:ty), $field:ident, $value:expr) => {
        #[derive(Clone, Copy)]
        struct $field_ty_name;

        #[deny(safe_packed_borrows)]
        unsafe impl Field for $field_ty_name {
            type Name = $crate::macros::Once<&'static str>;
            type Parent = $parent;
            type Type = $field_ty;

            fn name(&self) -> Self::Name {
                $crate::macros::once(stringify!($field))
            }

            #[inline]
            unsafe fn project_raw(
                &self,
                ptr: *const Self::Parent,
            ) -> *const Self::Type {
                &(*ptr).$field
            }

            #[inline]
            unsafe fn project_raw_mut(
                &self,
                ptr: *mut Self::Parent,
            ) -> *mut Self::Type {
                &mut (*ptr).$field
            }
        }

        impl $field_ty_name {
            pub fn new() -> Self {
                $field_ty_name
            }
        }
    };
}
