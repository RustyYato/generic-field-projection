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
fn simple() {
    let mut value = Foo::default();
    let foo_ref = &mut value;

    let foo = Foo::fields();
    let bar = Bar::fields();

    let (x, y_a) = foo_ref.project_set_to((foo.x, foo.y.chain(bar.a)));

    *x = 1;
    *y_a = 10;

    assert_eq!(value.x, 1);
    assert_eq!(value.y.a, 10);
}

#[test]
fn pin() {
    use gfp_core::{PinToPin, PinToPtr};
    use std::pin::Pin;

    let foo = Foo::fields();
    let bar = Bar::fields();

    let mut value = Foo::default();
    let value_ref = Pin::new(&mut value);

    let (mut x, y_a) = value_ref.project_set_to((
        unsafe { PinToPin::new_unchecked(foo.x) },
        PinToPtr::new(foo.y.chain(bar.a)),
    ));

    *x = 1;
    *y_a = 10;

    assert_eq!(value.x, 1);
    assert_eq!(value.y.a, 10);
}

#[test]
fn arc() {
    let mut foo = Foo::default();

    foo.x = 10;
    foo.y.a = 13;

    let arc = std::sync::Arc::new(foo);

    let foo = Foo::fields();
    let bar = Bar::fields();

    let (foo_x, foo_y_a) = arc
        .clone()
        .project_set_to((foo.x, foo.y.chain(bar.a)))
        .split();

    assert_eq!(*foo_x, 10);
    assert_eq!(*foo_y_a, 13);
}
