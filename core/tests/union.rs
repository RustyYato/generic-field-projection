use gfp_core::*;

#[derive(Field)]
union Union {
    foo: u32,
    bar: [u8; 4],
}

#[test]
fn union() {
    let a = Union { bar: [0, 1, 2, 3] };

    let union = unsafe { Union::fields() };

    let foo = a.project_to(union.foo);

    assert_eq!(*foo, unsafe { a.foo });
}

#[test]
#[should_panic]
fn try_aliasing() {
    let mut a = Union {
        bar: [0, 1, 2, 3] as [u8; 4],
    };

    let union = unsafe { Union::fields() };

    (&mut a).project_set_to((union.foo, union.bar));
}
