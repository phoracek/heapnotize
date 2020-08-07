#![no_std]

use core::cell::{RefCell, RefMut};
use core::mem::{self, MaybeUninit};
use core::ops::Drop;
use core::ptr;

#[derive(Debug)]
pub struct Rack {
    // All the stored units are kept inside `RefCell` to allow us to keep a
    // mutable reference to the data in multiple `Unit`s while keeping the
    // `Rack` immutable. That way we avoid issues with borrow checking.
    // The carried type is then enclosed in `MaybeUnit`, the reason for that we
    // don't need to require carried type to implement `Copy` and `Default` to
    // populate the whole array.
    data: [RefCell<MaybeUninit<i32>>; 2],
}

impl Rack {
    pub fn new() -> Self {
        Self {
            data: {
                // Create an uninitialized array of `MaybeUninit`. The `assume_init` is
                // safe because the type we are claiming to have initialized here is a
                // bunch of `MaybeUninit`s, which do not require initialization.
                let mut data: [MaybeUninit<RefCell<MaybeUninit<i32>>>; 2] =
                    unsafe { MaybeUninit::uninit().assume_init() };

                // Dropping a `MaybeUninit` does nothing. Thus using raw pointer
                // assignment instead of `ptr::write` does not cause the old
                // uninitialized value to be dropped. Also if there is a panic during
                // this loop, we have a memory leak, but there is no memory safety
                // issue.
                for elem in &mut data[..] {
                    *elem = MaybeUninit::new(RefCell::new(MaybeUninit::uninit()));
                }

                // Everything is initialized. Transmute the array to the
                // initialized type.
                unsafe { mem::transmute::<_, [RefCell<MaybeUninit<i32>>; 2]>(data) }
            },
        }
    }

    pub fn add(&self, value: i32) -> Unit {
        for cell in self.data.iter() {
            // If we can borrow it, nobody has a mutable reference, it is free to take
            if cell.try_borrow().is_ok() {
                cell.replace(MaybeUninit::new(value));
                return Unit {
                    cell: cell.borrow_mut(),
                };
            }
        }
        panic!("The rack is full");
    }
}

impl Default for Rack {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Unit<'a> {
    cell: RefMut<'a, MaybeUninit<i32>>,
}

impl Unit<'_> {
    pub fn value(&self) -> i32 {
        // This code is safe since we always populate the `MaybeUninit` with a
        // value on `add` call before an `Unit` is returned.
        unsafe { self.cell.assume_init() }
    }
}

impl Drop for Unit<'_> {
    fn drop(&mut self) {
        // The payload is carried inside `MaybeUninit`. `Drop` on `MaybeUninit`
        // does not do anything. Therefore, we have to implement the `Drop`
        // trait, making sure that a destructor is called on the carried
        // payload.
        unsafe {
            ptr::drop_in_place(self.cell.as_mut_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_rack() {
        let _rack: Rack = Rack::new();
    }

    #[test]
    fn add_unit_to_rack() {
        let rack = Rack::new();

        let _unit: Unit = rack.add(10);
    }

    #[test]
    fn get_unit_value() {
        let rack = Rack::new();

        let unit: Unit = rack.add(10);

        assert_eq!(unit.value(), 10)
    }

    #[test]
    fn accept_up_to_the_limit() {
        let rack = Rack::new();

        let _unit1: Unit = rack.add(10);
        let _unit2: Unit = rack.add(20);
    }

    #[test]
    #[should_panic(expected = "The rack is full")]
    fn reject_over_the_limit() {
        let rack = Rack::new();

        let _unit1: Unit = rack.add(10);
        let _unit2: Unit = rack.add(20);
        let _unit3: Unit = rack.add(30);
    }

    #[test]
    fn accept_more_units_once_old_ones_get_out_of_scope() {
        let rack = Rack::new();

        let _unit1: Unit = rack.add(10);
        {
            let _unit2: Unit = rack.add(20);
        }
        let _unit3: Unit = rack.add(30);
    }
}
