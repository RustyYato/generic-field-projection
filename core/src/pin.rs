use super::*;

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

unsafe impl<F: ?Sized + Field> Field for PinProjectableField<F> {
    type Parent = F::Parent;
    type Type = F::Type;
    type Name = F::Name;

    #[inline]
    fn name(&self) -> Self::Name {
        F::name(&self.field)
    }
    
    #[inline]
    unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
        F::project_raw(&self.field, ptr)
    }
    
    #[inline]
    unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
        F::project_raw_mut(&self.field, ptr)
    }
}

impl<F: Field> PinProjectableField<F> {
    /// You must validate the safety notes of [`PinProjectable<F>`](trait.PinProjectable.html)
    pub unsafe fn new_unchecked(field: F) -> Self {
        Self {
            field
        }
    }
    
    /// Convert to a dynamically dispatched field projection
    pub fn as_dyn_pin(&self) -> PinProjectableField<&dyn Field<Parent = F::Parent, Type = F::Type, Name = F::Name>> {
        PinProjectableField {
            field: &self.field
        }
    }
}

impl<F: Field + ?Sized> PinProjectableField<F> {
    pub(crate) fn field(&self) -> &F {
        &self.field
    }

    /// You must validate the safety notes of [`PinProjectable<F>`](trait.PinProjectable.html)
    pub unsafe fn from_ref_unchecked(field: &F) -> &Self {
        #[allow(clippy::transmute_ptr_to_ptr)]
        core::mem::transmute::<&F, &Self>(field)
    }
}
