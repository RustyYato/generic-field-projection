use super::*;

/// A marker trait that specifies pointer safely project inside of a pin
///
/// # Safety
///
/// TODO: add safety docs
pub unsafe trait PinnablePointer: std::ops::Deref {}

/// Represents a field that can will be projected to a
/// pointer when projected from a `Pin`
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PinToPtr<F: Field + ?Sized>(pub F);

/// A field-type which is pin-projectable
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PinToPin<F: Field + ?Sized> {
    field: F,
}

unsafe impl<F: ?Sized + Field> Field for PinToPin<F> {
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

unsafe impl<F: ?Sized + Field> Field for PinToPtr<F> {
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

impl<F: Field> PinToPin<F> {
    /// You must validate the safety notes of [`PinProjectable<F>`](trait.PinProjectable.html)
    /// 
    /// # Safety
    /// 
    /// It must be safe to go from `Pin<Ptr<T>>` to `Pin<Ptr<Field>>` for any pinnable pointer
    #[inline]
    pub unsafe fn new_unchecked(field: F) -> Self {
        Self { field }
    }

    /// Convert to a dynamically dispatched field projection
    #[inline]
    pub fn as_dyn_pin(
        &self,
    ) -> PinToPin<&dyn Field<Parent = F::Parent, Type = F::Type, Name = F::Name>> {
        PinToPin { field: &self.field }
    }

    #[inline]
    pub(crate) fn field(self) -> F {
        self.field
    }
}

impl<F: Field + ?Sized> PinToPin<F> {
    /// You must validate the safety notes of [`PinProjectable<F>`](trait.PinProjectable.html)
    /// 
    /// # Safety
    /// 
    /// It must be safe to go from `Pin<Ptr<T>>` to `Pin<Ptr<Field>>` for any pinnable pointer
    #[inline]
    pub unsafe fn from_ref_unchecked(field: &F) -> &Self {
        #[allow(clippy::transmute_ptr_to_ptr)]
        std::mem::transmute::<&F, &Self>(field)
    }

    /// converts to a reference to the underlying field
    #[inline]
    pub fn as_ref(&self) -> PinToPin<&F> {
        unsafe { PinToPin::new_unchecked(&self.field) }
    }
}

impl<F: Field> PinToPtr<F> {
    /// You must validate the safety notes of [`PinProjectable<F>`](trait.PinProjectable.html)
    #[inline]
    pub fn new(field: F) -> Self {
        Self(field)
    }

    /// Convert to a dynamically dispatched field projection
    #[inline]
    pub fn as_dyn_pin(
        &self,
    ) -> PinToPtr<&dyn Field<Parent = F::Parent, Type = F::Type, Name = F::Name>> {
        PinToPtr(&self.0)
    }
}

impl<F: Field + ?Sized> PinToPtr<F> {
    /// You must validate the safety notes of [`PinProjectable<F>`](trait.PinProjectable.html)
    #[inline]
    pub fn from_ref(field: &F) -> &Self {
        unsafe {
            #[allow(clippy::transmute_ptr_to_ptr)]
            std::mem::transmute::<&F, &Self>(field)
        }
    }

    /// converts to a reference to the underlying field
    #[inline]
    pub fn as_ref(&self) -> PinToPtr<&F> {
        PinToPtr(&self.0)
    }
}
