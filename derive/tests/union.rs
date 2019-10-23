use gfp_core::*;

#[derive(Field)]
union Union {
    foo: u32,
    bar: [u8; 4],
}

pub fn union() {
    let a = Union { bar: [0, 1, 2, 3] };

    let union = unsafe { Union::fields() };

    let foo = a.project_to(union.foo);

    assert_eq!(*foo, unsafe { a.foo });
}
