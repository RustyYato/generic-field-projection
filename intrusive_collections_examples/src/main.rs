#![feature(raw_ref_op)]
#![allow(unused)]

use gfp_core::*;
use gfp_derive::{Field, *};

#[derive(Field)]
struct Foo<T, U: Copy> {
    x: T,
    y: U,
}

#[derive(Field)]
struct Bar<T>(u32, T);

fn test() {
    let foo_fields = Foo::<Bar<f32>, i32>::fields();
    let bar_fields = Bar::<f32>::fields();

    let field = foo_fields.x.chain(bar_fields.1);

    let foo = Foo {
        x: Bar(0, 2.0),
        y: 12,
    };

    assert_eq!(*foo.project_to(field), 2.0);
}

fn main() {
    test();
}
