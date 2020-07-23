# heapnotize

Dynamic data allocation on the stack. That's right, no heap needed. Well, that
is a little stretch.

**Everything below this line is just cheap talk outlining the future
implementation, none of that is available.**

In fact, this allows you to dedicate parts of stack as storage for maximum `N`
of data types `T`.

What is this good for you ask. It allows you to live without heap, i.e. with
`#![no_std]` and thus help with memory management on microcontrollers. It may be
also useful for predictable memory requirements of your application.

Documentation:

* [API reference (docs.rs)]()
* [Analysis of the source code]()

## Usage

Add this to your `Cargo.toml`:

``` toml
[dependencies]
bitflags = "1.0"
```
