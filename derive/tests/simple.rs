#![allow(unused)]

use gfp_core::*;
use gfp_derive::*;

// #[derive(Field)]
// struct Foo<T, U: Copy> {
//     x: T,
//     y: U
// }

#[derive(Field)]
struct Bar<T>(u32, T);

fn test() {
    // let FooFields { x, y } = Foo::<u32, u16>::fields();
    // let BarFields { } = Bar::fields();
}
