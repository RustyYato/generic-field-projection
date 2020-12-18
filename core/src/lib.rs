#![feature(dropck_eyepatch, raw_ref_op)]
#![allow(clippy::needless_doctest_main)]
#![forbid(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

//! This crate provides a generic interface to project to fields, think of it as
//! an extended version of `Deref` that handles all pointer types equally.

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

mod chain;
#[doc(hidden)]
pub mod macros;
mod pin;
mod project;

#[doc(hidden)]
pub mod set;

pub use self::{chain::*, pin::*, set::FieldSet};
pub use gfp_derive::Field;

#[doc(hidden)]
#[macro_export]
macro_rules! ptr_project {
    ($mut:ident $ptr:ident $field:ident) => {
        &raw $mut (*$ptr).$field
    }
}

#[doc(hidden)]
pub mod derive {
    use core::cell::UnsafeCell;
    pub use core::{
        iter::{once, Once},
        marker::PhantomData,
    };

    pub struct Invariant<T: ?Sized>(pub PhantomData<UnsafeCell<T>>);

    unsafe impl<T: ?Sized> Send for Invariant<T> {
    }
    unsafe impl<T: ?Sized> Sync for Invariant<T> {
    }

    impl<T: ?Sized> Clone for Invariant<T> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<T: ?Sized> Copy for Invariant<T> {
    }
}

/// Projects a type to the given field
pub trait ProjectTo<F: Field> {
    /// The projection of the type, can be used to directly access the field
    type Projection: core::ops::Deref<Target = F::Type>;

    /// projects to the given field
    fn project_to(self, field: F) -> Self::Projection;
}

/// Projects a type to the given field
pub trait ProjectToSet<F: FieldSet> {
    /// The projection of the type, can be used to directly access the field
    type Projection;

    /// projects to the given field
    fn project_set_to(self, field: F) -> Self::Projection;
}

/// Represents a field of some `Parent` type.
///
/// You can derive this trait for all fields of `Parent` by using:
///
/// ```rust
/// #![feature(raw_ref_op)]
/// # mod __ {
/// use gfp_core::Field;
///
/// #[derive(Field)]
/// struct Foo {
///     bar: u32,
///     val: String
/// }
/// # }
/// ```
///
/// # Safety
///
/// * `Parent` must represent the type where the field came from
/// * `Type` must represent the type of the field itself
/// * `project_raw` and `project_raw_mut` must only access the given field
/// * `name` must return an iterator that yields all of the fields from `Parent`
///   to the given field,
/// * You must not override `field_offset`, `inverse_project_raw`,
///   or `inverse_project_raw_mut`
/// ex.
///
/// ```rust
/// struct Foo {
///     bar: Bar
/// }
///
/// struct Bar {
///     tap: Tap
/// }
///
/// struct Tap {
///     val: u32
/// }
/// ```
///
/// if want to get field `val` from `Foo`, you must implement field like so,
///
/// ```rust
/// #![feature(raw_ref_op)]
/// # struct Foo {
/// #     bar: Bar
/// # }
/// #
/// # struct Bar {
/// #     tap: Tap
/// # }
/// #
/// # struct Tap {
/// #     val: u32
/// # }
/// use gfp_core::Field;
///
/// struct FieldVal;
///
/// unsafe impl Field for FieldVal {
///     type Parent = Foo;
///     type Type = u32;
///     type Name = std::iter::Copied<std::slice::Iter<'static, &'static str>>;
///
///     fn name(&self) -> Self::Name {
///         ["bar", "tap", "val"].iter().copied()
///     }
///
///     unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
///         &raw const (*ptr).bar.tap.val
///     }
///
///     unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
///         &raw mut (*ptr).bar.tap.val
///     }
/// }
/// ```
///
/// But it is better to just derive `Field` on the types that you need to, and
/// then use the [`chain`](trait.Fields.html#method.chain) combinator to project
/// to the fields of fields
///
/// ```rust
/// #![feature(raw_ref_op)]
/// # mod main {
/// use gfp_core::Field;
///
/// #[derive(Field)]
/// struct Foo {
///     bar: Bar
/// }
///
/// #[derive(Field)]
/// struct Bar {
///     tap: Tap
/// }
///
/// #[derive(Field)]
/// struct Tap {
///     val: u32
/// }
///
/// fn main() {
///     let foo_to_val = Foo::fields().bar.chain(
///         Bar::fields().tap
///     ).chain(
///         Tap::fields().val
///     );
///
///     // or if you are going to use that projection a lot
///
///     use gfp_core::Chain;
///
///     const FOO_TO_VAL: Chain<Chain<Foo_fields::bar::<Foo>, Bar_fields::tap::<Bar>>, Tap_fields::val::<Tap>> = Chain::new(
///         Chain::new(
///             Foo::FIELDS.bar,
///             Bar::FIELDS.tap,
///         ),
///         Tap::FIELDS.val,
///     );
/// }
/// # }
/// ```
pub unsafe trait Field {
    /// The type that the field comes from
    type Parent: ?Sized;

