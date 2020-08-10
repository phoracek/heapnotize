#![no_std]

mod data_array;
pub mod rack;

pub use rack::{AddUnitError, Rack, Unit};
pub use rack::{
    Rack1, Rack1024, Rack128, Rack16, Rack2, Rack256, Rack32, Rack4, Rack512, Rack64, Rack8,
};

#[cfg(test)]
mod tests {
    use crate::*;

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
