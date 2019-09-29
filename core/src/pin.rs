use super::*;

/// A marker trait that specifies pointer safely project inside of a pin
/// 
/// # Safety
/// 
/// TODO: add safety docs
pub unsafe trait PinnablePointer: core::ops::Deref {}

pub type PPF<F> = PinProjectableField<F>;
pub type PTR<F> = PinToRef<F>;

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PinToRef<F: Field + ?Sized>(pub F);

/// A field-type which is pin-projectable
#[repr(transparent)]
#[derive(Copy, Clone)]
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

unsafe impl<F: ?Sized + Field> Field for PinToRef<F> {
    type Parent = F::Parent;
    type Type = F::Type;
    type Name = F::Name;

    #[inline]
    fn name(&self) -> Self::Name {
        F::name(&self.0)
    }
    
    #[inline]
    unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
        F::project_raw(&self.0, ptr)
    }
    
    #[inline]
    unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
        F::project_raw_mut(&self.0, ptr)
    }
}

impl<F: Field> PinProjectableField<F> {
    /// You must validate the safety notes of [`PinProjectable<F>`](trait.PinProjectable.html)
    #[inline]
    pub unsafe fn new_unchecked(field: F) -> Self {
        Self {
            field
        }
    }
    
    /// Convert to a dynamically dispatched field projection
    #[inline]
    pub fn as_dyn_pin(&self) -> PinProjectableField<&dyn Field<Parent = F::Parent, Type = F::Type, Name = F::Name>> {
        PinProjectableField {
            field: &self.field
        }
    }
    
    #[inline]
    pub(crate) fn field(self) -> F {
        self.field
    }
}

impl<F: Field + ?Sized> PinProjectableField<F> {
    /// You must validate the safety notes of [`PinProjectable<F>`](trait.PinProjectable.html)
    #[inline]
    pub unsafe fn from_ref_unchecked(field: &F) -> &Self {
        #[allow(clippy::transmute_ptr_to_ptr)]
        core::mem::transmute::<&F, &Self>(field)
    }

    #[inline]
    pub fn as_ref(&self) -> PinProjectableField<&F> {
        unsafe {
            PinProjectableField::new_unchecked(&self.field)
        }
    }
}

impl<F: Field> PinToRef<F> {
    /// You must validate the safety notes of [`PinProjectable<F>`](trait.PinProjectable.html)
    #[inline]
    pub fn new(field: F) -> Self {
        Self(field)
    }
    
    /// Convert to a dynamically dispatched field projection
    #[inline]
    pub fn as_dyn_pin(&self) -> PinToRef<&dyn Field<Parent = F::Parent, Type = F::Type, Name = F::Name>> {
        PinToRef(&self.0)
    }
}

impl<F: Field + ?Sized> PinToRef<F> {
    /// You must validate the safety notes of [`PinProjectable<F>`](trait.PinProjectable.html)
    #[inline]
    pub fn from_ref(field: &F) -> &Self {
        unsafe {
            #[allow(clippy::transmute_ptr_to_ptr)]
            core::mem::transmute::<&F, &Self>(field)
        }
    }

    #[inline]
    pub fn as_ref(&self) -> PinToRef<&F> {
        PinToRef(&self.0)
    }
}
