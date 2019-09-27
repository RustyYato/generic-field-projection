use crate::set::tuple::TypeFunction;
use core::marker::PhantomData;
use core::pin::Pin;

pub struct Enumerate(u64);

impl Default for Enumerate {
    fn default() -> Self {
        Self(0)
    }
}

impl Enumerate {
    pub fn start_at(start: u64) -> Self {
        Self(start)
    }
}

impl<T> TypeFunction<T> for Enumerate {
    type Output = (u64, T);

    fn call(&mut self, input: T) -> Self::Output {
        let count = self.0;
        self.0 += 1;
        (count, input)
    }
}

pub struct PtrToRef<'a>(PhantomData<&'a ()>);

impl<'a> PtrToRef<'a> {
    pub(crate) unsafe fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, T: ?Sized + 'a> TypeFunction<*const T> for PtrToRef<'a> {
    type Output = &'a T;

    fn call(&mut self, input: *const T) -> Self::Output {
        unsafe { &*input }
    }
}

pub struct PtrToRefMut<'a>(PhantomData<&'a ()>);

impl<'a> PtrToRefMut<'a> {
    pub(crate) unsafe fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'a, T: ?Sized + 'a> TypeFunction<*mut T> for PtrToRefMut<'a> {
    type Output = &'a mut T;

    fn call(&mut self, input: *mut T) -> Self::Output {
        unsafe { &mut *input }
    }
}

pub struct PinNewUnchecked(());

impl PinNewUnchecked {
    pub(crate) unsafe fn new() -> Self {
        Self(())
    }
}

impl<P: core::ops::Deref> TypeFunction<P> for PinNewUnchecked {
    type Output = Pin<P>;

    fn call(&mut self, input: P) -> Self::Output {
        unsafe { Pin::new_unchecked(input) }
    }
}
