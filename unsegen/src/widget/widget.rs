use base::{
    Window,
};
use base::basic_types::*;
use std::cmp::max;
use std::marker::PhantomData;
use std::ops:: {
    Add,
    AddAssign,
};
use std::iter::Sum;

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
pub struct Demand<T: AxisDimension> {
    pub min: PositiveAxisDiff<T>,
    pub max: Option<PositiveAxisDiff<T>>,
    _dim: PhantomData<T>,
}

impl<T: AxisDimension> Add<Demand<T>> for Demand<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Demand {
            min: self.min + rhs.min,
            max: if let (Some(l), Some(r)) = (self.max, rhs.max) {
                Some(l+r)
            } else {
                None
            },
            _dim: Default::default(),
        }
    }
}
impl<T: AxisDimension> AddAssign for Demand<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}
impl<T: AxisDimension + PartialOrd + Ord> Sum for Demand<T> {
    fn sum<I>(iter: I) -> Self where I: Iterator<Item=Self> {
        use ::std::ops::Add;
        iter.fold(Demand::exact(0), Demand::add)
    }
}
impl<'a, T: AxisDimension + PartialOrd + Ord> Sum<&'a Demand<T>> for Demand<T> {
    fn sum<I>(iter: I) -> Demand<T> where I: Iterator<Item=&'a Demand<T>> {
        iter.fold(Demand::zero(), |d1: Demand<T>, d2: &Demand<T>| d1 + *d2)
    }
}


impl<T: AxisDimension + PartialOrd + Ord> Demand<T> {
    pub fn zero() -> Self {
        Self::exact(0)
    }
    pub fn exact<I: Into<PositiveAxisDiff<T>> + Copy>(size: I) -> Self {
        Demand {
            min: size.into(),
            max: Some(size.into()),
            _dim: Default::default(),
        }
    }
    pub fn at_least<I: Into<PositiveAxisDiff<T>> + Copy>(size: I) -> Self {
        Demand {
            min: size.into(),
            max: None,
            _dim: Default::default(),
        }
    }
    pub fn from_to<I: Into<PositiveAxisDiff<T>> + Copy>(min: I, max: I) -> Self {
        debug_assert!(min.into() <= max.into(), "Invalid min/max");
        Demand {
            min: min.into(),
            max: Some(max.into()),
            _dim: Default::default(),
        }
    }

    pub fn max(&self, other: Self) -> Self {
        Demand {
            min: max(self.min, other.min),
            max: if let (Some(l), Some(r)) = (self.max, other.max) {
                Some(max(l, r))
            } else {
                None
            },
            _dim: Default::default(),
        }
    }

    pub fn max_assign(&mut self, other: Self) {
        *self = self.max(other);
    }
}

pub type ColDemand = Demand<ColDimension>;
pub type RowDemand = Demand<RowDimension>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Demand2D {
    pub width: ColDemand,
    pub height: RowDemand,
}
