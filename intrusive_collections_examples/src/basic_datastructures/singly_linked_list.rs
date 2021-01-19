//! ## Singly linked list
//!
//! This is an implementation of the canonical
//! [singly linked list](https://en.wikipedia.org/wiki/Singly_linked_list) that
//! you will find in any beginning programming course.  This implementation is
//! intended as a learning tool, and so the implementation is kept as simple as
//! possible.  One of the first thing the note about the implementation is that
//! there isn't any `special` node, here; all nodes are exactly the same.  That
//! means that optimizations such as storing the length of the list in a special
//! node, or maintaining a `first` or `last` node isn't possible.  It is
//! possible to build a more powerful linked list than this using various
//! techniques, but none of those will be done here.

use std::{iter::Iterator, rc::Rc};

/// Our didactic example, a single node of a singly linked list.
///
/// The first thing to notice about this type is that there is no storage for
/// the thing that we want to store.  Normally, we'd expect a type similar to:
/// ```rust
/// pub struct OwningSinglyLinkedList<T> {
///     object: T,
///     next: Option<*const OwningSinglyLinkedList<T>>
/// }
/// ```
/// which has storage for the type that we'd like to store.
///
/// An intrusive data structure flips this on it's head; each instance of `T`
/// owns a single node of the linked list.  So, given only a reference to a
/// `SinglyLinkedList` node, how do you get back the `T` that it's a part of?
/// The trick is that the compiler knows how `T` is laid out in memory.  Given
/// a pointer to a `T`, we can calculate how many bytes there are to reach where
/// the `SinglyLinkedList` node is laid out.  Here is a compressed example of
/// what we're talking about:
/// ```rust
/// #[derive(Debug, Default)]
/// pub struct SinglyLinkedList {
///     next: Option<*const SinglyLinkedList>,
/// }
///
/// #[derive(Debug, Default)]
/// pub struct Thing {
///     field: Vec<u8>,
///     next: SinglyLinkedList,
/// }
///
/// fn main() {
///     let thing = Thing::default();
///     let ptr1: *const Thing = &thing;
///     let ptr2: *const SinglyLinkedList = &(thing.next);
///     let offset = unsafe { (ptr2 as *const u8).offset_from(ptr1 as *const u8) };
///     println!(
///         "The 'next' field is offset from the start of 'thing' by {:?} bytes",
///         offset
///     );
///     assert!(offset >= 0);
///     let ptr3: *const Thing = unsafe { ((ptr2 as *const u8).sub(offset as usize)) as *const Thing };
///     assert_eq!(ptr1, ptr3);
/// }
/// ```
///
/// As you can see, we're doing a fair amount of casting, and there is some
/// `unsafe` code within the example.  This is a cost of intrusive collections,
/// and one that you should be aware of when implementing them.  Which brings up
/// the talk about safety...
///
/// ## Safety, or, how to shoot yourself in the foot
///
/// Intrusive collections are generally unsafe to work with.  They require
/// working with raw pointers instead of references, and this can't easily be
/// changed.  For a more in-depth explanation of why this is so, read the
/// [`README.md`](../README.md) file.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SinglyLinkedListNode {
    /// The pointer to the next node in the linked list.  When this is `Some`,
    /// there is another pointer to follow, when this is `None`, there is no
    /// next pointer to follow.
    next: Option<*const SinglyLinkedListNode>,
}

impl SinglyLinkedListNode {
    /// Sets the next pointer to `None`.
    ///
    /// This method is only useful in a few rare circumstances, but when you
    /// need it, you *really need it.*  The main use is when you're working on
    /// the head node of a list; you can't extract it, which means that without
    /// this method, it is impossible to remove it from the list it is a part
    /// of.
    ///
    /// ## Safety
    ///
    /// By the rules that govern rust, this method is safe.  **HOWEVER,** it is
    /// potentially powerful foot-gun.  If the only way you can access the
    /// elements that follow `self` is via `self.next`, then calling this method
    /// will mean that you no longer have access to the rest of the list.  Make
    /// sure that you are already holding the next object *before* you null out
    /// the next pointer!
    pub fn set_next_to_null(&mut self) {
        self.next = None;
    }

    /// Inserts `self` immediately after `previous`.
    ///
    /// If this method successfully completes, then `self` will be inserted into
    /// the linked list immediately after previous.  If there are detectable
    /// errors, then this will panic on the error.
    ///
    /// ## Parameters
    ///
    /// - `previous` - This is the node that will become `self`'s prior node. If
    ///   it is already pointing to something, then `self` will end up pointing
    ///   to that object, while `previous` will point to self.
    ///
    /// ## Safety
    ///
    /// This method is unsafe.  There is no proof that `previous` points to a
    /// valid memory location, it's just an assumption that we're forced to
    /// make. In addition, if `self` is already pointing to something, then that
    /// something will be dropped silently.  Finally, we assume that our singly
    /// linked list is not a circular list.  This method will permit you to
    /// create a circularly linked list, including make a node that links to
    /// itself.
    pub unsafe fn insert(&mut self, previous: *mut SinglyLinkedListNode) {
        self.next = (*previous).next;
        let me: *const SinglyLinkedListNode = self;
        (*previous).next = Some(me);
    }

    /// Extracts `self` from the linked list.
    ///
    /// This method extracts `self` from the linked list by setting `previous`
    /// to point to `self.next`, and then calling `self.set_next_to_null()`.
    /// The end result is that the linked list will no longer 'know' about
    /// `self`, and `self` will no longer 'know' about the linked list.
    ///
    /// ## Safety
    ///
    /// This method is unsafe because we're modifying a raw pointer whose
    /// provenance we know nothing about. If it is a bad pointer, then this will
    /// lead to undefined behavior.
    pub unsafe fn extract(&mut self, previous: *mut SinglyLinkedListNode) {
        if let Some(p) = (*previous).next {
            assert_eq!(
                self as *const SinglyLinkedListNode, p,
                "The 'previous' node doesn't actually point to this node.  \
                 That means that extracting `self` from here *will* cause a \
                 corruption in the linked list structure!"
            );
        }

        (*previous).next = self.next;
        self.set_next_to_null();
    }

    /// Produces a new iterator object for the singly linked list.
    ///
    /// This will produce a new iterator object for the list of
    /// `SinglyLinkedListNode` instances.  Because of the reasons that were
    /// listed earlier, iterating over a singly linked list is always unsafe.
    pub unsafe fn iter(&self) -> SinglyLinkedListNodeIterator {
        SinglyLinkedListNodeIterator {
            current: Some(self),
        }
    }
}

/// A type that implements `std::iter::Iterator` for `SinglyLinkedListNode`.
///
/// This type is an iterator of `SinglyLinkedListNode` instances.  You may use
/// the `SinglyLinkedListNode::iter` method to create an instance of this type,
/// which can then be iterated over.  E.g.:
///
/// ```rust
/// pub main() {
///     let mut a = SinglyLinkedListNode::default();
///     let mut b = SinglyLinkedListNode::default();
///     let mut c = SinglyLinkedListNode::default();
///
///     b.insert(&mut a);
///     c.insert(&mut b);
///
///     // We now have a small linked list `a -> b -> c`.
///     // We can now iterate over it.
///     unsafe {
///         for node in a.iter() {
///             // Pretty boring here, all we can do is iterate right now!
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SinglyLinkedListNodeIterator {
    current: Option<*const SinglyLinkedListNode>,
}

impl Iterator for SinglyLinkedListNodeIterator {
    type Item = *const SinglyLinkedListNode;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(c) => {
                self.current = unsafe { (*c).next };

                Some(c)
            },
            None => None,
        }
    }
}
