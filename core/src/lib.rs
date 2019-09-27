#![feature(const_fn_union, const_fn)]
#![forbid(missing_docs)]
#![no_std]

/*!
This crate provides a generic interface to project to fields, think of it as an extended version
of `Deref` that handles all pointer types equally.
*/

mod project;
mod pin;
mod field_type;
mod descriptor;
mod macros;
mod chain;

pub use self::pin::*;
pub use self::field_type::*;
pub use self::descriptor::*;
pub use self::chain::*;

/// Projects a type to the given field
pub trait ProjectTo<F: Field + ?Sized> {
    /// The projection of the type, can be used to directly access the field
    type Projection;

    /// projects to the given field
    fn project_to(self, field: &F) -> Self::Projection;
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

#[test]
#[allow(non_camel_case_types)]
fn simple_test() {
    struct MyType {
        _x: u8,
        _y: u8,
        z: u32,
    }

    field!(MyType_z(MyType => u32), z, MyType { _x: 0, _y: 0, z: 0 });

    impl MyType_z {
        pub fn pin() -> PinProjectableField<Self> {
            unsafe { PinProjectableField::new_unchecked(Self::new()) }
        }
    }

    let my_type = MyType {
        _x: 0,
        _y: 1,
        z: 3
    };

    use core::pin::Pin;

    let my_type_pin = Pin::new(&my_type);

    assert_eq!(*my_type_pin.project_to(&MyType_z::pin()), 3);
}

#[test]
pub fn generic_test() {
    use core::marker::PhantomData;

    #[repr(C)]
    struct MyType<T> {
        x: u8,
        y: u8,
        z: T
    }

    #[allow(non_camel_case_types)]
    struct MyType_z<T>(PhantomData<fn(T) -> T>);

    unsafe impl<T> Field for MyType_z<T> {
        type Parent = MyType<T>;
        type Type = T;
        type Name = core::iter::Once<&'static str>;

        fn name(&self) -> Self::Name {
            core::iter::once("z")
        }

        unsafe fn project_raw(&self, ptr: *const Self::Parent) -> *const Self::Type {
            &(*ptr).z
        }
        
        unsafe fn project_raw_mut(&self, ptr: *mut Self::Parent) -> *mut Self::Type {
            &mut (*ptr).z
        }
    }

    impl<T> MyType_z<T> {
        pub fn new() -> Self {
            Self(PhantomData)
        }
        
        pub fn pin() -> PinProjectableField<Self> {
            unsafe {
                PinProjectableField::new_unchecked(Self::new())
            }
        }
    }

    assert_eq!(core::mem::size_of_val(&MyType_z::<u32>::pin()), 0);

    use core::pin::Pin;

    let my_type = MyType {
        x: 0,
        y: 1,
        z: 3u8
    };

    let my_type_pin = Pin::new(&my_type);

    assert_eq!(*my_type_pin.project_to(&MyType_z::pin()), 3);
    
    let my_type = MyType {
        x: 0,
        y: 1,
        z: 3u32
    };

    let my_type_pin = Pin::new(&my_type);

    assert_eq!(*my_type_pin.project_to(&MyType_z::pin()), 3);
}

#[test]
#[allow(non_camel_case_types, unused)]
fn test_dyn() {
    struct MyType {
        x: u8,
        y: u8,
        z: u8,
    }

    field!(MyType_x(MyType => u8), x, MyType { x: 0, y: 0, z: 0 });
    field!(MyType_y(MyType => u8), y, MyType { x: 0, y: 0, z: 0 });
    field!(MyType_z(MyType => u8), z, MyType { x: 0, y: 0, z: 0 });

    let my_type = MyType {
        x: 0,
        y: 1,
        z: 2
    };

    let fields: [&dyn Field<Parent = MyType, Type = u8, Name = _>; 3] = [&MyType_x, &MyType_y, &MyType_z];

    for (i, field) in fields.iter().enumerate() {
        assert_eq!(*my_type.project_to(field), i as u8)
    }
}

#[test]
#[allow(non_camel_case_types, unused)]
fn test_chain() {
    #[derive(Default)]
    struct Foo {
        x: u8,
        y: Bar,
    }

    #[derive(Default)]
    struct Bar {
        a: u16,
        b: u32,
    }

    field!(Foo_y(Foo => Bar), y, Foo::default());
    field!(Bar_b(Bar => u32), b, Bar::default());

    let my_type = Foo {
        x: 0,
        y: Bar {
            a: 1,
            b: 2
        }
    };

    assert_eq!(
        *my_type.project_to(&Foo_y.chain(Bar_b)),
        2
    );
}
