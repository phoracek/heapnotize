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

First of all, allocate space on the stack for `N` items of your type `T`:

``` rust
let numbers = heapnotize::Rack::<i32, 16>::new()
```

Then we can add an item to this memory. The returned value will be a "pointer" on the stored value:

``` rust
let number_pointer = numbers.add(10) // Unit<i32>
```

The value can be accessed as a reference:

``` rust
let number_reference = number.as_ref() // &i32
```

A mutable reference:

``` rust
let number_mutable_reference = number.as_mut_ref() // &mut i32
```

We can also use dereference to move the value out of its unit:

``` rust
let number = *number // i32
```

When the value gets dereferenced or the unit gets out of scope, the memory will
be freed.

We can read capacity of the `Rack` and number of currently occupied slots:

``` rust
println!("numbers have currently occupied {} out of {} slots", numbers.used(), numbers.capacity())
```

