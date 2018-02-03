use std::cmp::{
    Ordering,
};
use std::ops:: {
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Mul,
    Div,
    Rem,
    Neg,
};
use std::marker::PhantomData;
use std::iter::Sum;

/// ----------------------------------------------------------------------------
/// AxisIndex (aka ColIndex or RowIndex) ---------------------------------------
/// ----------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, Ord, Eq)]
pub struct AxisIndex<T: AxisDimension>{
    val: i32,
    _dim: PhantomData<T>,
}
impl<T: AxisDimension> AxisIndex<T> {
    pub fn new(v: i32) -> Self {
        AxisIndex {
            val: v,
            _dim: Default::default(),
        }
    }
    pub fn raw_value(self) -> i32 {
        self.into()
    }
    pub fn diff_to_origin(self) -> AxisDiff<T> {
        AxisDiff::new(self.val)
    }
    pub fn positive_or_zero(self) -> AxisIndex<T> {
        AxisIndex::new(self.val.max(0))
    }
}
impl<T: AxisDimension> From<i32> for AxisIndex<T> {
    fn from(v: i32) -> Self {
        AxisIndex::new(v)
    }
}
impl<T: AxisDimension> Into<i32> for AxisIndex<T> {
    fn into(self) -> i32 {
        self.val
    }
}
impl<T: AxisDimension> Into<isize> for AxisIndex<T> {
    fn into(self) -> isize {
        self.val as isize
    }
}
impl<T: AxisDimension, I: Into<AxisDiff<T>>> Add<I> for AxisIndex<T> {
    type Output = Self;
    fn add(self, rhs: I) -> Self {
        AxisIndex::new(self.val + rhs.into().val)
    }
}
impl<T: AxisDimension, I: Into<AxisDiff<T>>> AddAssign<I> for AxisIndex<T> {
    fn add_assign(&mut self, rhs: I) {
        *self = *self + rhs;
    }
}
impl<T: AxisDimension, I: Into<AxisDiff<T>>> Sub<I> for AxisIndex<T> {
    type Output = Self;
    fn sub(self, rhs: I) -> Self {
        AxisIndex::new(self.val - rhs.into().val)
    }
}
impl<T: AxisDimension, I: Into<AxisDiff<T>>> SubAssign<I> for AxisIndex<T> {
    fn sub_assign(&mut self, rhs: I) {
        *self = *self - rhs;
    }
}
impl<T: AxisDimension> Sub<Self> for AxisIndex<T> {
    type Output = AxisDiff<T>;
    fn sub(self, rhs: Self) -> Self::Output {
        AxisDiff::new(self.val - rhs.val)
    }
}
impl<T: AxisDimension, I: Into<AxisIndex<T>>> Rem<I> for AxisIndex<T> {
    type Output = Self;

    fn rem(self, modulus: I) -> Self {
        Self::new(self.val % modulus.into().val)
    }
}
impl<T: AxisDimension, I: Into<AxisIndex<T>> + Copy> PartialEq<I> for AxisIndex<T> {
    fn eq(&self, other: &I) -> bool {
        let copy = *other;
        self.val == copy.into().val
    }
}
impl<T: AxisDimension, I: Into<AxisIndex<T>> + Copy> PartialOrd<I> for AxisIndex<T> {
    fn partial_cmp(&self, other: &I) -> Option<Ordering> {
        let copy = *other;
        Some(self.val.cmp(&copy.into().val))
    }
}
impl<T: AxisDimension> Neg for AxisIndex<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        AxisIndex::new(-self.val)
    }
}

