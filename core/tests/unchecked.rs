#![feature(raw_ref_op)]
#![allow(non_camel_case_types, clippy::blacklisted_name)]

use std::ptr::NonNull;

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

#[derive(Field)]
struct MyType<T> {
    x: u8,
    y: u8,
    z: T,
}

#[test]
#[allow(non_snake_case)]
fn test_nonnull() {
    let foo = Foo::default();
    let foo_addr = &foo as *const Foo as usize;
    let nn_foo = NonNull::from(&foo);
    let Foo = Foo::fields();
    let Bar = Bar::fields();
    let nn_foo_y_b = unsafe { nn_foo.project_to(Foo.y.chain(Bar.b)) };
    let offset = nn_foo_y_b.as_ptr() as usize - foo_addr;
    assert_eq!(offset, Foo.y.chain(Bar.b).field_offset())
}

#[test]
#[allow(non_snake_case)]
fn test_option() {
    let foo = Foo::default();
    let mut opt_nn_foo = Some(NonNull::from(&foo));
    let nn_foo = NonNull::from(&foo);
    let Foo = Foo::fields();
    let Bar = Bar::fields();
    let opt_nn_foo_y_b = unsafe { opt_nn_foo.project_to(Foo.y.chain(Bar.b)) };
    let nn_foo_y_b = unsafe { nn_foo.project_to(Foo.y.chain(Bar.b)) };
    assert_eq!(opt_nn_foo_y_b, Some(nn_foo_y_b));
    opt_nn_foo = None;
    let nn_foo_y_b = unsafe { opt_nn_foo.project_to(Foo.y.chain(Bar.b)) };
    assert!(nn_foo_y_b.is_none());
}
