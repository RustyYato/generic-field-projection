// #![no_std]
#![feature(raw_ref_op)]
#![feature(arbitrary_self_types)]

pub mod linked_list {
    use core::{cell::Cell, ptr::NonNull};

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
    }
}

pub mod doubly_linked_list {
    use crate::linked_list::Link;
    use core::ptr::NonNull;
    use gfp_core::{Field, UncheckedInverseProjectTo, UncheckedProjectTo};

    #[derive(Field)]
    pub struct DoubleLink {
        next: Link,
        prev: Link,
    }

    #[allow(non_snake_case)]
    impl DoubleLink {
        pub fn new() -> Self {
            Self {
                next: Link::new(),
                prev: Link::new(),
            }
        }

        pub fn next(&self) -> Option<NonNull<DoubleLink>> {
            unsafe { self.next.get().inverse_project_to(Self::fields().next) }
        }

        pub fn prev(&self) -> Option<NonNull<DoubleLink>> {
            unsafe { self.prev.get().inverse_project_to(Self::fields().prev) }
        }

        pub unsafe fn unlink_next(&self) {
            if let Some(next) = self.next.get() {
                let next = next.inverse_project_to(Self::fields().next);
                next.as_ref().prev.unlink();
            }
            self.next.unlink();
        }

        pub unsafe fn unlink_prev(&self) {
            if let Some(prev) = self.prev.get() {
                let prev = prev.inverse_project_to(Self::fields().prev);
                prev.as_ref().prev.unlink();
            }
            self.prev.unlink();
        }

        pub unsafe fn link_next(self: *const Self, next: NonNull<Self>) {
            let DoubleLink = Self::fields();
            let this = NonNull::new_unchecked(self as *mut Self);
            next.as_ref().prev.link(this.project_to(DoubleLink.prev));
            let next = next.project_to(DoubleLink.next);
            (*self).next.link(next);
        }

        pub unsafe fn link_prev(self: *const Self, prev: NonNull<Self>) {
            let DoubleLink = Self::fields();
            let this = NonNull::new_unchecked(self as *mut Self);
            prev.as_ref().next.link(this.project_to(DoubleLink.next));
            let prev = prev.project_to(DoubleLink.prev);
            (*self).prev.link(prev);
        }

        pub unsafe fn insert_next(self: *const Self, next: NonNull<Self>) {
            let DoubleLink = Self::fields();

            if let Some(self_next) = (*self).next.get() {
                let this: *const Self =
                    self_next.inverse_project_to(DoubleLink.next).as_ptr();
                this.link_prev(next);
            }

            self.link_next(next);
        }

        pub unsafe fn insert_prev(self: *const Self, prev: NonNull<Self>) {
            let DoubleLink = Self::fields();

            if let Some(self_prev) = (*self).prev.get() {
                let this: *const Self =
                    self_prev.inverse_project_to(DoubleLink.prev).as_ptr();
                this.link_next(prev);
            }

            self.link_prev(prev);
        }

        pub unsafe fn remove(&self) {
            let DoubleLink = Self::fields();
            let next = self.next.get().inverse_project_to(DoubleLink.next);
            let prev = self.prev.get().inverse_project_to(DoubleLink.prev);

            match (next, prev) {
                (Some(next), Some(prev)) => {
                    (prev.as_ptr() as *const Self).link_next(next);
                    (next.as_ptr() as *const Self).link_prev(prev);
                },
                (Some(next), None) => next.as_ref().unlink_prev(),
                (None, Some(prev)) => prev.as_ref().unlink_next(),
                (None, None) => (),
            }

            self.next.unlink();
            self.prev.unlink();
        }
    }
}

use std::ptr::NonNull;

use doubly_linked_list::DoubleLink;
use gfp_core::{Field, UncheckedInverseProjectTo, UncheckedProjectTo};

#[derive(Field)]
pub struct Foo {
    x:    Box<i32>,
    y:    Box<i32>,
    link: DoubleLink,
}

