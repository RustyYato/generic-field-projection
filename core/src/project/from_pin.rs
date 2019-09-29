use super::*;

impl<'a, F: Field, P: PinnablePointer + ProjectTo<F>> ProjectTo<PinToPin<F>> for Pin<P>
where P::Projection: core::ops::Deref<Target = F::Type> {
    type Projection = Pin<P::Projection>;

    fn project_to(self, pin_field: PinToPin<F>) -> Self::Projection {
        unsafe {
            let inner = Pin::into_inner_unchecked(self);

            Pin::new_unchecked(inner.project_to(pin_field.field()))
        }
    }
}

impl<'a, F: Field, P: PinnablePointer + ProjectTo<F>> ProjectTo<PinToPtr<F>> for Pin<P> {
    type Projection = P::Projection;

    fn project_to(self, pin_field: PinToPtr<F>) -> Self::Projection {
        unsafe {
            let inner = Pin::into_inner_unchecked(self);

            inner.project_to(pin_field.0)
        }
    }
}

pub struct MakePin;
pub struct MakeRef;

pub struct CheckMake;

impl<F: Field> TypeFunction<PinToPin<F>> for CheckMake {
    type Output = MakePin;

    #[inline]
    fn call(&mut self, _: PinToPin<F>) -> Self::Output { MakePin }
}

impl<F: Field> TypeFunction<PinToPtr<F>> for CheckMake {
    type Output = MakeRef;

    #[inline]
    fn call(&mut self, _: PinToPtr<F>) -> Self::Output { MakeRef }
}

pub struct PinCombine;

impl<T: Deref> TypeFunction<(MakePin, T)> for PinCombine {
    type Output = Pin<T>;

    #[inline]
    fn call(&mut self, (MakePin, value): (MakePin, T)) -> Self::Output {
        unsafe { Pin::new_unchecked(value) }
    }
}

impl<T> TypeFunction<(MakeRef, T)> for PinCombine {
    type Output = T;

    #[inline]
    fn call(&mut self, (MakeRef, value): (MakeRef, T)) -> Self::Output {
        value
    }
}

impl<F: Copy + FieldSet, P: ProjectToSet<F> + Deref> ProjectToSet<F> for Pin<P>
where
    F: TupleMap<CheckMake>,
    TMap<F, CheckMake>: TupleZip<P::Projection, PinCombine>
{
    type Projection = TZip<TMap<F, CheckMake>, P::Projection, PinCombine>;

    #[inline]
    fn project_set_to(self, field: F) -> Self::Projection {
        let check_make = field.tup_map(CheckMake);
        unsafe {
            let project = Pin::into_inner_unchecked(self)
                .project_set_to(field);
            
            check_make.tup_zip(project, PinCombine)
        }
    }
}
