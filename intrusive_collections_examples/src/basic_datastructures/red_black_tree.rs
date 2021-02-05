//! ## Red-Black Tree
//!
//! This is an implementation of the red-black tree described originally
//! described in the paper:
//! ```bibtex
//! @INPROCEEDINGS{4567957,
//!   author={L. J. {Guibas} and R. {Sedgewick}},
//!   booktitle={19th Annual Symposium on Foundations of Computer Science (sfcs 1978)},
//!   title={A dichromatic framework for balanced trees},
//!   year={1978},
//!   volume={},
//!   number={},
//!   pages={8-21},
//!   abstract={In this paper we present a uniform framework for the
//!             implementation and study of balanced tree algorithms. We show
//!             how to imbed in this framework the best known balanced tree
//!             techniques and then use the  framework to develop new algorithms
//!             which perform the update and rebalancing in one pass, on the
//!             way down towards a leaf. We conclude  with a study of
//!             performance issues and concurrent updating.},
//!   keywords={Computer science;
//!             Petroleum;
//!             Particle measurements;
//!             Algorithm design and analysis;
//!             Performance analysis},
//!   doi={10.1109/SFCS.1978.3},
//!   ISSN={0272-5428},
//!   month={Oct},}
//! ```
//!
//! This implementation is based on the pseudo-code in the article that can be
//! found on
//! [Wikipedia.org](https://en.wikipedia.org/wiki/Red%E2%80%93black_tree).
//!
//! ## Implementation notes
//!
//! This implementation was designed for clarity, not for speed or other
//! optimizations.

use std::{
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    default::Default,
    fmt::Debug,
    hash::Hash,
    iter::Iterator,
    marker::PhantomPinned,
    ops::Drop,
    pin::Pin,
};

/// The colors that a red-black tree's node may take on.
///
/// The names of the colors are traditional, but arbitrary.  We could use any
/// names we wanted, provided the behaviors were the same.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RedBlackTreeNodeColor {
    Red,
    Black,
}

impl Default for RedBlackTreeNodeColor {
    /// By default, all `RedBlackTreeNodeColor` instances are black.
    fn default() -> Self {
        RedBlackTreeNodeColor::Black
    }
}

/// Our didactic example, a single node of a red-black tree.
///
/// The first thing to notice about this type is that there is no storage for
/// the thing that we want to store.  Normally, we'd expect a type similar to:
/// ```rust
/// pub struct OwningRedBlackTreeNode<T> {
///     object: T,
///     color: RedBlackTreeNodeColor,
///     parent: Option<*const OwningRedBlackTreeNode<T>>
///     left_child: Option<*const OwningRedBlackTreeNode<T>>,
///     right_child: Option<*const OwningRedBlackTreeNode<T>>
/// }
/// ```
/// which has storage for the type that we'd like to store.
///
/// An intrusive data structure flips this on it's head; each instance of `T`
/// owns a single node of the red-black tree.  So, given only a reference to a
/// `RedBlackTreeNode` node, how do you get back the `T` that it's a part of?
/// The trick is that the compiler knows how `T` is laid out in memory.  Given
/// a pointer to a `T`, we can calculate how many bytes there are to reach where
/// the `RedBlackTreeNode` node is laid out.  Here is a compressed example of
/// what we're talking about:
/// ```rust
/// #[derive(Debug)]
/// pub enum RedBlackTreeNodeColor {
///     Red,
///     Black,
/// }
///
/// impl Default for RedBlackTreeNodeColor {
///     fn default() -> Self {
///         RedBlackTreeNodeColor::Black
///     }
/// }
///
/// #[derive(Debug, Default)]
/// pub struct RedBlackTreeNode {
///     color: RedBlackTreeNodeColor,
///     parent: Option<*const RedBlackTreeNode>
///     left_child: Option<*const RedBlackTreeNode>,
///     right_child: Option<*const RedBlackTreeNode>
/// }
///
/// #[derive(Debug, Default)]
/// pub struct Thing {
///     field: Vec<u8>,
///     next: RedBlackTreeNode,
/// }
///
/// fn main() {
///     let thing = Thing::default();
///     let ptr1: *const Thing = &thing;
///
///     // We could use any field we wanted, we're using `parent` just as an
///     // example here.
///     let ptr2: *const RedBlackTreeNode = &(thing.parent);
///     let offset = unsafe { (ptr2 as *const u8).offset_from(ptr1 as *const u8) };
///     println!(
///         "The 'parent' field is offset from the start of 'thing' by {:?} bytes",
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
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RedBlackTreeNode<T>
where
    T: Debug + Clone + Default + PartialEq + Eq + PartialOrd + Ord + Hash,
{
    /// This is the key used to compare different nodes in the tree.  While it
    /// is possible to defer this to the object that owns the node, we're going
    /// to use the simpler route and store the key in here.
    key: T,

    /// All red-black tree nodes are colored either red or black.
    ///
    /// Our chosen default color is black, which means that when we create a
    /// single root node, it is a complete tree by itself.
    color: RedBlackTreeNodeColor,

    /// The node that is considered to be the parent of this node.  If there is
    /// no parent, then this will be `None`.
    parent: Option<*const RedBlackTreeNode<T>>,

    /// The node that is considered to be the left child of this node.  If there
    /// is no left child, then this will be `None`.
    left_child: Option<*const RedBlackTreeNode<T>>,

    /// The node that is considered to be the right child of this node.  If
    /// there is no right child, then this will be `None`.
    right_child: Option<*const RedBlackTreeNode<T>>,

    /// Since we're using raw pointers, we **cannot** allow these instances to
    /// be moved by the compiler (dropping is OK, but moving will almost
    /// certainly cause UB).  For more information on why this is necessary,
    /// start reading at
    /// https://doc.rust-lang.org/std/pin/index.html#example-self-referential-struct
    /// and be extra careful to read all the way through the explanations about
    /// drop!
    _pin: PhantomPinned,
}

