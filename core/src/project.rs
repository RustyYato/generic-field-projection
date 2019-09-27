use super::*;

use core::pin::Pin;

unsafe impl<F: ?Sized> PinnablePointer for &F {}
impl<'a, F: Field + ?Sized> ProjectTo<F> for &'a F::Parent
where F::Parent: 'a, F::Type: 'a {
    type Projection = &'a F::Type;

    fn project_to(self, field: &F) -> Self::Projection {
        unsafe {
            &*field.project_raw(self)
        }
    }
}

unsafe impl<F: ?Sized> PinnablePointer for &mut F {}
impl<'a, F: Field + ?Sized> ProjectTo<F> for &'a mut F::Parent
where F::Parent: 'a, F::Type: 'a {
    type Projection = &'a mut F::Type;

    fn project_to(self, field: &F) -> Self::Projection {
        unsafe {
            &mut *field.project_raw_mut(self)
        }
    }
}

impl<'a, F: Field + ?Sized, P: PinnablePointer + ProjectTo<F>> ProjectTo<PinProjectableField<F>> for Pin<P>
where P::Projection: core::ops::Deref<Target = F::Type> {
    type Projection = Pin<P::Projection>;

    fn project_to(self, pin_field: &PinProjectableField<F>) -> Self::Projection {
        unsafe {
            let inner = Pin::into_inner_unchecked(self);

            Pin::new_unchecked(inner.project_to(pin_field.field()))
        }
    }
}
