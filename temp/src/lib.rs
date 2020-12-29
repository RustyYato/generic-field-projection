// #![no_std]
#![feature(raw_ref_op)]
#![feature(arbitrary_self_types)]

mod linked_list {
    use core::cell::Cell;
    use gfp_core::Field;

    pub struct Link {
        link: Cell<*const Link>,
    }

    impl Link {
        pub fn new() -> Self {
            Self {
                link: Cell::new(core::ptr::null_mut()),
            }
        }

        pub unsafe fn set(&self, next: *const Link) {
            self.link.set(next)
        }

        pub fn get(&self) -> *const Link {
            self.link.get()
        }

        pub unsafe fn next_value<F: Field<Type = Link>>(
            &self,
            field: F,
        ) -> *const F::Parent {
            if self.link.get().is_null() {
                core::ptr::null_mut()
            } else {
                field.inverse_project_raw(self.link.get())
            }
        }

        pub fn unlink(&self) {
            self.link.set(core::ptr::null_mut());
        }
    }

    #[derive(Field)]
    pub struct DoubleLink {
        next: Link,
        prev: Link,
    }

    impl DoubleLink {
        pub fn new() -> Self {
            Self {
                next: Link::new(),
                prev: Link::new(),
            }
        }

        pub fn next(&self) -> *const DoubleLink {
            unsafe {
                let next = self.next.get();
                if next.is_null() {
                    core::ptr::null_mut()
                } else {
                    Self::fields().next.inverse_project_raw(next)
                }
            }
        }

        pub fn prev(&self) -> *const DoubleLink {
            unsafe {
                let prev = self.prev.get();
                if prev.is_null() {
                    core::ptr::null_mut()
                } else {
                    Self::fields().prev.inverse_project_raw(prev)
                }
            }
        }

        pub unsafe fn set_next(self: *const Self, next: *const Self) {
            let next = if next.is_null() {
                core::ptr::null_mut()
            } else {
                (*next).prev.set(Self::fields().prev.project_raw(self));
                Self::fields().next.project_raw(next)
            };
            (*self).next.set(next);
        }

        pub unsafe fn set_prev(self: *const Self, prev: *const Self) {
            let prev = if prev.is_null() {
                core::ptr::null_mut()
            } else {
                (*prev).next.set(Self::fields().next.project_raw(self));
                Self::fields().prev.project_raw(prev)
            };
            (*self).prev.set(prev);
        }

        pub unsafe fn insert_next(self: *const Self, next: *const Self) {
            let fields = Self::fields();
            if !(*self).next.get().is_null() {
                let self_next = (*self).next.get();
                fields.next.inverse_project_raw(self_next).set_prev(next);
            }
            self.set_next(next);
        }

        pub unsafe fn insert_prev(self: *const Self, prev: *const Self) {
            let fields = Self::fields();
            if !(*self).prev.get().is_null() {
                let self_prev = (*self).prev.get();
                fields.prev.inverse_project_raw(self_prev).set_next(prev);
            }
            self.set_prev(prev);
        }

        pub unsafe fn next_value<F: Field<Type = DoubleLink>>(
            &self,
            field: F,
        ) -> *const F::Parent {
            self.next.next_value(field.chain(DoubleLink::fields().next))
        }

        pub unsafe fn prev_value<F: Field<Type = DoubleLink>>(
            &self,
            field: F,
        ) -> *const F::Parent {
            self.prev.next_value(field.chain(DoubleLink::fields().prev))
        }

        pub fn unlink(&self) {
            let fields = Self::fields();
            let next = self.next.get();
            let prev = self.prev.get();

            let next = if next.is_null() {
                core::ptr::null_mut()
            } else {
                unsafe { fields.next.inverse_project_raw(next) }
            };
            let prev = if prev.is_null() {
                core::ptr::null_mut()
            } else {
                unsafe { fields.prev.inverse_project_raw(prev) }
            };

            unsafe {
                if !prev.is_null() {
                    prev.set_next(next);
                }
                if !next.is_null() {
                    next.set_prev(prev);
                }
            }

            self.next.unlink();
            self.prev.unlink();
        }
    }
}

use gfp_core::Field;
use linked_list::DoubleLink;

#[derive(Field)]
pub struct Foo {
    x:    Box<i32>,
    y:    Box<i32>,
    link: DoubleLink,
}

impl Foo {
    pub fn new() -> Self {
        Self {
            x:    Box::new(0),
            y:    Box::new(0),
            link: DoubleLink::new(),
        }
    }

    fn link(&self) -> *const DoubleLink {
        unsafe { Self::fields().link.project_raw(self) }
    }

    unsafe fn set_next(&self, next: Option<&Self>) {
        let next = core::mem::transmute::<_, *const Self>(next);
        let link = Self::fields().link;
        let self_next = self.link.next();
        if !self_next.is_null() {
            self_next.set_prev(core::ptr::null());
        }
        link.project_raw(self).set_next(link.project_raw(next));
    }

    unsafe fn set_prev(&self, prev: Option<&Self>) {
        let prev = core::mem::transmute::<_, *const Self>(prev);
        let link = Self::fields().link;
        let self_prev = self.link.prev();
        if !self_prev.is_null() {
            self_prev.set_prev(core::ptr::null());
        }
        link.project_raw(self).set_prev(link.project_raw(prev))
    }

    unsafe fn insert_next(&self, next: &Self) {
        let link = Self::fields().link;
        link.project_raw(self).insert_next(link.project_raw(next))
    }

    unsafe fn insert_prev(&self, prev: &Self) {
        let link = Self::fields().link;
        link.project_raw(self).insert_prev(link.project_raw(prev))
    }

    unsafe fn next(&self) -> Option<&Self> {
        self.link.next_value(Foo::fields().link).as_ref()
    }

    unsafe fn prev(&self) -> Option<&Self> {
        self.link.prev_value(Foo::fields().link).as_ref()
    }

    unsafe fn unlink(&self) {
        self.link.unlink();
    }

    fn get(&self) -> (i32, i32) {
        (*self.x, *self.y)
    }
}

#[test]
fn foo() {
    let mut foo = Foo::new();
    let mut bar = Foo::new();
    let mut yam = Foo::new();

    *foo.x = 10;
    *foo.y = 20;

    *bar.x = 30;
    *bar.y = 40;

    *yam.x = 50;
    *yam.y = 60;

    unsafe {
        foo.set_next(Some(&yam));
        foo.set_next(Some(&bar));

        assert!(matches!(foo.prev(), None));
        assert_eq!(foo.next().map(Foo::get), Some((30, 40)));
        assert_eq!(bar.prev().map(Foo::get), Some((10, 20)));
        assert_eq!(bar.next().map(Foo::get), None);
        assert_eq!(yam.prev().map(Foo::get), None);
        assert_eq!(yam.next().map(Foo::get), None);

        foo.unlink();

        assert!(matches!(bar.prev(), None));

        foo.insert_next(&yam);
        foo.insert_next(&bar);

        assert!(matches!(foo.prev(), None));
        assert_eq!(foo.next().map(Foo::get), Some((30, 40)));
        assert_eq!(bar.prev().map(Foo::get), Some((10, 20)));
        assert_eq!(bar.next().map(Foo::get), Some((50, 60)));
        assert_eq!(yam.prev().map(Foo::get), Some((30, 40)));
        assert_eq!(yam.next().map(Foo::get), None);
    }
}
