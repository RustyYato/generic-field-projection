#![feature(const_fn_union, const_fn, specialization)]
// #![forbid(missing_docs)]
#![no_std]

/*!
This crate provides a generic interface to project to fields, think of it as an extended version
of `Deref` that handles all pointer types equally.
*/

mod project;
mod project_set;
mod pin;
#[doc(hidden)]
pub mod macros;
mod chain;
mod set;

pub use self::pin::*;
pub use self::chain::*;
pub use self::set::{FieldSet, tuple::*};
pub use gfp_derive::Field;

#[doc(hidden)]
pub mod derive {
    pub use core::marker::PhantomData;
    pub use core::iter::{Once, once};
}

/// Projects a type to the given field
pub trait ProjectTo<F: Field> {
    /// The projection of the type, can be used to directly access the field
    type Projection;

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

/// Represents a field of some `Parent` type
/// 
/// e.x.
/// 
/// ```rust
/// # use gfp_core::*;
/// struct Foo {
///     y: u8,
///     x: u32,
/// }
/// 
/// struct Foo_x;
/// 
/// unsafe impl Field for Foo_x {
///     // Parent type of `Foo_x` is `Foo`, because field `x` is from type `Foo`
///     type Parent = Foo;
///     
///     // Field `x` of type `Foo` has the type ` 
///     type Type = u32;
/// 
///     type Name = std::iter::Once<&'static str>;
///     
///     fn name(&self) -> Self::Name {
///         std::iter::once("x")
///     }
///     
///     unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
///         &(*ptr).x
///     }
/// 
///     unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
///         &mut (*ptr).x
///     }
/// }
/// ```
/// 
/// # Safety
/// 
/// * `Parent` must represent the type where the field came from
/// * `Type` must represent the type of the field itself
/// * `field_descriptor` must return the correct descriptor for the given field
///     * i.e. using `FieldDescriptor::project_raw_unchecked` on a valid `*const Field::Parent` should be sound
/// * `into_dyn` must not have a different implementation from the default implementation
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
    /// * `ptr` must point to a valid, initialized allocation of `Parent`
    /// * the projection is not safe to write to
    unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type;
    
    /// projects the raw pointer from the `Parent` type to the field `Type`
    /// 
    /// # Safety
    /// 
    /// `ptr` must point to a valid, initialized allocation of `Parent`
    unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type;

    /// Chains the projection of this field with another field `F`
    fn chain<F: Field<Parent = Self::Type>>(self, f: F) -> Chain<Self, F> where Self: Sized {
        Chain::new(self, f)
    }
}

unsafe impl<F: ?Sized + Field> Field for &F {
    type Parent = F::Parent;
    type Type = F::Type;
    type Name = F::Name;

    #[inline]
    fn name(&self) -> Self::Name {
        F::name(self)
    }
    
    #[inline]
    unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
        F::project_raw(self, ptr)
    }
    
    #[inline]
    unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
        F::project_raw_mut(self, ptr)
    }
}

unsafe impl<F: ?Sized + Field> Field for &mut F {
    type Parent = F::Parent;
    type Type = F::Type;
    type Name = F::Name;

    #[inline]
    fn name(&self) -> Self::Name {
        F::name(self)
    }
 
    #[inline]   
    unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
        F::project_raw(self, ptr)
    }
    
    #[inline]
    unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
        F::project_raw_mut(self, ptr)
    }
}
