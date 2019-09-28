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
fn simple() {
    let mut foo = Foo::default();
    let foo_ref = &mut foo;

    let (x, y) = foo_ref.project_set_to((
        Foo_x,
        Foo_y
    ));

    *x = 1;
    y.a = 10;

    assert_eq!(foo.x, 1);
    assert_eq!(foo.y.a, 10);
}