    /// The type of the field itself
    type Type: ?Sized;

    /// An iterator that returns the fuully qualified name of the field
    type Name: Iterator<Item = &'static str>;

    /// An iterator that returns the fully qualified name of the field
    ///
    /// This must be unique for each field of the given `Parent` type
    fn name(&self) -> Self::Name;

    /// projects the raw pointer from the `Parent` type to the field `Type`
    ///
    /// # Safety
    ///
    /// * `ptr` must point to a valid allocation of `Parent`
    /// * the projection is not safe to write to
    unsafe fn project_raw(&self, ptr: *const Self::Parent)
    -> *const Self::Type;

    /// projects the raw pointer from the `Parent` type to the field `Type`
    ///
    /// # Safety
    ///
    /// `ptr` must point to a valid allocation of `Parent`
    unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent)
    -> *mut Self::Type;

    /// gets the offset of the field from the base pointer of `Parent`
    fn field_offset(&self) -> usize
    where
        Self::Parent: Sized,
    {
        use core::mem::MaybeUninit;

        unsafe {
            let parent = MaybeUninit::<Self::Parent>::uninit();
            let parent_ptr = parent.as_ptr();

            // Safety
            // * `parent_ptr` does point to a valid allocation of `Parent`
            //      * it just happens to be uninitialized
            // * the projection is not safe to write to
            //      * we never write to `field_ptr`
            let field_ptr = self.project_raw(parent_ptr);

            // Safety
            // * `parent_ptr` and `field_ptr` are guaranteed to be in the same
            //      allocation because of the safety requirements on `Field`
            let offset =
                field_ptr.cast::<u8>().offset_from(parent_ptr.cast::<u8>());

            // The offset will always be positive, because fields must
            // come after their parents
            offset as usize
        }
    }

    /// projects the raw pointer from the field `Type` to the `Parent` type
    ///
    /// # Safety
    ///
    /// * `ptr` must point into a valid allocation of `Type`
    /// * `ptr` must point to a field of `Parent` with the type `Type`
    unsafe fn inverse_project_raw(
        &self,
        ptr: *const Self::Type,
    ) -> *const Self::Parent
    where
        Self::Parent: Sized,
    {
        // Safety
        // * `ptr` is guaranteed to be a pointer to a field of `Parent`
        // * `field_offset` is guarateed to give the correct offset of the field
        ptr.cast::<u8>().sub(self.field_offset()).cast()
    }

    /// projects the raw pointer from the field `Type` to the `Parent` type
    ///
    /// # Safety
    ///
    /// * `ptr` must point into a valid allocation of `Type`
    /// * `ptr` must point to a field of `Parent` with the type `Type`
    unsafe fn inverse_project_raw_mut(
        &self,
        ptr: *mut Self::Type,
    ) -> *mut Self::Parent
    where
        Self::Parent: Sized,
    {
        // Safety
        // * `ptr` is guaranteed to be a pointer to a field of `Parent`
        // * `field_offset` is guarateed to give the correct offset of the field
        ptr.cast::<u8>().sub(self.field_offset()).cast()
    }

    /// projects the raw pointer from the field `Type` to the `Parent` type
    fn wrapping_project_raw(
        &self,
        ptr: *const Self::Type,
    ) -> *const Self::Parent
    where
        Self::Parent: Sized,
    {
        ptr.cast::<u8>().wrapping_add(self.field_offset()).cast()
    }

    /// projects the raw pointer from the field `Type` to the `Parent` type
    fn wrapping_project_raw_mut(
        &self,
        ptr: *mut Self::Type,
    ) -> *mut Self::Parent
    where
        Self::Parent: Sized,
    {
        ptr.cast::<u8>().wrapping_add(self.field_offset()).cast()
    }

    /// projects the raw pointer from the field `Type` to the `Parent` type
    fn wrapping_inverse_project_raw(
        &self,
        ptr: *const Self::Type,
    ) -> *const Self::Parent
    where
        Self::Parent: Sized,
    {
        ptr.cast::<u8>().wrapping_sub(self.field_offset()).cast()
    }

    /// projects the raw pointer from the field `Type` to the `Parent` type
    fn wrapping_inverse_project_raw_mut(
        &self,
        ptr: *mut Self::Type,
    ) -> *mut Self::Parent
    where
        Self::Parent: Sized,
    {
        ptr.cast::<u8>().wrapping_sub(self.field_offset()).cast()
    }

    /// Chains the projection of this field with another field `F`
    fn chain<F: Field<Parent = Self::Type>>(self, f: F) -> Chain<Self, F>
    where
        Self: Sized,
    {
        Chain::new(self, f)
    }
}

