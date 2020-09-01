//! A Rust library providing memory allocation on the stack.
//!
//! # Initializing memory
//!
//! In order to allocate values on the stack, [`Rack`](trait.Rack.html) needs to
//! be initialized first. A `Rack` is initialized with a type of values it can
//! store and with a maximum number of values it can store. The `Rack` will
//! occupy its full size in the memory, so choose the capacity wisely. Unlike
//! [`Box`](https://doc.rust-lang.org/std/boxed/index.html), a `Rack` can store
//! only values of a single type. In case you want to store different types,
//! define multiple instances of `Rack`. There are several implementations of
//! `Rack` available with capacities of powers of 2, up to 1024:
//! [`Rack1`](struct.Rack1.html), [`Rack2`](struct.Rack2.html),
//! [`Rack4`](struct.Rack4.html), [`Rack8`](struct.Rack8.html),
//! [`Rack16`](struct.Rack16.html), [`Rack32`](struct.Rack32.html), ... ,
//! [`Rack1024`](struct.Rack1024.html).
//!
//! Learn more in the [documentation of the Rack trait](trait.Rack.html).
//!
//! # Storing and accessing values
//!
//! After the `Rack` is initalized, it is possible to store values on it. When a
//! value is stored, a [`Unit`](struct.Unit.html) struct is returned. A `Unit`
//! provides an ownership of the value. Moreover, the value can be accessed
//! through it, both mutably and immutably. Once `Unit` gets out of scope, it
//! will make sure that the stored value gets dropped.
//!
//! Learn more in the [documentation of the Unit struct](struct.Unit.html).
//!
//! # Examples
//!
//! Store a numeric value on the `Rack` and access it through the `Unit`:
//!
//! ```
//! # use heapnotize::*;
//! let rack = Rack64::new();
//! let five = rack.must_add(5);
//! assert_eq!(*five, 5);
//! ```
//! Use `Unit` to compose a recursive type:
//!
//! ```
//! # use heapnotize::*;
//! enum List<'a> {
//!     Cons(i32, Unit<'a, List<'a>>),
//!     Nil,
//! }
//!
//! use List::{Cons, Nil};
//!
//! let rack = Rack64::new();
//! let list = Cons(1, rack.must_add(Cons(2, rack.must_add(Cons(3, rack.must_add(Nil))))));
//! ```
//!
//! See more examples in the documentation of the [`Rack`](trait.Rack.html)
//! trait and the [`Unit`](struct.Unit.html) struct.

#![no_std]

mod data_array;

use core::cell::{RefCell, RefMut};
use core::fmt;
use core::mem::MaybeUninit;
use core::ops::Drop;
use core::ops::{Deref, DerefMut};
use core::ptr;

/// An enumeration of possible errors which can happen when adding a new value
/// to a [Rack](trait.Rack.html).
#[derive(Debug)]
pub enum AddUnitError {
    /// The [Rack](trait.Rack.html) is on its full capacity and cannot accept
    /// more values.
    FullRack,
}

impl fmt::Display for AddUnitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::FullRack => write!(f, "the rack is full"),
        }
    }
}

/// A trait specifying functions and methods for initialization of a `Rack` and
/// for storing values in it.
///
/// # Capacity
///
/// A `Rack` keep an allocated memory on the stack for values to be stored in.
/// It has several implementations varying in the capacity they provide:
/// [`Rack1`](struct.Rack1.html), [`Rack2`](struct.Rack2.html),
/// [`Rack4`](struct.Rack4.html), [`Rack8`](struct.Rack8.html),
/// [`Rack16`](struct.Rack16.html), [`Rack32`](struct.Rack32.html), ... ,
/// [`Rack1024`](struct.Rack1024.html).
///
/// # Stored type
///
/// It can store only a single type of values it is initialized with. The type
/// can be specified during initialization `Rack64::<i32>`, but Rust is usually
/// able to deduce the type on its own based on the code adding values to the
/// `Rack`.
///
/// # Memory requirements
///
/// Unlike a basic array, `Rack` is not zero-cost when it comes to memory
/// requirements. The formula for the memory requirements of a rack is
/// following:
///
/// **`capacity_of_the_rack * (round_up_to_the_closest_multiple_of_8(size_of(value)) + 8)`**
pub trait Rack<T> {
    /// Add a value to the `Rack` and return an error if it is full.
    ///
    /// # Errors
    ///
    /// This method will return an error in case the `Rack` is fully populated.
    /// If you don't expect it to ever fail, use
    /// [`must_add`](trait.Rack.html#tymethod.must_add) instead.
    ///
    /// # Examples
    ///
    /// Initialize the Rack and add an integer to it. Notice that since Rust can
    /// deduce the `T` of `Rack<T>` based on the value in `add`, there is no
    /// need to specify the type during the initialization:
    ///
    /// ```
    /// # use heapnotize::*;
    /// let rack = Rack64::new();
    /// let five = rack.must_add(5);
    /// ```
    fn add(&self, value: T) -> Result<Unit<T>, AddUnitError>;

