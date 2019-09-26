#![allow(unused)]

use gfp_core::*;
use gfp_derive::*;

#[derive(Field)]
#[default]
struct Foo<T, U: Copy> {
    x: T,

    #[pin]
    y: U
}

fn test(f: &Foo<u32, u32>) -> impl gfp_core::Field<Parent = Foo<u32, u32>, Type = u32> {
    f.fields().x
}