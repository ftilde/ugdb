use super::{
    Window,
};
use std::cmp::max;

pub trait Widget {
    fn space_demand(&self) -> (Demand, Demand);
    fn draw(&mut self, window: Window);
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

impl Demand {
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

    pub fn max(self, other: Self) -> Self {
        Demand {
            min: max(self.min, other.min),
            max: if let (Some(l), Some(r)) = (self.max, other.max) {
                Some(max(l, r))
            } else {
                None
            }
        }
    }
}
