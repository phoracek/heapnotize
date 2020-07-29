#![no_std]

use core::cell::Ref;
use core::cell::RefCell;

#[derive(Debug)]
pub struct Rack {
    data: [(RefCell<bool>, RefCell<i32>); 2],
}

impl Rack {
    pub fn new() -> Self {
        Self {
            data: [
                (RefCell::new(false), RefCell::new(0)),
                (RefCell::new(false), RefCell::new(0)),
            ],
        }
    }

    pub fn add(&self, value: i32) -> Unit {
        for (taken, cell) in self.data.iter() {
            if !*taken.borrow() {
                taken.replace(true);
                cell.replace(value);
                return Unit {
                    value: cell.borrow(),
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
    value: Ref<'a, i32>,
}

impl Unit<'_> {
    pub fn value(&self) -> i32 {
        *self.value
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

    #[test]
    fn accept_more_units_once_old_ones_are_not_referenced_anymore() {
        panic!("Not implemented")
    }
}
