use core::marker::PhantomData;

use crate::Field;

struct Invariant<T: ?Sized>(fn() -> *mut T);

/// A runtime offset based `Field`. This is a more efficient version
/// of `dyn Field<Parent = P, Type = T, Name = N>`.
///
/// Generated from [`Field::dynamic`]
pub struct Dynamic<P: ?Sized, T, N> {
    offset: usize,
    name:   N,
    mark:   PhantomData<Invariant<(T, P)>>,
}

impl<P: ?Sized, T, N: Clone> Clone for Dynamic<P, T, N> {
    fn clone(&self) -> Self {
        Self {
            offset: self.offset,
            name:   self.name.clone(),
            mark:   PhantomData,
        }
    }
}

impl<P: ?Sized, T, N: Iterator<Item = &'static str> + Clone> Dynamic<P, T, N> {
    /// Create a dynamic field from an offset and a name iterator.
    ///
    /// # Safety
    ///
    /// * `offset` - must be the offset in *bytes* from the start of `P`
    ///              to a field/sub-field of type `T`
    /// * `name`   - The fully qualified name of the field
    pub unsafe fn from_raw_parts(offset: usize, name: N) -> Self {
        Self {
            offset,
            name,
            mark: PhantomData,
        }
    }
}

impl<P: ?Sized, T, N> Dynamic<P, T, N> {
    /// Get the offset
    pub fn offset(&self) -> usize {
        self.offset
    }
}

unsafe impl<P: ?Sized, T: Field, N: Iterator<Item = &'static str> + Clone> Field
    for Dynamic<P, T, N>
{
    type Name = N;
    type Parent = P;
    type Type = T;

    fn name(&self) -> Self::Name {
        self.name.clone()
    }

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

    fn field_offset(&self) -> usize
    where
        Self::Parent: Sized,
    {
        self.offset
    }
}