impl<T> Drop for RedBlackTreeNode<T>
where
    T: Debug + Clone + Default + PartialEq + Eq + PartialOrd + Ord + Hash,
{
    /// Our custom Drop implementation to fix up our neighbors' pointers.
    ///
    /// When a node is dropped, it leaves a hole in the tree.  All of the node's
    /// neighbors (parents and children) will have pointers to a node that no
    /// longer exists.  We want to fix them up by making them forget about this
    /// node, if they haven't already forgotten about it.
    ///
    /// ## Safety
    ///
    /// This is potentially unsafe; there is no guarantee that when we traverse
    /// the pointers that they are actually pointing to valid memory.  Even if
    /// they *are* pointing to valid memory, there's no guarantee that we have a
    /// tree!  If the pointers actually form some kind of arbitrary
    /// non-tree-like graph, arbitrarily bad things can happen, which is why we
    /// need to use `unsafe` block below.  In addition, I'm casting away the
    /// mutability/immutability constraints, which can trick the compiler into
    /// doing very weird things.
    fn drop(&mut self) {
        unsafe {
            unsafe fn fixup_neighbor<T>(
                s: &RedBlackTreeNode<T>,
                neighbor: *mut RedBlackTreeNode<T>,
            ) where
                T: Debug
                    + Clone
                    + Default
                    + PartialEq
                    + Eq
                    + PartialOrd
                    + Ord
                    + Hash,
            {
                let self_pointer: *const RedBlackTreeNode<T> = s;
                if let Some(parent) = (*neighbor).parent {
                    if parent == self_pointer {
                        (*neighbor).parent = None;
                    }
                }

                if let Some(left_child) = (*neighbor).left_child {
                    if left_child == self_pointer {
                        (*neighbor).left_child = None;
                    }
                }

                if let Some(right_child) = (*neighbor).right_child {
                    if right_child == self_pointer {
                        (*neighbor).right_child = None;
                    }
                }
            }

            if let Some(parent) = self.parent {
                fixup_neighbor(self, parent as *mut RedBlackTreeNode<T>);
            }

            if let Some(left_child) = self.left_child {
                fixup_neighbor(self, left_child as *mut RedBlackTreeNode<T>);
            }

            if let Some(right_child) = self.right_child {
                fixup_neighbor(self, right_child as *mut RedBlackTreeNode<T>);
            }
        }
    }
}

impl<T> RedBlackTreeNode<T>
where
    T: Debug + Clone + Default + PartialEq + Eq + PartialOrd + Ord + Hash,
{
    /// Returns an immutable reference to the key in this node.
    ///
    /// ## Safety
    ///
    /// As long as `self` is a valid object, this is safe to do as the compiler
    /// will enforce the safety aspects.
    pub fn key(&self) -> &T {
        &self.key
    }

    /// Returns the current color of this node.
    ///
    /// ## Safety
    ///
    /// As long as `self` is a valid object, this is safe to do as the compiler
    /// will enforce the safety aspects.
    pub fn color(&self) -> RedBlackTreeNodeColor {
        self.color
    }

    /// Sets the color of this node.
    ///
    /// This is an internal method as arbitrary mutations will lead to arbitrary
    /// undefined behavior.
    ///
    /// ## Parameters
    ///
    /// - `new_color` - The new color you wish to set this node to.
    ///
    /// ## Returns
    ///
    /// The old color that this node used to have.
    ///
    /// ## Safety
    ///
    /// As long as `self` is a valid object, this is safe to do (in a very
    /// narrow sense) as the compiler will enforce the safety aspects.  The
    /// reason that it is in the 'narrow sense' is because while you won't
    /// violate any of rust's guarantees by using this method improperly, you
    /// can violate the invariants that a red-black tree requires.
    fn set_color(
        &mut self,
        new_color: RedBlackTreeNodeColor,
    ) -> RedBlackTreeNodeColor {
        let old_color = self.color;
        self.color = new_color;
        old_color
    }

    /// Produces a new iterator object for the singly linked list.
    ///
    /// This will produce a new iterator object for the list of
    /// `RedBlackTreeNode` instances.  Because of the reasons that were
    /// listed earlier, iterating over a singly linked list is always unsafe.
    pub unsafe fn iter(&self) -> RedBlackTreeNodeIterator<T> {
        todo!("Finish me!");
    }
}

/// A type that implements `std::iter::Iterator` for `RedBlackTreeNode`.
///
/// This type is an iterator of `RedBlackTreeNode` instances.  You may use
/// the `RedBlackTreeNode::iter` method to create an instance of this type,
/// which can then be iterated over.  E.g.:
///
/// ```rust
/// pub main() {
///     let mut a = RedBlackTreeNode::default();
///     let mut b = RedBlackTreeNode::default();
///     let mut c = RedBlackTreeNode::default();
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
pub struct RedBlackTreeNodeIterator<T>
where
    T: Debug + Clone + Default + PartialEq + Eq + PartialOrd + Ord + Hash,
{
    current: Option<*const RedBlackTreeNode<T>>,
}

impl<T> Iterator for RedBlackTreeNodeIterator<T>
where
    T: Debug + Clone + Default + PartialEq + Eq + PartialOrd + Ord + Hash,
{
    type Item = *const RedBlackTreeNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!("Finish me!");
    }
}
