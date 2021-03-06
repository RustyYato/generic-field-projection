#![feature(dropck_eyepatch, raw_ref_op)]
#![allow(clippy::needless_doctest_main)]
#![forbid(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

//! A generic interface to project to struct fields as an extended version of
//! `Deref` which handles all pointer types equally and allows for arbitrary
//! composition and cloning of fields between selected structs using
//! procedurally generated code and associated macro based helper methods.

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc as std;

mod chain;
mod dynamic;
#[doc(hidden)]
pub mod macros;
mod pin;
mod project;
mod unchecked_project;

#[doc(hidden)]
pub mod type_list;

pub use self::{chain::*, dynamic::Dynamic, pin::*};
pub use gfp_derive::Field;

use core::{marker::PhantomData, ops::Range};

#[doc(hidden)]
#[macro_export]
macro_rules! ptr_project {
    ($mut:ident $ptr:ident $field:ident) => {
        &raw $mut (*$ptr).$field
    }
}

#[doc(hidden)]
pub mod derive {
    pub use core::iter::{once, Once};
    use core::marker::PhantomData;

    pub struct Invariant<T: ?Sized>(PhantomData<fn() -> *mut T>);

    impl<T: ?Sized> Invariant<T> {
        pub const INIT: Self = Self(PhantomData);
    }

    impl<T: ?Sized> Clone for Invariant<T> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<T: ?Sized> Copy for Invariant<T> {
    }
}

// Dev Note: we use `fn() -> T` so that we are covariant and non-owning in `T`,
// this means that auto-traits are always automatically implemented, and we have
// minimal lifetime restrictions.
/// Identity operations on a `Field` struct which are guaranteed to be no-op
pub struct Identity<T>(PhantomData<fn() -> T>);

impl<T> Identity<T> {
    /// Default initializer for the `Identity`
    pub const NEW: Self = Self(PhantomData);
}

unsafe impl<T> Field for Identity<T> {
    type Parent = T;
    type Type = T;

    unsafe fn project_raw(
        &self,
        ptr: *const Self::Parent,
    ) -> *const Self::Type {
        ptr
    }

    unsafe fn project_raw_mut(
        &self,
        ptr: *mut Self::Parent,
    ) -> *mut Self::Type {
        ptr
    }

    fn field_offset(&self) -> usize {
        0
    }

    unsafe fn inverse_project_raw(
        &self,
        ptr: *const Self::Type,
    ) -> *const Self::Parent {
        ptr
    }

    unsafe fn inverse_project_raw_mut(
        &self,
        ptr: *mut Self::Type,
    ) -> *mut Self::Parent {
        ptr
    }

    fn wrapping_project_raw(
        &self,
        ptr: *const Self::Type,
    ) -> *const Self::Parent {
        ptr
    }

    fn wrapping_project_raw_mut(
        &self,
        ptr: *mut Self::Type,
    ) -> *mut Self::Parent {
        ptr
    }

    fn wrapping_inverse_project_raw(
        &self,
        ptr: *const Self::Type,
    ) -> *const Self::Parent {
        ptr
    }

    fn wrapping_inverse_project_raw_mut(
        &self,
        ptr: *mut Self::Type,
    ) -> *mut Self::Parent {
        ptr
    }
}

/// Projects a type to a given `Field`
pub trait ProjectTo<F: Field> {
    /// Direct access to the field
    type Projection;

    /// Project to a given `Field`
    fn project_to(self, field: F) -> Self::Projection;
}

/// Project a given `Field` onto a procedurally generated Field using an `unsafe`
/// field projection trait. Safe usage depends entirely on the type implementing
/// this trait.
///
/// Safety:
/// * For `*const T`, `*mut T`, and `NonNull<T>`, the safety condition is
/// equivalent to `project_raw`/`project_raw_mut`
///
/// * For `Option<T>`, if it is `Some`, the safety condition on `T` applies
///
/// * Other types must define their own safety conditions
pub trait UncheckedProjectTo<F: Field> {
    /// Direct access to via a pointer-like type to the field
    type Projection;

    /// Projection to a given `Field`s field
    ///
    /// # Safety
    ///
    /// see trait docs
    unsafe fn project_to(self, field: F) -> Self::Projection;
}