    /// Add a value to the `Rack` and panic if it is full.
    ///
    /// # Panics
    ///
    /// This method will panic in case the `Rack` is fully populated. If you
    /// would rather receive an error, use [`add`](trait.Rack.html#tymethod.add)
    /// instead.
    ///
    /// # Examples
    ///
    /// Initialize the Rack and add an integer to it. Notice that since Rust can
    /// deduce the `T` of `Rack<T>` based on the value in `add`, there is no
    /// need to specify the type during the initialization:
    ///
    /// ```
    /// # use heapnotize::*;
    /// let rack = Rack64::new();
    /// let five = rack.add(5).unwrap();
    /// ```
    fn must_add(&self, value: T) -> Unit<T>;
}

macro_rules! rack {
    ($name:ident, $size:expr, $data_initializer:expr) => {
        /// Implementation of [`Rack`](trait.Rack.html) trait holding up to N
        /// values of a type T.
        ///
        /// See more in the [documentation of the `Rack`](trait.Rack.html) trait.
        pub struct $name<T> {
            // All the stored units are kept inside `RefCell` to allow us to
            // keep a mutable reference to the data in multiple `Unit`s while
            // keeping the `Rack` immutable. That way we avoid issues with
            // borrow checking. The carried type is then enclosed in
            // `MaybeUnit`, the reason for that we don't need to require carried
            // type to implement `Copy` and `Default` to populate the whole
            // array during `Rack`'s initialization.
            data: [RefCell<MaybeUninit<T>>; $size],
        }

        impl<T> $name<T> {
            /// Initialize a new Rack with a capacity based on the given implementation.
            ///
            /// # Examples
            ///
            /// Initialize a `Rack` holding up to 64 values of type `i32`:
            ///
            /// ```
            /// # use heapnotize::*;
            /// let rack = Rack64::<i32>::new();
            /// ```
            pub fn new() -> Self {
                Self {
                    data: $data_initializer,
                }
            }
        }

        impl<T> Rack<T> for $name<T> {
            fn add(&self, value: T) -> Result<Unit<T>, AddUnitError> {
                for cell in self.data.iter() {
                    // If we can borrow it, nobody has a mutable reference, it is free
                    // to take.
                    if cell.try_borrow().is_ok() {
                        cell.replace(MaybeUninit::new(value));
                        return Ok(Unit {
                            cell: cell.borrow_mut(),
                        });
                    }
                }
                Err(AddUnitError::FullRack)
            }

            fn must_add(&self, value: T) -> Unit<T> {
                self.add(value).expect("The rack is full")
            }
        }

        impl<T> Default for $name<T> {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}
rack!(Rack1, 1, data_array::init_1());
rack!(Rack2, 2, data_array::init_2());
rack!(Rack4, 4, data_array::init_4());
rack!(Rack8, 8, data_array::init_8());
rack!(Rack16, 16, data_array::init_16());
rack!(Rack32, 32, data_array::init_32());
rack!(Rack64, 64, data_array::init_64());
rack!(Rack128, 128, data_array::init_128());
rack!(Rack256, 256, data_array::init_256());
rack!(Rack512, 512, data_array::init_512());
rack!(Rack1024, 1024, data_array::init_1024());

/// A type serving as an owner of a value stored on the
/// [`Rack`](trait.Rack.html).
///
/// A `Unit` can be obtained by adding a value to the `Rack`. After that, it can
/// be used to access the value, both mutably and immutably. Once the `Unit`
/// gets out of the scope, the value that it holds gets dropped.
#[derive(Debug)]
pub struct Unit<'a, T> {
    cell: RefMut<'a, MaybeUninit<T>>,
}

impl<T> Unit<'_, T> {
    /// Get a reference to the data stored on the Rack.
    ///
    /// # Examples
    ///
    /// Reference to the stored value can be accessed using this method:
    ///
    /// ```
    /// # use heapnotize::*;
    /// let rack = Rack64::new();
    /// let five = rack.must_add(5);
    /// assert_eq!(*five.get_ref(), 5);
    /// ```
    ///
    /// The stored value can be also accessed using a dereference `*`:
    ///
    /// ```
    /// # use heapnotize::*;
    /// let rack = Rack64::new();
    /// let five = rack.must_add(5);
    /// assert_eq!(*five, 5);
    /// ```
    ///
    /// Finally, this allows users to use defer coercion and pass `&Unit<T>` to
    /// functions accepting `&T`:
    ///
    /// ```
    /// # use heapnotize::*;
    /// fn add_one(num: &i32) -> i32 {
    ///     num + 1
    /// }
    ///
    /// let rack = Rack64::new();
    /// let five = rack.must_add(5);
    ///
    /// assert_eq!(add_one(&five), 6)
    /// ```
    pub fn get_ref(&self) -> &T {
        // This code is safe since we always populate the `MaybeUninit` with a
        // value on `add` call before an `Unit` is returned.
        unsafe { &*self.cell.as_ptr() }
    }

    /// Get a mutable reference to the data stored on the Rack.
    ///
    /// # Examples
    ///
    /// Mutable reference to the stored value can be obtained using this method:
    ///
    /// ```
    /// # use heapnotize::*;
    /// let rack = Rack64::new();
    ///
    /// let mut number = rack.must_add(5);
    /// *number.get_mut() = 10;
    ///
    /// assert_eq!(*number.get_ref(), 10);
    /// ```
    ///
    /// The stored value can be also changed directly using a dereference `*`:
    ///
    /// ```
    /// # use heapnotize::*;
    /// let rack = Rack64::new();
    ///
    /// let mut number = rack.must_add(5);
    /// *number = 10;
    ///
    /// assert_eq!(*number, 10);
    /// ```
    ///
    /// Finally, this allows users to use defer coercion and pass `&mut Unit<T>`
    /// to functions accepting `&mut T`:
    ///
    /// ```
    /// # use heapnotize::*;
    /// fn set_to_ten(num: &mut i32) {
    ///     *num = 10;
    /// }
    ///
    /// let rack = Rack64::new();
    ///
    /// let mut number = rack.must_add(5);
    /// set_to_ten(&mut number);
    ///
    /// assert_eq!(*number, 10)
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        // This code is safe since we always populate the `MaybeUninit` with a
        // value on `add` call before an `Unit` is returned.
        unsafe { &mut *self.cell.as_mut_ptr() }
    }
}

