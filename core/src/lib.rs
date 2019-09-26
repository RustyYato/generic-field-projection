#![feature(const_fn_union)]

mod project;
mod pin;
mod field_type;
mod descriptor;
mod macros;

pub use self::pin::*;
pub use self::field_type::*;
pub use self::descriptor::*;

pub trait ProjectTo<F: Field + ?Sized> {
    type Projection;

    fn project_to(self, field: &F) -> Self::Projection;
}

pub unsafe trait Field {
    /// The type of the type that the field comes from
    type Parent: ?Sized;

    /// The type of the field itself
    type Type: ?Sized;

    fn field_descriptor(&self) -> FieldDescriptor<Self::Parent, Self::Type>;

    fn into_dyn(self) -> FieldType<Self::Parent, Self::Type> where Self:Sized {
        unsafe {
            FieldType::new_unchecked(self.field_descriptor())
        }
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

    use std::pin::Pin;

    let my_type_pin = Pin::new(&my_type);

    assert_eq!(*my_type_pin.project_to(&MyType_z::pin()), 3);
}

#[test]
pub fn generic_test() {
    use std::marker::PhantomData;

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
                FieldDescriptor::from_offset(std::mem::align_of::<T>().max(2))
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

    assert_eq!(std::mem::size_of_val(&MyType_z::<u32>::pin()), 0);

    use std::pin::Pin;

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
