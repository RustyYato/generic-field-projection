#![allow(non_camel_case_types, clippy::blacklisted_name)]

use gfp_core::*;

#[derive(Default)]
struct Foo {
    x: u8,
    y: Bar,
    z: u128,
}

#[derive(Default)]
struct Bar {
    a: u16,
    b: u32,
    c: Quaz,
}

#[derive(Default)]
struct Quaz {
    q: (u16, u32),
    r: u32,
}

field!(Foo_x(Foo => u8), x, Foo::default());
field!(Foo_y(Foo => Bar), y, Foo::default());
field!(Foo_z(Foo => u128), z, Foo::default());

field!(Bar_a(Bar => u16), a, Bar::default());
field!(Bar_b(Bar => u32), b, Bar::default());
field!(Bar_c(Bar => Quaz), c, Bar::default());

field!(Quaz_q(Quaz => (u16, u32)), q, Quaz::default());
field!(Quaz_r(Quaz => u32), r, Quaz::default());

#[test]
#[allow(non_camel_case_types)]
fn simple_test() {
    use core::pin::Pin;

    let mut foo = Foo::default();

    foo.x = 3;

    let foo_pin = Pin::new(&foo);

    unsafe {
        assert_eq!(*foo_pin.project_to(PinProjectableField::new_unchecked(Foo_x)), 3);
    }
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

    unsafe {
        assert_eq!(*my_type_pin.project_to(PinProjectableField::new_unchecked(MyType_z::new())), 3);
    }
    
    let my_type = MyType {
        x: 0,
        y: 1,
        z: 3u32
    };

    let my_type_pin = Pin::new(&my_type);

    unsafe {
        assert_eq!(*my_type_pin.project_to(PinProjectableField::new_unchecked(MyType_z::new())), 3);
    }
}

#[test]
#[allow(non_camel_case_types, unused)]
fn test_chain() {
    let mut my_type = Foo {
        x: 0,
        y: Bar {
            a: 1,
            b: 2,
            c: Quaz {
                q: (3, 4),
                r: 5
            }
        },
        z: 6
    };

    assert_eq!(
        *my_type.project_to(&Foo_y.chain(Bar_c).chain(Quaz_r)),
        5
    );
}
