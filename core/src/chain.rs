use super::*;

/// Chains two projections together
#[derive(Clone, Copy)]
pub struct Chain<A, B> {
    a: A,
    b: B
}

impl<A, B> Chain<A, B> {
    #[inline]
    pub const fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

unsafe impl<A: Field, B: Field<Parent = A::Type>> Field for Chain<A, B> {
    type Parent = A::Parent;
    type Type = B::Type;
    type Name = core::iter::Chain<A::Name, B::Name>;

    #[inline]
    fn name(&self) -> Self::Name {
        self.a.name().chain(self.b.name())
    }
    
    #[inline]
    unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
        let ptr = self.a.project_raw(ptr);
        self.b.project_raw(ptr)
    }
    
    #[inline]
    unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
        let ptr = self.a.project_raw_mut(ptr);
        self.b.project_raw_mut(ptr)
    }
}