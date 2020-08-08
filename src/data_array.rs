use core::cell::RefCell;
use core::mem::MaybeUninit;

// TODO: Use "Constants in array repeat expressions" once it is implemented.
// https://github.com/rust-lang/rust/issues/49147

pub fn init_2<T>() -> [RefCell<MaybeUninit<T>>; 2] {
    [
        RefCell::new(MaybeUninit::uninit()),
        RefCell::new(MaybeUninit::uninit()),
    ]
}
