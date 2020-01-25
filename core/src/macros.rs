pub use core::iter::{once, Once};

/// Create a new compile-time field type for the given field
/// 
/// This macro can be used like so,
/// 
/// ```
/// # #![feature(raw_ref_op)]
/// // given
/// 
/// struct Foo {
///     x: i32,
/// }
/// 
/// // you can do
/// 
/// # #[allow(deprecated)] gfp_core::
/// field!(Foo_x (Foo => i32), x)
/// ```
/// 
/// # Deprecated
/// 
/// Please use the `#[derive(Field)]` macro instead
#[deprecated]
#[macro_export]
macro_rules! field {
    ($field_ty_name:ident ($parent:ty => $field_ty:ty), $field:ident, $value:expr) => {
        #[derive(Clone, Copy)]
        struct $field_ty_name;

        #[deny(safe_packed_borrows)]
        unsafe impl Field for $field_ty_name {
            type Parent = $parent;
            type Type = $field_ty;
            type Name = $crate::macros::Once<&'static str>;

            fn name(&self) -> Self::Name {
                $crate::macros::once(stringify!($field))
            }

            #[inline]
            unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
                let Self { $field, .. };
                &raw const (*ptr).$field
            }

            #[inline]
            unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
                let Self { $field, .. };
                &raw mut (*ptr).$field
            }
        }

        impl $field_ty_name {
            pub fn new() -> Self {
                $field_ty_name
            }
        }
    };
}