#[allow(unused, non_snake_case)]
impl Foo {
    pub fn new() -> Self {
        Self {
            x:    Box::new(0),
            y:    Box::new(0),
            link: DoubleLink::new(),
        }
    }

    fn link(&self) -> NonNull<DoubleLink> {
        unsafe { NonNull::from(self).project_to(Self::fields().link) }
    }

    unsafe fn unlink_next(&self) {
        self.link.unlink_next()
    }

    unsafe fn unlink_prev(&self) {
        self.link.unlink_prev()
    }

    unsafe fn link_next(&self, next: &Self) {
        let Foo = Self::fields();
        let self_next = self.link.next();
        if let Some(next) = self.link.next() {
            next.as_ref().unlink_prev();
        }
        Foo.link
            .project_raw(self)
            .link_next(NonNull::from(next).project_to(Foo.link));
    }

    unsafe fn link_prev(&self, prev: &Self) {
        let Foo = Self::fields();
        let self_prev = self.link.prev();
        if let Some(prev) = self.link.prev() {
            prev.as_ref().unlink_next();
        }
        Foo.link
            .project_raw(self)
            .link_prev(NonNull::from(prev).project_to(Foo.link));
    }

    unsafe fn insert_next(&self, next: &Self) {
        let Foo = Self::fields();
        Foo.link
            .project_raw(self)
            .insert_next(NonNull::from(next).project_to(Foo.link))
    }

    unsafe fn insert_prev(&self, prev: &Self) {
        let Foo = Self::fields();
        Foo.link
            .project_raw(self)
            .insert_prev(NonNull::from(prev).project_to(Foo.link))
    }

    unsafe fn next(&self) -> Option<&Self> {
        self.link
            .next()
            .inverse_project_to(Foo::fields().link)
            .map(|foo| &*foo.as_ptr())
    }

    unsafe fn prev(&self) -> Option<&Self> {
        self.link
            .prev()
            .inverse_project_to(Foo::fields().link)
            .map(|foo| &*foo.as_ptr())
    }

    unsafe fn remove(&self) {
        self.link.remove();
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
        foo.link_next(&yam);
        foo.link_next(&bar);

        assert_eq!(foo.prev().map(Foo::get), None);
        assert_eq!(foo.next().map(Foo::get), Some((30, 40)));
        assert_eq!(bar.prev().map(Foo::get), Some((10, 20)));
        assert_eq!(bar.next().map(Foo::get), None);
        assert_eq!(yam.prev().map(Foo::get), None);
        assert_eq!(yam.next().map(Foo::get), None);

        foo.remove();

        assert_eq!(foo.prev().map(Foo::get), None);
        assert_eq!(foo.next().map(Foo::get), None);
        assert_eq!(bar.prev().map(Foo::get), None);
        assert_eq!(bar.next().map(Foo::get), None);
        assert_eq!(yam.prev().map(Foo::get), None);
        assert_eq!(yam.next().map(Foo::get), None);

        foo.insert_next(&yam);
        foo.insert_next(&bar);

        assert_eq!(foo.prev().map(Foo::get), None);
        assert_eq!(foo.next().map(Foo::get), Some((30, 40)));
        assert_eq!(bar.prev().map(Foo::get), Some((10, 20)));
        assert_eq!(bar.next().map(Foo::get), Some((50, 60)));
        assert_eq!(yam.prev().map(Foo::get), Some((30, 40)));
        assert_eq!(yam.next().map(Foo::get), None);

        bar.remove();

        assert_eq!(foo.prev().map(Foo::get), None);
        assert_eq!(foo.next().map(Foo::get), Some((50, 60)));
        assert_eq!(bar.prev().map(Foo::get), None);
        assert_eq!(bar.next().map(Foo::get), None);
        assert_eq!(yam.prev().map(Foo::get), Some((10, 20)));
        assert_eq!(yam.next().map(Foo::get), None);
    }
}
