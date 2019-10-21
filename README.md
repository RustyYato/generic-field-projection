# generic-field-projection

This is a general purpose projection library that doesn't special case any pointer type except for
raw pointers. This makes the UX for all smart pointers the same, and more consistent.

This workspace is a proof of concept for the pointer to field concept discussed in this [Rust Internals](https://internals.rust-lang.org/t/idea-pointer-to-field/10061) discussion, and the [RFC](https://github.com/rust-lang/rfcs/pull/2708).

Disclaimer: This workspace does not intend to be a fully fledged implementation, and will likely be missing many key features to making the pointer to field idea truly work.

Basic sketch of how this workspace workspace

```rust
#[derive(Field)]     // This derive is given in the ptr-to-field-macro crate
struct Foo {
    pub names: String,
    bar: Bar,
}
```

// TODO: Finish README

# Contributing

See CONTRIBUTING.md
