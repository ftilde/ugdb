use base::{
    Window,
};
use std::cmp::max;

pub trait Widget {
    fn space_demand(&self) -> Demand2D;
    fn draw(&self, window: Window, hints: RenderingHints);
}

#[derive(Clone, Copy, Debug)]
pub struct RenderingHints {
    pub active: bool,
}

impl Default for RenderingHints {
    fn default() -> Self {
        RenderingHints {
            active: false,
        }
    }
}

#[derive(Eq, PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Demand {
    pub min: u32,
    pub max: Option<u32>,
}

impl ::std::ops::Add<Demand> for Demand {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Demand {
            min: self.min + rhs.min,
            max: if let (Some(l), Some(r)) = (self.max, rhs.max) {
                Some(l+r)
            } else {
                None
            }
        }
    }
}
impl ::std::ops::AddAssign for Demand {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}
impl ::std::iter::Sum for Demand {
    fn sum<I>(iter: I) -> Self where I: Iterator<Item=Self> {
        use ::std::ops::Add;
        iter.fold(Demand::exact(0), Demand::add)
    }
}
impl<'a> ::std::iter::Sum<&'a Demand> for Demand {
    fn sum<I>(iter: I) -> Demand where I: Iterator<Item=&'a Demand> {
        iter.fold(Demand::zero(), |d1: Demand, d2: &Demand| d1 + *d2)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Demand2D {
    pub width: Demand,
    pub height: Demand,
}


impl Demand {
    pub fn zero() -> Self {
        Self::exact(0)
    }
    pub fn exact(size: u32) -> Self {
        Demand {
            min: size,
            max: Some(size),
        }
    }
    pub fn at_least(size: u32) -> Self {
        Demand {
            min: size,
            max: None,
        }
    }
    pub fn from_to(min: u32, max: u32) -> Self {
        debug_assert!(min <= max, "Invalid min/max");
        Demand {
            min: min,
            max: Some(max),
        }
    }

    pub fn max(&self, other: Self) -> Self {
        Demand {
            min: max(self.min, other.min),
            max: if let (Some(l), Some(r)) = (self.max, other.max) {
                Some(max(l, r))
            } else {
                None
            }
        }
    }

    pub fn max_assign(&mut self, other: Self) {
        *self = self.max(other);
    }
}
