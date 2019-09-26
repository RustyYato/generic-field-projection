#![feature(const_fn_union)]
// #![forbid(missing_docs)]
#![no_std]

/*!
# generic field projection

This crate provides a generic interface to project to fields, think of it as an extended version
of `Deref` that handles all pointer types equally.
*/

mod project;
mod pin;
mod field_type;
mod descriptor;
mod macros;

pub use self::pin::*;
pub use self::field_type::*;
pub use self::descriptor::*;

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
/// #[repr(C)]
/// struct Foo {
///     y: u8,
///     x: u32,
/// }
/// 
/// struct Foo_x;
/// 
/// impl Field for Foo_x {
///     // Parent type of `Foo_x` is `Foo`, because field `x` is from type `Foo`
///     type Parent = Foo;
///     
///     // Field `x` of type `Foo` has the type ` 
///     type Type = u32;
///     
///     // Field `x` is offset `4` bytes from the start of `Foo`
///     fn field_descriptor(self) -> FieldType<Self::Parent, Self::Type> {
///         unsafe { FieldDescriptor::from_offset(4) }
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

    /// Get the field descriptor that can be used to access the field
    fn field_descriptor(&self) -> FieldDescriptor<Self::Parent, Self::Type>;

    /// Convert to an efficient dynamic representation
    fn into_dyn(self) -> FieldType<Self::Parent, Self::Type> where Self:Sized {
        FieldType { descriptor: self.field_descriptor() }
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
            PinProjectableField::new_unpin(Self::new())
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

        fn field_descriptor(&self) -> FieldDescriptor<MyType<T>, T> {
            unsafe {
                FieldDescriptor::from_offset(core::mem::align_of::<T>().max(2))
            }
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
