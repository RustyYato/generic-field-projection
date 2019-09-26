use super::*;

pub unsafe trait PinProjectable<F: Field<Parent = Self> + ?Sized> {}
pub unsafe trait PinnablePointer: std::ops::Deref {}

#[repr(transparent)]
pub struct PinProjectableField<F: Field + ?Sized> {
    field: F
}

unsafe impl<F: Field + ?Sized> Field for PinProjectableField<F> {
    type Parent = F::Parent;
    type Type = F::Type;

    fn field_descriptor(&self) -> FieldDescriptor<Self::Parent, Self::Type> {
        self.field.field_descriptor()
    }
}

impl<F: Field> PinProjectableField<F> {
    pub unsafe fn new_unchecked(field: F) -> Self {
        Self {
            field
        }
    }
    
    pub fn new(field: F) -> Self where F::Parent: PinProjectable<F> {
        Self {
            field
        }
    }

    pub fn new_unpin(field: F) -> Self where F::Parent: Unpin {
        Self {
            field
        }
    }

    pub fn into_dyn_pin(self) -> PinProjectableField<FieldType<F::Parent, F::Type>> {
        unsafe {
            PinProjectableField::new_unchecked(self.field.into_dyn())
        }
    }
}

impl<F: Field + ?Sized> PinProjectableField<F> {
    pub(crate) fn field(&self) -> &F {
        &self.field
    }

    pub unsafe fn from_ref_new_unchecked(field: &F) -> &Self {
        #[allow(clippy::transmute_ptr_to_ptr)]
        std::mem::transmute::<&F, &Self>(field)
    }
    
    pub fn from_ref(field: &F) -> &Self where F::Parent: PinProjectable<F> {
        unsafe {
            #[allow(clippy::transmute_ptr_to_ptr)]
            std::mem::transmute::<&F, &Self>(field)
        }
    }

    pub fn from_ref_unpin(field: &F) -> &Self where F::Parent: Unpin {
        unsafe {
            #[allow(clippy::transmute_ptr_to_ptr)]
            std::mem::transmute::<&F, &Self>(field)
        }
    }

    pub fn as_dyn_pin(&self) -> PinProjectableField<FieldType<F::Parent, F::Type>> {
        unsafe {
            let field_type = FieldType::new_unchecked(self.field.field_descriptor());
            PinProjectableField::new_unchecked(field_type)
        }
    }
}