/// When the Unit gets out of scope, it will deallocate its space on the Rack
/// and make sure that the stored value gets properly dropped.
// Unit's value is carried inside `MaybeUninit`. `Drop` on `MaybeUninit` does
// not do anything. Therefore, we have to implement the `Drop` trait, making
// sure that a destructor is called on the carried payload.
impl<T> Drop for Unit<'_, T> {
    fn drop(&mut self) {
        // This is safe since the Unit was the only owner of the stored data.
        unsafe {
            ptr::drop_in_place(self.cell.as_mut_ptr());
        }
    }
}

impl<T> Deref for Unit<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_ref()
    }
}

impl<T> DerefMut for Unit<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_rack() {
        let _rack: Rack2<_> = Rack2::<i32>::new();
    }

    #[test]
    fn add_unit_to_rack() {
        let rack = Rack2::<i32>::new();

        let _unit: Unit<_> = rack.must_add(10);
    }

    #[test]
    fn get_immutable_reference_to_unit_value() {
        let rack = Rack2::new();

        let unit = rack.must_add(10);

        assert_eq!(*unit.get_ref(), 10);
    }

    #[test]
    fn get_multiple_immutable_references_to_unit_value() {
        let rack = Rack2::new();

        let unit = rack.must_add(10);

        let ref_1 = unit.get_ref();
        let ref_2 = unit.get_ref();

        assert_eq!(ref_1, ref_2);
    }

    #[test]
    fn get_mutable_reference_to_unit_value() {
        let rack = Rack2::new();

        let mut unit = rack.must_add(10);

        assert_eq!(*unit.get_mut(), 10);
    }

    #[test]
    fn access_unit_value_by_dereferencing() {
        let rack = Rack2::new();

        let unit = rack.must_add(10);

        assert_eq!(*unit, 10);
    }

    #[test]
    fn pass_immutable_unit_by_deref_coercion() {
        fn assert_ref_i32_eq_10(num: &i32) {
            assert_eq!(*num, 10)
        }

        let rack = Rack2::new();

        let unit = rack.must_add(10);

        assert_ref_i32_eq_10(&unit)
    }

    #[test]
    fn change_unit_value_through_mutable_reference() {
        let rack = Rack2::new();

        let mut unit = rack.must_add(10);

        let mut_ref = unit.get_mut();
        *mut_ref = 20;

        assert_eq!(*unit.get_ref(), 20);
    }

    #[test]
    fn change_unit_struct_field_through_mutable_reference() {
        struct Foo(i32);

        let rack = Rack2::new();

        let mut unit = rack.must_add(Foo(10));

        let mut_ref = unit.get_mut();
        mut_ref.0 = 20;

        assert_eq!(unit.get_ref().0, 20);
    }

    #[test]
    fn change_unit_value_by_mutable_dereferencing() {
        let rack = Rack2::new();

        let mut unit = rack.must_add(10);
        *unit = 20;

        assert_eq!(*unit.get_ref(), 20);
    }

    #[test]
    fn pass_mutable_unit_by_deref_coercion() {
        fn assert_mut_ref_i32_editable(num: &mut i32) {
            *num = 20;
            assert_eq!(*num, 20)
        }

        let rack = Rack2::new();

        let mut unit = rack.must_add(10);

        assert_mut_ref_i32_editable(&mut unit)
    }

    #[test]
    fn accept_up_to_the_limit() {
        let rack = Rack2::new();

        let _unit1 = rack.must_add(10);
        let _unit2 = rack.must_add(20);
    }

    #[test]
    #[should_panic(expected = "The rack is full")]
    fn rejects_over_the_limit_with_panic_on_must_add() {
        let rack = Rack2::new();

        let _unit1 = rack.must_add(10);
        let _unit2 = rack.must_add(20);
        let _unit3 = rack.must_add(30);
    }

    #[test]
    fn rejects_over_the_limit_with_error_on_add() {
        let rack = Rack2::new();

        let _unit1 = rack.add(10).unwrap();
        let _unit2 = rack.add(20).unwrap();

        // Allow unreachable patterns in case more error types are added to
        // AddUnitError, so the match would panic on the default arm.
        #[allow(unreachable_patterns)]
        match rack
            .add(30)
            .expect_err("Add to full stack should return an error")
        {
            AddUnitError::FullRack => (),
            _ => panic!("Adding over limit returned unexpected error"),
        };
    }

    #[test]
    fn accept_more_units_once_old_ones_get_out_of_scope() {
        let rack = Rack2::new();

        let _unit1 = rack.must_add(10);
        {
            let _unit2 = rack.must_add(20);
        }
        let _unit3 = rack.must_add(30);
    }

    #[test]
    fn measure_memory_overhead_of_rack() {
        // Rounds up to 8 bytes and takes another 8 for MaybeUninit keept in
        // RefCell.
        // https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#layout

        use core::mem;

        fn round_up_to_8(x: usize) -> usize {
            x + 7 & !7
        }

        let item_size = mem::size_of::<[u8; 4]>();
        let rack_size = mem::size_of::<Rack2<[u8; 4]>>();

        assert_eq!(rack_size, 2 * (round_up_to_8(item_size) + 8));
    }

    #[test]
    #[allow(unused_variables)]
    fn exercise_basic_demo_from_readme() {
        fn main() {
            let rack = Rack64::new();
            let unit = rack.must_add(10);
            assert_eq!(*unit, 10);
        }

        main();
    }

    #[test]
    #[allow(unused_variables)]
    fn exercise_list_demo_from_readme() {
        enum List<'a> {
            Cons(i32, Unit<'a, List<'a>>),
            Nil,
        }

        use List::{Cons, Nil};

        fn main() {
            let rack = Rack64::new();
            let list = Cons(
                1,
                rack.must_add(Cons(2, rack.must_add(Cons(3, rack.must_add(Nil))))),
            );
        }

        main();
    }
}
