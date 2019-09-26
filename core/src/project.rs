use super::*;

use std::pin::Pin;

unsafe impl<F: ?Sized> PinnablePointer for &F {}
impl<'a, F: Field> ProjectTo<F> for &'a F::Parent
where F::Parent: 'a, F::Type: 'a {
    type Projection = &'a F::Type;

    fn project_to(self, _: F) -> Self::Projection {
        unsafe {
            &*F::FIELD_DESCRIPTOR.project_raw_unchecked(self)
        }
    }
}

unsafe impl<F: ?Sized> PinnablePointer for &mut F {}
impl<'a, F: Field> ProjectTo<F> for &'a mut F::Parent
where F::Parent: 'a, F::Type: 'a {
    type Projection = &'a mut F::Type;

    fn project_to(self, _: F) -> Self::Projection {
        unsafe {
            &mut *F::FIELD_DESCRIPTOR.project_raw_mut_unchecked(self)
        }
    }
}

impl<'a, F: Field + PinProjectable, P: PinnablePointer + ProjectTo<F>> ProjectTo<F> for Pin<P>
where P::Projection: std::ops::Deref<Target = F::Type> {
    type Projection = Pin<P::Projection>;

    fn project_to(self, field: F) -> Self::Projection {
        unsafe {
            let inner = Pin::into_inner_unchecked(self);

            Pin::new_unchecked(inner.project_to(field))
        }
    }
}
