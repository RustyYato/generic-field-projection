use super::*;

/// A marker trait that specifies pointer safely project inside of a pin
///
/// # Safety
///
/// FIXME: add safety docs
pub unsafe trait PinnablePointer: core::ops::Deref {}

/// Represents a field that can will be projected to a pointer when projected
/// from a `Pin`
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PinToPtr<F: Field>(pub F);

/// A field-type which is pin-projectable
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PinToPin<F: Field> {
    field: F,
}

unsafe impl<F: Field> Field for PinToPin<F> {
    type Parent = F::Parent;
    type Type = F::Type;

    #[inline]
    unsafe fn project_raw(
        &self,
        ptr: *const Self::Parent,
    ) -> *const Self::Type {
        F::project_raw(&self.field, ptr)
    }

    #[inline]
    unsafe fn project_raw_mut(
        &self,
        ptr: *mut Self::Parent,
    ) -> *mut Self::Type {
        F::project_raw_mut(&self.field, ptr)
    }
}

unsafe impl<F: Field> Field for PinToPtr<F> {
    type Parent = F::Parent;
    type Type = F::Type;

    #[inline]
    unsafe fn project_raw(
        &self,
        ptr: *const Self::Parent,
    ) -> *const Self::Type {
        F::project_raw(&self.0, ptr)
    }

    #[inline]
    unsafe fn project_raw_mut(
        &self,
        ptr: *mut Self::Parent,
    ) -> *mut Self::Type {
        F::project_raw_mut(&self.0, ptr)
    }
}

impl<F: Field> PinToPin<F> {
    /// You must validate the safety notes of
    /// [`PinProjectable<F>`](trait.PinProjectable.html)
    ///
    /// # Safety
    ///
    /// It must be safe to go from `Pin<Ptr<T>>` to `Pin<Ptr<Field>>` for any
    /// pinnable pointer
    #[inline]
    pub unsafe fn new_unchecked(field: F) -> Self {
        Self {
            field,
        }
    }

    /// Get the wrapped field
    #[inline]
    pub fn field(self) -> F {
        self.field
    }

    /// converts to a reference to the underlying field
    #[inline]
    pub fn as_ref(&self) -> PinToPin<&F> {
        unsafe { PinToPin::new_unchecked(&self.field) }
    }

    /// Convert to a dynamic field that can project pinned types to pinned fields
    pub fn pin_dynamic(&self) -> PinToPin<crate::Dynamic<F::Parent, F::Type>> {
        // # Safety
        //
        // * It is to go from `Pin<Ptr<T>>` to `Pin<Ptr<Field>>` for any
        //   pinnable pointer by virtue of `Self` being a `PinToPin`
        //   and `Field::dynamic` returning the *same* field
        unsafe { PinToPin::new_unchecked(self.field.dynamic()) }
    }
}

impl<F: Field> PinToPtr<F> {
    /// You must validate the safety notes of
    /// [`PinProjectable<F>`](trait.PinProjectable.html)
    #[inline]
    pub fn new(field: F) -> Self {
        Self(field)
    }

    /// converts to a reference to the underlying field
    #[inline]
    pub fn as_ref(&self) -> PinToPtr<&F> {
        PinToPtr(&self.0)
    }
}
