use core::cell::{RefCell, RefMut};
use core::fmt;
use core::mem::MaybeUninit;
use core::ops::Drop;
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::data_array;

#[derive(Debug)]
pub enum AddUnitError {
    FullRack,
}

impl fmt::Display for AddUnitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::FullRack => write!(f, "the rack is full"),
        }
    }
}

pub trait Rack<T> {
    fn new() -> Self;
    fn add(&self, value: T) -> Result<Unit<T>, AddUnitError>;
    fn must_add(&self, value: T) -> Unit<T>;
}

macro_rules! rack {
    ($name:ident, $size:expr, $data_initializer:expr) => {
        pub struct $name<T> {
            // All the stored units are kept inside `RefCell` to allow us to keep a
            // mutable reference to the data in multiple `Unit`s while keeping the
            // `Rack` immutable. That way we avoid issues with borrow checking.
            // The carried type is then enclosed in `MaybeUnit`, the reason for that we
            // don't need to require carried type to implement `Copy` and `Default` to
            // populate the whole array during `Rack`'s initialization.
            data: [RefCell<MaybeUninit<T>>; $size],
        }

        impl<T> Rack<T> for $name<T> {
            fn new() -> Self {
                Self {
                    data: $data_initializer,
                }
            }

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

#[derive(Debug)]
pub struct Unit<'a, T> {
    cell: RefMut<'a, MaybeUninit<T>>,
}

impl<T> Unit<'_, T> {
    pub fn get_ref(&self) -> &T {
        // This code is safe since we always populate the `MaybeUninit` with a
        // value on `add` call before an `Unit` is returned.
        unsafe { &*self.cell.as_ptr() }
    }

    pub fn get_mut(&mut self) -> &mut T {
        // This code is safe since we always populate the `MaybeUninit` with a
        // value on `add` call before an `Unit` is returned.
        unsafe { &mut *self.cell.as_mut_ptr() }
    }
}

// The payload is carried inside `MaybeUninit`. `Drop` on `MaybeUninit` does not
// do anything. Therefore, we have to implement the `Drop` trait, making sure
// that a destructor is called on the carried payload.
impl<T> Drop for Unit<'_, T> {
    fn drop(&mut self) {
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
        let rack_size = mem::size_of::<Rack1<[u8; 4]>>();

        assert_eq!(rack_size, round_up_to_8(item_size) + 8);
    }
}
