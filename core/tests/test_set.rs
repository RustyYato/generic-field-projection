#![allow(non_camel_case_types, clippy::blacklisted_name)]

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
fn simple() {
    let mut foo = Foo::default();
    let foo_ref = &mut foo;

    let FooFields { x, y, .. } = Foo::fields();
    let BarFields { a, .. } = Bar::fields();

    let (x, y_a) = foo_ref.project_set_to((
        x, y.chain(a)
    ));

    *x = 1;
    *y_a = 10;

    assert_eq!(foo.x, 1);
    assert_eq!(foo.y.a, 10);
}

#[test]
fn pin() {
    use std::pin::Pin;
    use gfp_core::{PinToPin, PinToRef};

    let FooFields { x, y, .. } = Foo::fields();
    let BarFields { a, .. } = Bar::fields();

    let mut foo = Foo::default();
    let foo_ref = Pin::new(&mut foo);

    let (mut x, y_a) = foo_ref.project_set_to((
        unsafe { PinToPin::new_unchecked(x) },
        PinToRef::new(y.chain(a))
    ));

    *x = 1;
    *y_a = 10;

    assert_eq!(foo.x, 1);
    assert_eq!(foo.y.a, 10);
}
