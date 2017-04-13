use std::ops::{
    Range,
    RangeFrom,
    RangeFull,
    RangeTo,
};

//TODO: Move to std traits and types once they are stabilized: https://github.com/rust-lang/rust/issues/30877
pub enum Bound<T> {
    Unbound,
    Inclusive(T),
    Exclusive(T),
}
pub trait RangeArgument<T> {
    fn start(&self) -> Bound<T>;
    fn end(&self) -> Bound<T>;
}

impl<T: Copy> RangeArgument<T> for Range<T> {
    fn start(&self) -> Bound<T> {
        Bound::Inclusive(self.start)
    }
    fn end(&self) -> Bound<T> {
        Bound::Exclusive(self.end)
    }
}
impl<T: Copy> RangeArgument<T> for RangeFrom<T> {
    fn start(&self) -> Bound<T> {
        Bound::Inclusive(self.start)
    }
    fn end(&self) -> Bound<T> {
        Bound::Unbound
    }
}
impl<T: Copy> RangeArgument<T> for RangeTo<T> {
    fn start(&self) -> Bound<T> {
        Bound::Unbound
    }
    fn end(&self) -> Bound<T> {
        Bound::Exclusive(self.end)
    }
}
impl<T> RangeArgument<T> for RangeFull {
    fn start(&self) -> Bound<T> {
        Bound::Unbound
    }
    fn end(&self) -> Bound<T> {
        Bound::Unbound
    }
}

