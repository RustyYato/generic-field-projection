use super::*;

/// A marker trait that specifies that the given field is safe to pin-project to
/// 
/// # Safety
/// 
/// For the field represented by the field type `F`, three things need to be ensured:
/// * If the struct implements `Drop`, the drop method is not allowed to move the value of the field.
/// * If the struct wants to implement `Unpin`, it has to do so conditionally: The struct can only implement `Unpin` if the field's type is `Unpin`.
/// * The struct must not be #[repr(packed)].
/// 
/// See [pinning is structural for field](https://doc.rust-lang.org/std/pin/#pinning-is-structural-for-field) for more information
pub unsafe trait PinProjectable<F: Field<Parent = Self> + ?Sized> {}

/// A marker trait that specifies pointer safely project inside of a pin
/// 
/// # Safety
/// 
/// TODO: add safety docs
pub unsafe trait PinnablePointer: core::ops::Deref {}

/// A field-type which is pin-projectable
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
    /// You must validate the safety notes of [PinProjectable](trait.PinProjectable.html)
    pub unsafe fn new_unchecked(field: F) -> Self {
        Self {
            field
        }
    }
    
    /// Create a new pin-projectable field, validated by [`PinProjectable<F>`](trait.PinProjectable.html) trait
    pub fn new(field: F) -> Self where F::Parent: PinProjectable<F> {
        Self {
            field
        }
    }

    /// Convert to a dynamically dispatched field projection
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

    /// You must validate the safety notes of [PinProjectable](trait.PinProjectable.html)
    pub unsafe fn from_ref_new_unchecked(field: &F) -> &Self {
        #[allow(clippy::transmute_ptr_to_ptr)]
        core::mem::transmute::<&F, &Self>(field)
    }
    
    /// Create a new pin-projectable field, validated by [`PinProjectable<F>`](trait.PinProjectable.html) trait
    pub fn from_ref(field: &F) -> &Self where F::Parent: PinProjectable<F> {
        unsafe {
            #[allow(clippy::transmute_ptr_to_ptr)]
            core::mem::transmute::<&F, &Self>(field)
        }
    }

    /// Convert to a dynamically dispatched field projection
    pub fn as_dyn_pin(&self) -> PinProjectableField<FieldType<F::Parent, F::Type>> {
        unsafe {
            let field_type = FieldType { descriptor: self.field.field_descriptor() };
            PinProjectableField::new_unchecked(field_type)
        }
    }
}
