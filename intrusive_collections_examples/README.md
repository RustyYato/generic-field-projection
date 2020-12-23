# Introduction

This crate contains examples that may be of use in learning not only how to use
the [`generic-field-projection`](FIXME) create, but also make it clear as to why
that crate (and intrusive collections in general are useful).  The code in this
crate is not optimized; the main criteria for the implementations was for use as
teaching tools.  As a result, their performance is low.  We leave to others
(maybe you!) to improve the performance of this code.

## Intrusive Collections

Not everyone has heard of intrusive collections; if you haven't, then start
here, otherwise skip to the next section.

As programmers, a large part of our time is spent organizing data to make it
easy to access when we need it.  Because this is such a common problem, computer
scientists have developed a number of useful data structures, some of which can
be found in the rust standard library's
[`collections`](file:///home/cfkaran2/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/share/doc/rust/html/std/collections/index.html)
module.  The main thing to note about each of these types is that the collection
becomes the owner of the object that is placed within it.  *Objects are in the
container.*  (As a side-note, these types of collections are sometimes referred
to as *extrusive* collections to distinguish them from *intrusive* collections)

Intrusive collections are the exact opposite; *each object owns a portions of
the collection*.  If this seems confusing, consider how we could make a
linked list of `Things`.

#### Extrusive example

```rust
struct Thing {
    field: u8
}
```

The easiest way is to use the
[`LinkedList`](https://doc.rust-lang.org/std/collections/struct.LinkedList.html)
type, and transfer ownership of the set of objects to the linked list.  This is
the paradigm that most programmers are used to.  We assume that you're familiar
with this paradigm, so we won't talk about it any more.

#### Intrusive example

```rust
struct Thing<'a> {
    field: u8,
    next: Option<&'a Thing<'a>>
}
```

An alternative to the extrusive collection is the intrusive collection as
embodied in the example above.  In this case, the storage for the linked list is
held within each element (`Thing`); it's what the `next` pointer is.  This is
what I mean when I say that each object owns a portions of the collection; in
this case, `Thing` owns a portion of the linked list.

### Why they are useful

At this point, it doesn't look like they're very useful, so why do you care?

You care in two cases:

- Intrusive collections do not allocate memory, and so are useful in `#[no_std]`
environments.
- You want to compose collections with as little overhead as possible.

The former case can be important in embedded applications, as well as inside of
operating system kernels.

The latter case is less obvious.  Since rust collections types are effectively
templates, we know that creating a new
[`LinkedList`](https://doc.rust-lang.org/std/collections/struct.LinkedList.html)
for a `Thing` will produce code similar to the following:

```rust
struct LinkedListNode<'a> {
    field: Thing,
    prev: Option<&'a LinkedListNode<'a>>,
    next: Option<&'a LinkedListNode<'a>>
}
```

The only difference is the `prev` pointer to the previous node, but that's just
because
[`LinkedList`](https://doc.rust-lang.org/std/collections/struct.LinkedList.html)
is a [doubly-linked list](https://en.wikipedia.org/wiki/Doubly_linked_list).  If
it were a [singly-linked list](https://en.wikipedia.org/wiki/Linked_list), then
the storage would be exactly the same between the intrusive and extrusive
versions.  So why bother with intrusive versions?

The big reason is because of how easy it is to compose them into new, more
complex data structures.  As an example, consider making a [priority
queue](https://en.wikipedia.org/wiki/Priority_queue) where the priority of the
items can be modified while they are in the queue.  This operation makes certain
algorithms like [A\*](https://en.wikipedia.org/wiki/A*_search_algorithm) much
faster.  How can we do this?

One method is to use the standard collections
[`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) and
[`BinaryHeap`](https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html)
types together.  The `HashMap` makes it easy to find the object we're looking
for, and the `BinaryHeap` provides `log(N)` priority logic.  There's only two
problems; it doesn't work the way we want, and it's slow.

#### Why it doesn't work the way we want

What we want from our priority queue is *fast* access.  We want to be able to
find the object quickly (which is what the `HashMap` does), and we want to be
able to change its priority quickly.  We already have a slow method of doing
this; we can store the `Thing`s in an array, and sort the array each time we
insert a new `Thing`, or change a `Thing`'s priority.  This works, but is very,
very slow.  So let's try to build a new data structure that lets us do the
following:

- Find the `Thing` we want to update quickly.
- Update its priority quickly.
- Extract the next `Thing` quickly.

We already know that the collections types own whatever we put into them, so we
can't put `Thing` into the collection directly.  Fortunately, this isn't a big
deal; we'll use an `Rc` to share access between the collections.  So now our
collections are `HashMap<Rc<Thing>>` and `BinaryHeap<Rc<Thing>>`.  If we want to
modify the priority of the `Thing` with ID `36`, we can find the `Thing` in our
`HashMap` using `get_mut()` or some other method.  Great, we now have a
reference to our `Thing`; how do we get access to whatever is holding that
instance in our `BinaryHeap`?

It turns out that we can't.  We don't have a way of accessing the `BinaryHeap`'s
internal storage given just a `Thing`.  The only way we can modify a single
`Thing`'s priority is to create a brand-new `BinaryHeap` where we drain the old
heap and push all of the `Thing`s into the new heap, modifying our single,
solitary `Thing`'s priority along the way.

This is where intrusive collections can help.

#### How intrusive collections save the day (and why you need the `generic-field-projection` crate)

Earlier, I mentioned that in an intrusive collection, each object owns a portion
of the collection that they are a part of.  So for a mutable priority queue, we
might have something like the following:

```rust
use std::rc::{Rc, Weak};

pub struct BinaryTreeNode {
    key: usize,
    parent: Option<Weak<BinaryTreeNode>>,
    left_child: Option<Rc<BinaryTreeNode>>,
    right_child: Option<Rc<BinaryTreeNode>>
}

pub struct HeapTreeNode {
    priority: f64,
    parent: Option<Weak<HeapTreeNode>>,
    left_child: Option<Rc<HeapTreeNode>>,
    right_child: Option<Rc<HeapTreeNode>>
}

pub struct Thing<'a> {
    field: u8,
    priority_node: HeapTreeNode,
    key_node: BinaryTreeNode
}
```

(Normally `BinaryTreeNode` and `HeapTreeNode` would be in their own crates, but
I'm trying to keep the example compact)

Assume that we've got a properly formed data structure to start with, with the
`priority_node`s forming a
[heap](https://en.wikipedia.org/wiki/Heap_(data_structure)), and the `key_node`s
forming a [balanced binary
tree](https://en.wikipedia.org/wiki/Balanced_binary_tree).  Assuming that there
is a way to get the `Thing` that the `priority_node` or `key_node` is a part of
easily, then modifying the priority is simple:
1. Start at some `Thing`.  Using the `key_node` instance, traverse the parent
   and child links until you find the `Thing` that you're interested in.
2. Modify the priority in that `Thing`'s `priority_node`.
3. Shift the `Thing`'s `priority_node`'s parent and child nodes to ensure that
   the heap priority is maintained.

So how do we get the `Thing` give only the `priority_node` or the `key_node` of
the `Thing`?  If we were' programming in C or C++, we would use the
[`offset_of`](https://en.wikipedia.org/wiki/Offsetof) macro and do some pointer
math to find the `Thing` given the location of the node within it.  While we can
do that in Rust, pointer math is inherently an `unsafe` operation; do it wrong,
and you can have some very serious memory corruption issues.

##### `generic-field-projection` saves the day!

This is where the `generic-field-projection` crate saves the day; used
correctly, it allows you to 'project' pointers from objects like `Thing` above.
The pointer you get is actually a newly created instance of the `Field` type,
with its own lifetime, etc.  The `Field` type is in effect a smart pointer, with
interfaces that make it obvious when you're in safe territory, and when you're
not.

So, that's what this example crate is going to show; how to make intrusive
collections that use the `generic-field-projection` to gain access to the
objects that they are a part of.

### Why they are a bad idea

Intrusive collections aren't perfect; at the end of the day, you're going to
have to deal with some amount of unsafe code in order to make them work.  That
means that there is always a chance for the dreaded curse of Undefined Behavior.
`generic-field-projection` does what it can to warn you about what could be bad
ideas by marking various interfaces as `unsafe`, but it is up to you to use the
given interfaces correctly.

To give you an idea of how you could land in undefined behavior territory,
consider the following:

Assume you have a reference to a `Thing` that is part of a priority queue.  You
want to change the priority, and shift it around in the heap to maintain the
[heap property](https://en.wikipedia.org/wiki/Heap_(data_structure)).  That
involves the following steps:
1. Getting a reference to the `priority_node` in the `Thing`.  This is safe.
2. Traversing the parent or child pointers to some other `Thing`.  This is safe
   because `Rc` and `Weak` enforce the rules for you.
3. At each `priority_node` you wish to examine the `Thing` that contains the
   `priority_node`.  This is **unsafe.**

The reason that the last point is unsafe is because you don't know if the
`HeapTreeNode` you now have a reference to is actually a part of a `Thing`.
That means that it is possible that when you try to get what you think is the
owning `Thing` back, you've actually referenced (and read) arbitrary memory.
`generic-field-projection` can't protect against this, it can only warn you of
it with `unsafe` interfaces.

# Where to go from here

There are two high-level modules in this crate.  The first is the
`basic_datastructures` crate, and the second is the `lessons` crate.

The `basic_datastructures` crate has some very simple implementations of common
data structures ([singly linked
list](https://en.wikipedia.org/wiki/Linked_list), [binary
heap](https://en.wikipedia.org/wiki/Binary_heap), [red-black
tree](https://en.wikipedia.org/wiki/Red%E2%80%93black_tree)).  These are
deliberately designed to be as simple as is reasonably possible.  They are also
heavily documented to make them easy to understand.  They are not performance
optimized and are likely quite slow.  In addition, they are designed to panic
rather than return results; in short, don't use these types in production code.

The `lessons` crate is organized in what I thought would be a logical order from
simplest composition to most difficult.  Each module is prefixed with a two
digit value, so you can see the progression.  They are also heavily documented
to make it clearer as to what is happening.

# Comments, complaints, and issues

Please use the [issue
tracker](https://github.com/RustyYato/generic-field-projection/issues) to report
any issues you find within these examples.  Good luck, and we hope this is
helpful to you!