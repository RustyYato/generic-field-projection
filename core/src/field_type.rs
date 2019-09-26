use super::*;

pub struct FieldType<Parent: ?Sized, Type: ?Sized> {
    descriptor: FieldDescriptor<Parent, Type>,
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

impl<Parent: ?Sized, Type: ?Sized> FieldType<Parent, Type> {
    pub unsafe fn new_unchecked(descriptor: FieldDescriptor<Parent, Type>) -> Self {
        Self { descriptor }
    }
}
