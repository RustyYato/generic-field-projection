use crate::Field;

pub(crate) mod any;
pub(crate) mod call;
pub(crate) mod fold;
pub(crate) mod map;
pub(crate) mod zip;

use call::{CallMut, CallOnce};

#[derive(Clone, Copy)]
pub struct Cons<T, R>(pub T, pub R);
#[derive(Clone, Copy)]
pub struct Nil;

/// Represents a list of fields
pub trait FieldList<Parent: ?Sized> {
    /// The list of types for each of the fields (each element is `*const _`)
    type Type;

    /// The list of types for each of the fields (each element is `*mut _`)
    type TypeMut;

    /// projects the raw pointer from the `Parent` type to the field `Type`
    ///
    /// # Safety
    ///
    /// * `ptr` must point to a valid, initialized allocation of `Parent`
    /// * the projection is not safe to write to
    unsafe fn project_raw(&self, ptr: *const Parent) -> Self::Type;

    /// projects the raw pointer from the `Parent` type to the field `Type`
    ///
    /// # Safety
    ///
    /// * `ptr` must point to a valid, initialized allocation of `Parent`
    /// * the projection is not safe to write to
    unsafe fn project_raw_mut(&self, ptr: *mut Parent) -> Self::TypeMut;
}

impl<Parent: ?Sized> FieldList<Parent> for Nil {
    type Type = Nil;
    type TypeMut = Nil;

    unsafe fn project_raw(&self, _: *const Parent) -> Self::Type {
        Nil
    }

    unsafe fn project_raw_mut(&self, _: *mut Parent) -> Self::TypeMut {
        Nil
    }
}

impl<F: Field, R: FieldList<F::Parent>> FieldList<F::Parent> for Cons<F, R> {
    type Type = Cons<*const F::Type, R::Type>;
    type TypeMut = Cons<*mut F::Type, R::TypeMut>;

    unsafe fn project_raw(&self, ptr: *const F::Parent) -> Self::Type {
        Cons(self.0.project_raw(ptr), self.1.project_raw(ptr))
    }

    unsafe fn project_raw_mut(&self, ptr: *mut F::Parent) -> Self::TypeMut {
        Cons(self.0.project_raw_mut(ptr), self.1.project_raw_mut(ptr))
    }
}

/// Creates a type-level cons list
#[macro_export]
macro_rules! List {
    () => { $crate::type_list::Nil };
    ($t:ty $(, $rest:ty)* $(,)?) => { $crate::type_list::Cons<$t, $crate::list!($($rest),*)> };
}

/// Creates a type-level cons list
#[macro_export]
macro_rules! list {
    () => { $crate::type_list::Nil };
    ($t:expr $(, $rest:expr)* $(,)?) => { $crate::type_list::Cons($t, $crate::list!($($rest),*)) };
}

/// Creates a type-level cons list
#[macro_export]
macro_rules! list_pat {
    () => { $crate::type_list::Nil };
    ($t:pat $(, $rest:pat)* $(,)?) => { $crate::type_list::Cons($t, $crate::list_pat!($($rest),*)) };
}
