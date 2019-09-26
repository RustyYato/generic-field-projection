
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

#[macro_export]
macro_rules! field {
    ($field_ty_name:ident ($parent:ty => $field_ty:ty), $field:ident, $value:expr) => {
        struct $field_ty_name;

        unsafe impl Field for $field_ty_name {
            type Parent = $parent;
            type Type = $field_ty;
            
            #[deny(safe_packed_borrows)]
            fn field_descriptor(&self) -> FieldDescriptor<Self::Parent, Self::Type> {
                let mut value = $value;
                let parent: *mut $parent = &mut value;
                let field: *mut $field_ty = &mut value.$field;

                unsafe {
                    $crate::FieldDescriptor::from_pointers(parent, field)
                }
            }
        }

        impl $field_ty_name {
            pub fn new() -> Self {
                $field_ty_name
            }
        }
    };
}