unsafe impl<F: ?Sized + Field> Field for &F {
    type Name = F::Name;
    type Parent = F::Parent;
    type Type = F::Type;

    #[inline]
    fn name(&self) -> Self::Name {
        F::name(self)
    }

    #[inline]
    unsafe fn project_raw(
        &self,
        ptr: *const Self::Parent,
    ) -> *const Self::Type {
        F::project_raw(self, ptr)
    }

    #[inline]
    unsafe fn project_raw_mut(
        &self,
        ptr: *mut Self::Parent,
    ) -> *mut Self::Type {
        F::project_raw_mut(self, ptr)
    }
}

unsafe impl<F: ?Sized + Field> Field for &mut F {
    type Name = F::Name;
    type Parent = F::Parent;
    type Type = F::Type;

    #[inline]
    fn name(&self) -> Self::Name {
        F::name(self)
    }

    #[inline]
    unsafe fn project_raw(
        &self,
        ptr: *const Self::Parent,
    ) -> *const Self::Type {
        F::project_raw(self, ptr)
    }

    #[inline]
    unsafe fn project_raw_mut(
        &self,
        ptr: *mut Self::Parent,
    ) -> *mut Self::Type {
        F::project_raw_mut(self, ptr)
    }
}

#[cfg(feature = "alloc")]
unsafe impl<F: ?Sized + Field> Field for std::boxed::Box<F> {
    type Name = F::Name;
    type Parent = F::Parent;
    type Type = F::Type;

    #[inline]
    fn name(&self) -> Self::Name {
        F::name(self)
    }

    #[inline]
    unsafe fn project_raw(
        &self,
        ptr: *const Self::Parent,
    ) -> *const Self::Type {
        F::project_raw(self, ptr)
    }

    #[inline]
    unsafe fn project_raw_mut(
        &self,
        ptr: *mut Self::Parent,
    ) -> *mut Self::Type {
        F::project_raw_mut(self, ptr)
    }
}

#[cfg(feature = "alloc")]
unsafe impl<F: ?Sized + Field> Field for std::rc::Rc<F> {
    type Name = F::Name;
    type Parent = F::Parent;
    type Type = F::Type;

    #[inline]
    fn name(&self) -> Self::Name {
        F::name(self)
    }

    #[inline]
    unsafe fn project_raw(
        &self,
        ptr: *const Self::Parent,
    ) -> *const Self::Type {
        F::project_raw(self, ptr)
    }

    #[inline]
    unsafe fn project_raw_mut(
        &self,
        ptr: *mut Self::Parent,
    ) -> *mut Self::Type {
        F::project_raw_mut(self, ptr)
    }
}

#[cfg(feature = "alloc")]
unsafe impl<F: ?Sized + Field> Field for std::sync::Arc<F> {
    type Name = F::Name;
    type Parent = F::Parent;
    type Type = F::Type;

    #[inline]
    fn name(&self) -> Self::Name {
        F::name(self)
    }

    #[inline]
    unsafe fn project_raw(
        &self,
        ptr: *const Self::Parent,
    ) -> *const Self::Type {
        F::project_raw(self, ptr)
    }

    #[inline]
    unsafe fn project_raw_mut(
        &self,
        ptr: *mut Self::Parent,
    ) -> *mut Self::Type {
        F::project_raw_mut(self, ptr)
    }
}