/// ----------------------------------------------------------------------------
/// AxisDiff (Difference between AxisIndex) ------------------------------------
/// ----------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, Ord, Eq)]
pub struct AxisDiff<T: AxisDimension>{
    val: i32,
    _dim: PhantomData<T>,
}
impl<T: AxisDimension> AxisDiff<T> {
    pub fn new(v: i32) -> Self {
        AxisDiff {
            val: v,
            _dim: Default::default(),
        }
    }
    pub fn from_origin(self) -> AxisIndex<T> {
        AxisIndex::new(self.val)
    }
    pub fn try_into_positive(self) -> Result<PositiveAxisDiff<T>, Self> {
        PositiveAxisDiff::new(self.val).map_err(|()| self)
    }
    pub fn abs(self) -> PositiveAxisDiff<T> {
        PositiveAxisDiff::new_unchecked(self.val.abs())
    }
    pub fn positive_or_zero(self) -> PositiveAxisDiff<T> {
        PositiveAxisDiff::new_unchecked(self.val.max(0))
    }
    pub fn raw_value(self) -> i32 {
        self.into()
    }
}
impl<T: AxisDimension> From<i32> for AxisDiff<T> {
    fn from(v: i32) -> Self {
        AxisDiff::new(v)
    }
}
impl<T: AxisDimension> Into<i32> for AxisDiff<T> {
    fn into(self) -> i32 {
        self.val
    }
}
impl<T: AxisDimension, I: Into<AxisDiff<T>>> Add<I> for AxisDiff<T> {
    type Output = Self;
    fn add(self, rhs: I) -> Self {
        AxisDiff::new(self.val + rhs.into().val)
    }
}
impl<T: AxisDimension, I: Into<AxisDiff<T>>> AddAssign<I> for AxisDiff<T> {
    fn add_assign(&mut self, rhs: I) {
        *self = *self + rhs;
    }
}
impl<T: AxisDimension> Mul<i32> for AxisDiff<T> {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        AxisDiff::new(self.val * rhs)
    }
}
impl<T: AxisDimension> Div<i32> for AxisDiff<T> {
    type Output = AxisDiff<T>;
    fn div(self, rhs: i32) -> Self::Output {
        AxisDiff::new(self.val / rhs)
    }
}
impl<T: AxisDimension, I: Into<AxisDiff<T>>> Sub<I> for AxisDiff<T> {
    type Output = Self;
    fn sub(self, rhs: I) -> Self {
        AxisDiff::new(self.val - rhs.into().val)
    }
}
impl<T: AxisDimension, I: Into<AxisDiff<T>>> SubAssign<I> for AxisDiff<T> {
    fn sub_assign(&mut self, rhs: I) {
        *self = *self - rhs;
    }
}
impl<T: AxisDimension, I: Into<AxisDiff<T>>> Rem<I> for AxisDiff<T> {
    type Output = Self;

    fn rem(self, modulus: I) -> Self {
        AxisDiff::new(self.val % modulus.into().val)
    }
}
impl<T: AxisDimension, I: Into<AxisDiff<T>> + Copy> PartialEq<I> for AxisDiff<T> {
    fn eq(&self, other: &I) -> bool {
        let copy = *other;
        self.val == copy.into().val
    }
}
impl<T: AxisDimension, I: Into<AxisDiff<T>> + Copy> PartialOrd<I> for AxisDiff<T> {
    fn partial_cmp(&self, other: &I) -> Option<Ordering> {
        let copy = *other;
        Some(self.val.cmp(&copy.into().val))
    }
}
impl<T: AxisDimension> Neg for AxisDiff<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        AxisDiff::new(-self.val)
    }
}


/// ----------------------------------------------------------------------------
/// PositiveAxisDiff (aka Width or Height) -------------------------------------
/// ----------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, Ord, Eq)]
pub struct PositiveAxisDiff<T: AxisDimension>{
    val: i32,
    _dim: PhantomData<T>,
}
impl<T: AxisDimension> PositiveAxisDiff<T> {
    fn new_unchecked(v: i32) -> Self {
        debug_assert!(v >= 0, "Invalid value for PositiveAxisDiff");
        PositiveAxisDiff {
            val: v,
            _dim: Default::default(),
        }
    }

    pub fn new(v: i32) -> Result<Self,()> {
        if v >= 0 {
            Ok(PositiveAxisDiff {
                val: v,
                _dim: Default::default(),
            })
        } else {
            Err(())
        }
    }
    pub fn from_origin(self) -> AxisIndex<T> {
        AxisIndex::new(self.val)
    }
    pub fn origin_range_contains(self, i: AxisIndex<T>) -> bool {
        0 <= i.val && i.val < self.val
    }

