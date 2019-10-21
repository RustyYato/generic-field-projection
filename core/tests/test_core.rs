#![allow(non_camel_case_types, clippy::blacklisted_name)]

#[global_allocator]
static ALLOC: static_alloc::Slab<[u8; 1 << 16]> = static_alloc::Slab::uninit();

use gfp_core::*;

#[derive(Default, Field)]
struct Foo {
    x: u8,
    y: Bar,
    z: u128,
}

#[derive(Default, Field)]
struct Bar {
    a: u16,
    b: u32,
    c: Quaz,
}

#[derive(Default, Field)]
struct Quaz {
    q: (u16, u32),
    r: u32,
}

#[test]
#[allow(non_camel_case_types)]
fn simple_test() {
    use core::pin::Pin;

    let mut foo = Foo::default();

    foo.x = 3;

    let foo_pin = Pin::new(&foo);

    let x = Foo::fields().x;

    unsafe {
        assert_eq!(*foo_pin.project_to(PinToPin::new_unchecked(x)), 3);
    }
}

#[derive(Field)]
struct MyType<T> {
    x: u8,
    y: u8,
    z: T
}

#[test]
pub fn generic_test() {
    impl<T> MyType_fields::z<MyType<T>> {
        pub fn pin(self) -> PinToPin<Self> {
            unsafe {
                PinToPin::new_unchecked(self)
            }
        }
    }

    let z = MyType::fields().z;
    assert_eq!(core::mem::size_of_val(&z.pin()), 0);

    use core::pin::Pin;

    let my_type = MyType {
        x: 0,
        y: 1,
        z: 3u8
    };

    let my_type_pin = Pin::new(&my_type);

    assert_eq!(*my_type_pin.project_to(z.pin()), 3);
    
    let my_type = MyType {
        x: 0,
        y: 1,
        z: 3u32
    };

    let z = MyType::fields().z;
    let my_type_pin = Pin::new(&my_type);

    assert_eq!(*my_type_pin.project_to(z.pin()), 3);
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

    let foo = Foo::fields();
    let bar = Bar::fields();
    let quaz = Quaz::fields();

    assert_eq!(
        *my_type.project_to(&foo.y.chain(bar.c).chain(quaz.r)),
        5
    );
}
