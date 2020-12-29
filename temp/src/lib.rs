// #![no_std]
#![feature(raw_ref_op)]
#![feature(arbitrary_self_types)]

pub mod linked_list {
    use core::{cell::Cell, ptr::NonNull};
    use gfp_core::Field;

    pub struct Link {
        link: Cell<Option<NonNull<Link>>>,
    }

    impl Link {
        pub fn new() -> Self {
            Self {
                link: Cell::new(None),
            }
        }

        pub fn unlink(&self) {
            self.link.set(None);
        }

        pub unsafe fn link(&self, next: NonNull<Link>) {
            self.link.set(Some(next))
        }

        pub unsafe fn set(&self, next: Option<NonNull<Link>>) {
            self.link.set(next)
        }

        pub fn get(&self) -> Option<NonNull<Link>> {
            self.link.get()
        }

        pub unsafe fn owner<F: Field<Type = Link>>(
            &self,
            field: F,
        ) -> Option<NonNull<F::Parent>> {
            self.get().map(|link| unsafe {
                NonNull::new_unchecked(
                    field.inverse_project_raw_mut(link.as_ptr()),
                )
            })
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

        pub unsafe fn unlink_next(&self) {
            let next = self.next.get();
            if !next.is_null() {
                let next = Self::fields().next.inverse_project_raw(next);
                (*next).prev.link(core::ptr::null());
            }
            self.next.link(core::ptr::null());
        }

        pub unsafe fn unlink_prev(&self) {
            let next = self.next.get();
            if !next.is_null() {
                let next = Self::fields().next.inverse_project_raw(next);
                (*next).prev.link(core::ptr::null());
            }
            self.next.link(core::ptr::null());
        }

        pub unsafe fn link_next(self: *const Self, next: *const Self) {
            (*next).prev.link(Self::fields().prev.project_raw(self));
            let next = Self::fields().next.project_raw(next);
            (*self).next.link(next);
        }

        pub unsafe fn link_prev(self: *const Self, prev: *const Self) {
            (*prev).next.link(Self::fields().next.project_raw(self));
            let prev = Self::fields().prev.project_raw(prev);
            (*self).prev.link(prev);
        }

        pub unsafe fn insert_next(self: *const Self, next: *const Self) {
            let fields = Self::fields();
            if !(*self).next.get().is_null() {
                let self_next = (*self).next.get();
                fields.next.inverse_project_raw(self_next).link_prev(next);
            }
            self.link_next(next);
        }

        pub unsafe fn insert_prev(self: *const Self, prev: *const Self) {
            let fields = Self::fields();
            if !(*self).prev.get().is_null() {
                let self_prev = (*self).prev.get();
                fields.prev.inverse_project_raw(self_prev).link_next(prev);
            }
            self.link_prev(prev);
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
                    prev.link_next(next);
                }
                if !next.is_null() {
                    next.link_prev(prev);
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

    unsafe fn link_next(&self, next: Option<&Self>) {
        let next = core::mem::transmute::<_, *const Self>(next);
        let link = Self::fields().link;
        let self_next = self.link.next();
        if !self_next.is_null() {
            self_next.link_prev(core::ptr::null());
        }
        link.project_raw(self).link_next(link.project_raw(next));
    }

    unsafe fn link_prev(&self, prev: Option<&Self>) {
        let prev = core::mem::transmute::<_, *const Self>(prev);
        let link = Self::fields().link;
        let self_prev = self.link.prev();
        if !self_prev.is_null() {
            self_prev.link_prev(core::ptr::null());
        }
        link.project_raw(self).link_prev(link.project_raw(prev))
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
        foo.link_next(Some(&yam));
        foo.link_next(Some(&bar));

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
