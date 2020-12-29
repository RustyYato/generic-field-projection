use crate::{Field, UncheckedInverseProjectTo, UncheckedProjectTo};

use core::ptr::NonNull;

impl<F: Field> UncheckedProjectTo<F> for *const F::Parent {
    type Projection = *const F::Type;

    unsafe fn project_to(self, field: F) -> Self::Projection {
        field.project_raw(self)
    }
}

impl<F: Field> UncheckedInverseProjectTo<F> for *const F::Type {
    type Projection = *const F::Parent;

    unsafe fn inverse_project_to(self, field: F) -> Self::Projection {
        field.inverse_project_raw(self)
    }
}

impl<F: Field> UncheckedProjectTo<F> for *mut F::Parent {
    type Projection = *mut F::Type;

    unsafe fn project_to(self, field: F) -> Self::Projection {
        field.project_raw_mut(self)
    }
}

impl<F: Field> UncheckedInverseProjectTo<F> for *mut F::Type {
    type Projection = *mut F::Parent;

    unsafe fn inverse_project_to(self, field: F) -> Self::Projection {
        field.inverse_project_raw_mut(self)
    }
}

impl<F: Field> UncheckedProjectTo<F> for NonNull<F::Parent> {
    type Projection = NonNull<F::Type>;

    unsafe fn project_to(self, field: F) -> Self::Projection {
        NonNull::new_unchecked(field.project_raw_mut(self.as_ptr()))
    }
}

impl<F: Field> UncheckedInverseProjectTo<F> for NonNull<F::Type> {
    type Projection = NonNull<F::Parent>;

    unsafe fn inverse_project_to(self, field: F) -> Self::Projection {
        NonNull::new_unchecked(field.inverse_project_raw_mut(self.as_ptr()))
    }
}

impl<F: Field, T: UncheckedProjectTo<F>> UncheckedProjectTo<F> for Option<T> {
    type Projection = Option<T::Projection>;

    unsafe fn project_to(self, field: F) -> Self::Projection {
        match self {
            Some(value) => Some(value.project_to(field)),
            None => None,
        }
    }
}

impl<F: Field, T: UncheckedInverseProjectTo<F>> UncheckedInverseProjectTo<F>
    for Option<T>
{
    type Projection = Option<T::Projection>;

    unsafe fn inverse_project_to(self, field: F) -> Self::Projection {
        match self {
            Some(value) => Some(value.inverse_project_to(field)),
            None => None,
        }
    }
}
