//! Experimental support for pinnable pointers

use type_list::FieldList;
use typsy::{
    map::{Map, Mapped},
    zip::{Zip, Zipped},
};

use super::*;

impl<'a, F: Field, P> ProjectTo<PinToPin<F>> for Pin<P>
where
    P: PinnablePointer + ProjectTo<F>,
    P::Projection: core::ops::Deref<Target = F::Type>,
{
    type Projection = Pin<P::Projection>;

    fn project_to(self, pin_field: PinToPin<F>) -> Self::Projection {
        unsafe {
            let inner = Pin::into_inner_unchecked(self);

            Pin::new_unchecked(inner.project_to(pin_field.field()))
        }
    }
}

impl<'a, F: Field, P> ProjectTo<PinToPtr<F>> for Pin<P>
where
    // TODO: I don't know if `PinnablePointer` is strictly required here
    P: PinnablePointer + ProjectTo<F>,
{
    type Projection = P::Projection;

    fn project_to(self, pin_field: PinToPtr<F>) -> Self::Projection {
        unsafe {
            let inner = Pin::into_inner_unchecked(self);

            inner.project_to(pin_field.0)
        }
    }
}

pub struct MakePin;
pub struct MakePtr;

pub struct CreateTag;

pub struct BuildOutput;

typsy::call! {
    fn[F: Field](&mut self: CreateTag, _pin_to_pin: PinToPin<F>) -> MakePin {
        MakePin
    }

    fn[F: Field](&mut self: CreateTag, _pin_to_ptr: PinToPtr<F>) -> MakePtr {
        MakePtr
    }

    fn[T: Deref](&mut self: BuildOutput, arg: (MakePin, T)) -> Pin<T> {
        let (MakePin,  value) = arg;
        unsafe { Pin::new_unchecked(value) }
    }

    fn[T](&mut self: BuildOutput, arg: (MakePtr, T)) -> T {
        let (MakePtr, value) = arg;
        value
    }
}

impl<Parent, F: Copy + FieldList<Parent>, P> ProjectAll<Parent, F> for Pin<P>
where
    P: PinnablePointer + ProjectAll<Parent, F>,
    F: Map<CreateTag>,
    Mapped<F, CreateTag>: Zip<P::Projection>,
    Zipped<Mapped<F, CreateTag>, P::Projection>: Map<BuildOutput>,
{
    type Projection =
        Mapped<Zipped<Mapped<F, CreateTag>, P::Projection>, BuildOutput>;

    #[inline]
    fn project_all(self, field: F) -> Self::Projection {
        unsafe {
            let tags = field.map(CreateTag);

            let raw_output = Pin::into_inner_unchecked(self).project_all(field);

            tags.zip(raw_output).map(BuildOutput)
        }
    }
}
