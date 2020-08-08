use core::cell::{RefCell, RefMut};
use core::mem::MaybeUninit;
use core::ops::Drop;
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::data_array;

#[derive(Debug)]
pub struct Rack<T> {
    // All the stored units are kept inside `RefCell` to allow us to keep a
    // mutable reference to the data in multiple `Unit`s while keeping the
    // `Rack` immutable. That way we avoid issues with borrow checking.
    // The carried type is then enclosed in `MaybeUnit`, the reason for that we
    // don't need to require carried type to implement `Copy` and `Default` to
    // populate the whole array during `Rack`'s initialization.
    data: [RefCell<MaybeUninit<T>>; 2],
}

impl<T> Rack<T> {
    pub fn new() -> Self {
        Self {
            data: data_array::init_2(),
        }
    }

    pub fn add(&self, value: T) -> Unit<T> {
        for cell in self.data.iter() {
            // If we can borrow it, nobody has a mutable reference, it is free
            // to take.
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

impl<T> Default for Rack<T> {
    fn default() -> Self {
        Self::new()
    }
}

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
        let _rack: Rack<_> = Rack::<i32>::new();
    }

    #[test]
    fn add_unit_to_rack() {
        let rack = Rack::<i32>::new();

        let _unit: Unit<_> = rack.add(10);
    }

    #[test]
    fn get_immutable_reference_to_unit_value() {
        let rack = Rack::new();

        let unit = rack.add(10);

        assert_eq!(*unit.get_ref(), 10);
    }

    #[test]
    fn get_multiple_immutable_references_to_unit_value() {
        let rack = Rack::new();

        let unit = rack.add(10);

        let ref_1 = unit.get_ref();
        let ref_2 = unit.get_ref();

        assert_eq!(ref_1, ref_2);
    }

    #[test]
    fn get_mutable_reference_to_unit_value() {
        let rack = Rack::new();

        let mut unit = rack.add(10);

        assert_eq!(*unit.get_mut(), 10);
    }

    #[test]
    fn access_unit_value_by_dereferencing() {
        let rack = Rack::new();

        let unit = rack.add(10);

        assert_eq!(*unit, 10);
    }

    #[test]
    fn pass_immutable_unit_by_deref_coercion() {
        fn assert_ref_i32_eq_10(num: &i32) {
            assert_eq!(*num, 10)
        }

        let rack = Rack::new();

        let unit = rack.add(10);

        assert_ref_i32_eq_10(&unit)
    }

    #[test]
    fn change_unit_value_through_mutable_reference() {
        let rack = Rack::new();

        let mut unit = rack.add(10);

        let mut_ref = unit.get_mut();
        *mut_ref = 20;

        assert_eq!(*unit.get_ref(), 20);
    }

    #[test]
    fn change_unit_struct_field_through_mutable_reference() {
        struct Foo(i32);

        let rack = Rack::new();

        let mut unit = rack.add(Foo(10));

        let mut_ref = unit.get_mut();
        mut_ref.0 = 20;

        assert_eq!(unit.get_ref().0, 20);
    }

    #[test]
    fn change_unit_value_by_mutable_dereferencing() {
        let rack = Rack::new();

        let mut unit = rack.add(10);
        *unit = 20;

        assert_eq!(*unit.get_ref(), 20);
    }

    #[test]
    fn pass_mutable_unit_by_deref_coercion() {
        fn assert_mut_ref_i32_editable(num: &mut i32) {
            *num = 20;
            assert_eq!(*num, 20)
        }

        let rack = Rack::new();

        let mut unit = rack.add(10);

        assert_mut_ref_i32_editable(&mut unit)
    }

    #[test]
    fn accept_up_to_the_limit() {
        let rack = Rack::new();

        let _unit1 = rack.add(10);
        let _unit2 = rack.add(20);
    }

    #[test]
    #[should_panic(expected = "The rack is full")]
    fn reject_over_the_limit() {
        let rack = Rack::new();

        let _unit1 = rack.add(10);
        let _unit2 = rack.add(20);
        let _unit3 = rack.add(30);
    }

    #[test]
    fn accept_more_units_once_old_ones_get_out_of_scope() {
        let rack = Rack::new();

        let _unit1 = rack.add(10);
        {
            let _unit2 = rack.add(20);
        }
        let _unit3 = rack.add(30);
    }
}
