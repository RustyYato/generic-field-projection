use super::*;

/// Represents a dynamic `FieldType` that can point to any field of that satisfy the given types
pub struct FieldType<Parent: ?Sized, Type: ?Sized> {
    /// The descriptor for the field type
    pub descriptor: FieldDescriptor<Parent, Type>
}

unsafe impl<Parent: ?Sized, Type: ?Sized> Field for FieldType<Parent, Type> {
    type Parent = Parent;
    type Type = Type;

    fn field_descriptor(&self) -> FieldDescriptor<Self::Parent, Self::Type> {
        self.descriptor
    }

    fn into_dyn(self) -> FieldType<Self::Parent, Self::Type> {
        self
    }
}
