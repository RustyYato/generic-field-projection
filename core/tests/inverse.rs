#![feature(raw_ref_op)]

use gfp_core::Field;

#[derive(Field)]
struct Foo {
    x: u32,
    y: u8,
}

#[test]
fn basic_inverse() {
    let foo = Foo {
        x: 0x89abcdef,
        y: 0x23,
    };

    let fields = Foo::fields();

    let foo_ptr = &foo as *const Foo;
    unsafe {
        let foo_y_ptr = fields.y.project_raw(foo_ptr);
        assert_eq!(*foo_y_ptr, 0x23);
        let new_foo_ptr = fields.y.inverse_project_raw(foo_y_ptr);
        assert_eq!(new_foo_ptr, foo_ptr);
    }
}