    pub fn to_signed(self) -> AxisDiff<T> {
        AxisDiff::new(self.val)
    }
    pub fn raw_value(self) -> i32 {
        self.into()
    }
}
impl<T: AxisDimension> Into<i32> for PositiveAxisDiff<T> {
    fn into(self) -> i32 {
        self.val
    }
}
impl<T: AxisDimension> Into<usize> for PositiveAxisDiff<T> {
    fn into(self) -> usize {
        self.val as usize
    }
}
impl<T: AxisDimension> Into<AxisDiff<T>> for PositiveAxisDiff<T> {
    fn into(self) -> AxisDiff<T> {
        AxisDiff::new(self.val)
    }
}
impl<T: AxisDimension, I: Into<PositiveAxisDiff<T>>> Add<I> for PositiveAxisDiff<T> {
    type Output = Self;
    fn add(self, rhs: I) -> Self {
        PositiveAxisDiff::new_unchecked(self.val + rhs.into().val)
    }
}
impl<T: AxisDimension, I: Into<PositiveAxisDiff<T>>> AddAssign<I> for PositiveAxisDiff<T> {
    fn add_assign(&mut self, rhs: I) {
        *self = *self + rhs;
    }
}
impl<T: AxisDimension> Mul<i32> for PositiveAxisDiff<T> {
    type Output = AxisDiff<T>;
    fn mul(self, rhs: i32) -> Self::Output {
        AxisDiff::new(self.val * rhs)
    }
}
impl<T: AxisDimension> Mul<usize> for PositiveAxisDiff<T> {
    type Output = Self;
    fn mul(self, rhs: usize) -> Self::Output {
        PositiveAxisDiff::new_unchecked(self.val * rhs as i32)
    }
}
impl<T: AxisDimension> Div<i32> for PositiveAxisDiff<T> {
    type Output = AxisDiff<T>;
    fn div(self, rhs: i32) -> Self::Output {
        AxisDiff::new(self.val / rhs)
    }
}
impl<T: AxisDimension> Div<usize> for PositiveAxisDiff<T> {
    type Output = Self;
    fn div(self, rhs: usize) -> Self::Output {
        PositiveAxisDiff::new_unchecked(self.val / rhs as i32)
    }
}
impl<T: AxisDimension, I: Into<AxisDiff<T>>> Sub<I> for PositiveAxisDiff<T> {
    type Output = AxisDiff<T>;
    fn sub(self, rhs: I) -> Self::Output {
        AxisDiff::new(self.val - rhs.into().val)
    }
}
impl<T: AxisDimension, I: Into<PositiveAxisDiff<T>>> Rem<I> for PositiveAxisDiff<T> {
    type Output = Self;
    fn rem(self, modulus: I) -> Self {
        PositiveAxisDiff::new_unchecked(self.val % modulus.into().val)
    }
}
impl<T: AxisDimension, I: Into<AxisDiff<T>> + Copy> PartialEq<I> for PositiveAxisDiff<T> {
    fn eq(&self, other: &I) -> bool {
        let copy = *other;
        self.val == copy.into().val
    }
}
impl<T: AxisDimension, I: Into<AxisDiff<T>> + Copy> PartialOrd<I> for PositiveAxisDiff<T> {
    fn partial_cmp(&self, other: &I) -> Option<Ordering> {
        let copy = *other;
        Some(self.val.cmp(&copy.into().val))
    }
}
impl<T: AxisDimension + PartialOrd + Ord> Sum for PositiveAxisDiff<T> {
    fn sum<I>(iter: I) -> Self where I: Iterator<Item=Self> {
        iter.fold(PositiveAxisDiff::new_unchecked(0), PositiveAxisDiff::add)
    }
}
impl<T: AxisDimension> From<usize> for PositiveAxisDiff<T> {
    fn from(v: usize) -> Self {
        assert!(v < i32::max_value() as usize, "Invalid PositiveAxisDiff value");
        PositiveAxisDiff::new_unchecked(v as i32)
    }
}


/// ----------------------------------------------------------------------------
/// Concrete type for concrete dimensions --------------------------------------
/// ----------------------------------------------------------------------------

pub trait AxisDimension : Copy { }

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RowDimension;
impl AxisDimension for RowDimension { }
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ColDimension;
impl AxisDimension for ColDimension { }


pub type RowIndex = AxisIndex<RowDimension>;
pub type RowDiff = AxisDiff<RowDimension>;
pub type Height = PositiveAxisDiff<RowDimension>;

pub type ColIndex = AxisIndex<ColDimension>;
pub type ColDiff = AxisDiff<ColDimension>;
pub type Width = PositiveAxisDiff<ColDimension>;
