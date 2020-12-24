use crate::Field;

/// A runtime offset based `Field`. This is a more efficient version
/// of `dyn Field<Parent = P, Type = T, Name = N>`.
///
/// Generated from [`Field::dynamic`]
pub struct Dynamic<P, T> {
    offset: usize,
    _mark:  crate::derive::Invariant<(*const T, *const P)>,
}

impl<P, T> Copy for Dynamic<P, T> {
}
impl<P, T> Clone for Dynamic<P, T> {
    fn clone(&self) -> Self {
        Self {
            offset: self.offset,
            _mark:  crate::derive::Invariant::INIT,
        }
    }
}

impl<P, T> Dynamic<P, T> {
    /// Create a dynamic field from an offset and a name iterator.
    ///
    /// # Safety
    ///
    /// * `offset` - must be the offset in *bytes* from the start of `P`
    ///              to a field/sub-field of type `T`
    /// * `name`   - The fully qualified name of the field
    pub unsafe fn from_offset(offset: usize) -> Self {
        Self {
            offset,
            _mark: crate::derive::Invariant::INIT,
        }
    }
}

impl<P, T> Dynamic<P, T> {
    /// Get the offset
    pub fn offset(&self) -> usize {
        self.offset
    }
}

unsafe impl<P, T: Field> Field for Dynamic<P, T> {
    type Parent = P;
    type Type = T;

    unsafe fn project_raw(
        &self,
        ptr: *const Self::Parent,
    ) -> *const Self::Type {
        ptr.cast::<u8>().add(self.offset).cast()
    }

    unsafe fn project_raw_mut(
        &self,
        ptr: *mut Self::Parent,
    ) -> *mut Self::Type {
        ptr.cast::<u8>().add(self.offset).cast()
    }

    fn field_offset(&self) -> usize {
        self.offset
    }
}
