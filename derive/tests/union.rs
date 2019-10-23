use gfp_core::*;

#[derive(Field)]
union Union<T: Copy> {
    foo: u32,
    bar: T
}
