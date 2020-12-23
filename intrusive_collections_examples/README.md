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

That latter case is important for speed.

### Why they are a bad idea

Intrusive collections aren't perfect; they can cause weird bugs, and they can be
hard to reason about.  For example,