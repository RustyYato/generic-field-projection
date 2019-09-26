
/// Create a new field descriptor for the given type
#[macro_export]
macro_rules! field_descriptor {
    ($field:ident, $value:expr) => {{
        let mut value = $value;
        let parent: *mut _ = &mut value;
        let field: *mut _ = &mut value.$field;
        unsafe {
            $crate::FieldDescriptor::from_pointers(parent, field)
        }
    }};
}

/// Create a new dynamic field type for the given field
#[macro_export]
macro_rules! field_type {
    ($field:ident, $value:expr) => {{
        let mut value = $value;
        let parent: *mut _ = &mut value;
        let field: *mut _ = &mut value.$field;
        unsafe {
            $crate::FieldType::new_unchecked($crate::FieldDescriptor::from_pointers(parent, field))
        }
    }};
}

/// Create a new compile-time field type for the given field
#[macro_export]
macro_rules! field {
    ($field_ty_name:ident ($parent:ty => $field_ty:ty), $field:ident, $value:expr) => {
        struct $field_ty_name;

        unsafe impl Field for $field_ty_name {
            type Parent = $parent;
            type Type = $field_ty;
            
            #[deny(safe_packed_borrows)]
            fn field_descriptor(&self) -> FieldDescriptor<Self::Parent, Self::Type> {
                $crate::field_descriptor!($field, $value)
            }
        }

        impl $field_ty_name {
            pub fn new() -> Self {
                $field_ty_name
            }
        }
    };
}