/// Project a generated `Field` type back into its related source `struct` using
/// an `unsafe` field projection trait. Safe usage depends entirely on the type
/// implementing this trait.
///
/// Safety Specifications:
/// * For `*const T`, `*mut T`, and `NonNull<T>`, the safety condition is
/// equivalent to `inverse_project_raw`/`inverse_project_raw_mut`
///
/// * For `Option<T>`, if it is `Some`, the safety condition on `T` applies
///
/// * Other types must define their own safety conditions
pub trait UncheckedInverseProjectTo<F: Field> {
    /// Direct access via a pointer-like type to the field
    type Projection;

    /// Projection to the related source `struct` of a given `Field`
    ///
    /// # Safety
    ///
    /// see trait docs
    unsafe fn inverse_project_to(self, field: F) -> Self::Projection;
}

/// Projects a type to the given `Field` list
pub trait ProjectAll<Parent, F> {
    /// Direct access to the generated `Field` list
    type Projection;

    /// Projection to the given `Field` list
    fn project_all(self, field_list: F) -> Self::Projection;
}

/// A generated representation of some `Parent` types `struct` or `union`.
/// `Parent` represents the type where the field came from, `Type` represents
/// the type of the field itself.
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
/// * `project_raw` and `project_raw_mut` must only access the given field
/// * `field_offset`, `inverse_project_raw`, or `inverse_project_raw_mut` must
/// not be overridden
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
/// It's possible to get field `val` from `Foo` by implementing `Field` manually:
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
/// Best practice is to derive `Field` on the needed types, using the
/// [`chain`](trait.Fields.html#method.chain) combinator to project to the
/// fields of `Field` rather than implementing `Field` manually
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
///     // or for convenience
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
    // TODO: find a way to relax both of these bounds on `Parent` and `Type` to
    // `?Sized` see
    // https://github.com/RustyYato/generic-field-projection/issues/39 for more
    // information

    /// Type which is generating `Field`
    type Parent;

    /// A type representation of `Field` itself
    type Type;

    /// Project a raw pointer from `Parent` to `Type`
    ///
    /// # Safety
    ///
    /// * `ptr` must point to a valid allocation of `Parent`
    /// * the projection is not safe to write to
    unsafe fn project_raw(&self, ptr: *const Self::Parent)
    -> *const Self::Type;

    /// Project a mutable raw pointer from `Parent` to `Type`
    ///
    /// # Safety
    ///
    /// * `ptr` must point to a valid allocation of `Parent`
    unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent)
    -> *mut Self::Type;

    /// Return range of offsets covered by the field
    fn range(&self) -> Range<usize> {
        let offset = self.field_offset();
        offset..offset.wrapping_add(core::mem::size_of::<Self::Type>())
    }

    /// Create an equivalent run-time offset-based `Field`
    fn dynamic(&self) -> Dynamic<Self::Parent, Self::Type> {
        unsafe { Dynamic::from_offset(self.field_offset()) }
    }

    /// Return the offset of a `Field` from a base pointer of a `Parent`, the
    /// offset will always be positive, since `Field`s are derived from
    /// `Parent`s
    fn field_offset(&self) -> usize {
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

            // The offset will always be positive, because fields must come
            // after their parents
            offset as usize
        }
    }

    /// Project a raw pointer from a `Type` to its generating `Parent`
    ///
    /// # Safety
    ///
    /// * `ptr` must point into a valid allocation of `Type`
    /// * `ptr` must point to a field of `Parent` with the type `Type`
    /// * `ptr` must have provenance over `Parent`
    ///
    /// In particular it's always instant UB to convert a
    /// `field: &Type` to `&Parent`
    ///
    /// For example, the following function is UB no matter the choice of `Parent`
    /// and `Field`, because `&Field` *never* has provenance over `Parent`
    /// ```rust
    /// # struct Parent { field: Type } struct Type; use gfp_core::Field;
    /// unsafe fn unsound(field: &Type, field_type: impl Field<Type = Type, Parent = Parent>) -> *const Parent {
    ///     field_type.inverse_project_raw(field)
    /// }
    /// ```
    ///
    /// Therefore, in order to safely use `inverse_project_raw` safely,
    /// you *must* only operate on raw pointers.
    unsafe fn inverse_project_raw(
        &self,
        ptr: *const Self::Type,
    ) -> *const Self::Parent {
        // Safety
        // * `ptr` is guaranteed to be a pointer to a field of `Parent`
        // * `field_offset` is guarateed to give the correct offset of the field
        ptr.cast::<u8>().sub(self.field_offset()).cast()
    }

    /// Project a raw pointer from a `Type` to its generating `Parent`
    ///
    /// # Safety
    ///
    /// * `ptr` must point into a valid allocation of `Type`
    /// * `ptr` must point to a field of `Parent` with the type `Type`
    /// * `ptr` must have provenance over `Parent`
    ///
    /// In particular it's always instant UB to convert a
    /// `field: &mut Type` to `&mut Parent`
    ///
    /// For example, the following function is UB no matter the choice of `Parent`
    /// and `Field`, because `&Field` *never* has provenance over `Parent`
    /// ```rust
    /// # struct Parent { field: Type } struct Type; use gfp_core::Field;
    /// unsafe fn unsound(field: &mut Type, field_type: impl Field<Type = Type, Parent = Parent>) -> *mut Parent {
    ///     field_type.inverse_project_raw_mut(field)
    /// }
    /// ```
    ///
    /// Therefore, in order to safely use `inverse_project_raw_mut` safely,
    /// you *must* only operate on raw pointers.
    unsafe fn inverse_project_raw_mut(
        &self,
        ptr: *mut Self::Type,
    ) -> *mut Self::Parent {
        // Safety
        // * `ptr` is guaranteed to be a pointer to a field of `Parent`
        // * `field_offset` is guarateed to give the correct offset of the field
        ptr.cast::<u8>().sub(self.field_offset()).cast()
    }

    /// Project a raw pointer from a `Type` to its `Parent`
    ///
    /// # Safety
    ///
    /// In order to safely dereference the resulting pointer, you *must* follow
    /// the safety docs in `project_raw`
    fn wrapping_project_raw(
        &self,
        ptr: *const Self::Type,
    ) -> *const Self::Parent {
        ptr.cast::<u8>().wrapping_add(self.field_offset()).cast()
    }

    /// Project a raw pointer from a `Type` to its `Parent`
    ///
    /// # Safety
    ///
    /// In order to safely dereference the resulting pointer, you *must* follow
    /// the safety docs in `project_raw_mut`
    fn wrapping_project_raw_mut(
        &self,
        ptr: *mut Self::Type,
    ) -> *mut Self::Parent {
        ptr.cast::<u8>().wrapping_add(self.field_offset()).cast()
    }

    /// Projects a raw pointer from a `Type` to its `Parent`
    ///
    /// # Safety
    ///
    /// In order to safely dereference the resulting pointer, you *must* follow
    /// the safety docs in `inverse_project_raw`
    fn wrapping_inverse_project_raw(
        &self,
        ptr: *const Self::Type,
    ) -> *const Self::Parent {
        ptr.cast::<u8>().wrapping_sub(self.field_offset()).cast()
    }

    /// Projects a raw pointer from a `Type` to its `Parent`
    ///
    /// # Safety
    ///
    /// In order to safely dereference the resulting pointer, you *must* follow
    /// the safety docs in `inverse_project_raw_mut`
    fn wrapping_inverse_project_raw_mut(
        &self,
        ptr: *mut Self::Type,
    ) -> *mut Self::Parent {
        ptr.cast::<u8>().wrapping_sub(self.field_offset()).cast()
    }

    /// Chain a projection of one `Field` with another
    fn chain<F: Field<Parent = Self::Type>>(self, f: F) -> Chain<Self, F>
    where
        Self: Sized,
    {
        Chain::new(self, f)
    }
}

unsafe impl<F: ?Sized + Field> Field for &F {
    type Parent = F::Parent;
    type Type = F::Type;

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
    type Parent = F::Parent;
    type Type = F::Type;

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
    type Parent = F::Parent;
    type Type = F::Type;

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
    type Parent = F::Parent;
    type Type = F::Type;

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
    type Parent = F::Parent;
    type Type = F::Type;

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
