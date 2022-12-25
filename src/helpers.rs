use std::iter::FusedIterator;

use bevy::prelude::Vec2;
use enum_iterator::Sequence;
use num_traits::{AsPrimitive, FromPrimitive, PrimInt};
use rstar::AABB;
use std::fmt::Debug;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Box1<T: PrimInt> {
    pub lo_incl: T,
    pub hi_excl: T,
}

impl<T: PrimInt> Box1<T> {
    pub fn size(self) -> T {
        self.hi_excl - self.lo_incl
    }

    pub fn new(lo_incl: T, hi_excl: T) -> Self {
        assert!(lo_incl <= hi_excl);
        Self { lo_incl, hi_excl }
    }

    pub fn from_point(x: T) -> Self {
        Self::new(x, x + T::one())
    }

    pub fn is_empty(self) -> bool {
        self.lo_incl == self.hi_excl
    }

    pub fn union_cover(self, other: Self) -> Self {
        Self::new(
            self.lo_incl.min(other.lo_incl),
            self.hi_excl.max(other.hi_excl),
        )
    }

    pub fn intersect(self, other: Self) -> Self {
        Self::new(
            self.lo_incl.max(other.lo_incl),
            self.hi_excl.min(other.hi_excl),
        )
    }

    pub fn intersects(self, other: Self) -> bool {
        self.hi_excl > other.lo_incl && self.lo_incl < other.hi_excl
    }

    pub fn contains(self, point: T) -> bool {
        self.lo_incl <= point && point < self.hi_excl
    }

    pub fn contains_box(self, other: Self) -> bool {
        self.lo_incl <= other.lo_incl && self.hi_excl >= other.hi_excl
    }

    pub fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut min = T::zero();
        let mut max = T::zero();
        for t in iter.into_iter() {
            min = min.min(t);
            max = max.max(t);
        }
        Self::new(min, max + T::one())
    }

    pub fn iter(self) -> Box1Iter<T> {
        Box1Iter {
            lo_incl: self.lo_incl,
            hi_excl: self.hi_excl,
        }
    }
}

impl<T: PrimInt + AsPrimitive<f32>> Box1<T> {
    pub fn center(self) -> f32 {
        self.lo_incl.as_() + self.size().as_() / 2.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SubtractResult<T: PrimInt> {
    None,
    One(Box1<T>),
    Two(Box1<T>, Box1<T>),
}

impl<T: PrimInt> Box1<T> {
    pub fn subtract(self, other: Self) -> SubtractResult<T> {
        if self.lo_incl >= other.lo_incl && self.hi_excl <= other.hi_excl {
            return SubtractResult::None;
        } else if self.lo_incl >= other.lo_incl {
            return SubtractResult::One(Box1::new(other.hi_excl, self.hi_excl));
        } else if self.hi_excl <= other.hi_excl {
            return SubtractResult::One(Box1::new(self.lo_incl, other.lo_incl));
        } else {
            return SubtractResult::Two(
                Box1::new(self.lo_incl, other.lo_incl),
                Box1::new(other.hi_excl, self.hi_excl),
            );
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Box1Iter<T: PrimInt> {
    pub lo_incl: T,
    pub hi_excl: T,
}

impl<T: PrimInt> Iterator for Box1Iter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.lo_incl == self.hi_excl {
            None
        } else {
            let lo = self.lo_incl;
            self.lo_incl = self.lo_incl + T::one();
            Some(lo)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = if self.hi_excl >= self.lo_incl {
            self.hi_excl - self.lo_incl
        } else {
            T::zero()
        };
        let size = size.to_usize().unwrap();
        (size, Some(size))
    }

    fn count(self) -> usize {
        let size = if self.hi_excl >= self.lo_incl {
            self.hi_excl - self.lo_incl
        } else {
            T::zero()
        };
        size.to_usize().unwrap()
    }

    fn nth(&mut self, n: usize) -> Option<T> {
        let lo = self.lo_incl + T::from(n)?;
        if lo < self.hi_excl {
            self.lo_incl = lo + T::one();
            Some(lo)
        } else {
            None
        }
    }
}

impl<T: PrimInt> FusedIterator for Box1Iter<T> {}

impl<T: PrimInt> ExactSizeIterator for Box1Iter<T> {}

impl<T: PrimInt> DoubleEndedIterator for Box1Iter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.hi_excl = self.hi_excl - T::one();
        (self.hi_excl > self.lo_incl).then_some(self.hi_excl)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Box2<T: PrimInt> {
    pub x: Box1<T>,
    pub y: Box1<T>,
}

impl<T: PrimInt> Box2<T> {
    pub fn size(self) -> T {
        self.x.size() * self.y.size()
    }

    pub fn new(lo_incl: (T, T), hi_excl: (T, T)) -> Self {
        Self {
            x: Box1::new(lo_incl.0, hi_excl.0),
            y: Box1::new(lo_incl.1, hi_excl.1),
        }
    }

    pub fn from_point(x: (T, T)) -> Self {
        Self {
            x: Box1::from_point(x.0),
            y: Box1::from_point(x.1),
        }
    }

    pub fn from_box1s(x: Box1<T>, y: Box1<T>) -> Self {
        Self { x, y }
    }

    pub fn is_empty(self) -> bool {
        self.x.is_empty() || self.y.is_empty()
    }

    pub fn union_cover(self, other: Self) -> Self {
        Self {
            x: self.x.union_cover(other.x),
            y: self.y.union_cover(other.y),
        }
    }

    pub fn intersect(self, other: Self) -> Self {
        Self {
            x: self.x.intersect(other.x),
            y: self.y.intersect(other.y),
        }
    }

    pub fn intersects(self, other: Self) -> bool {
        self.x.intersects(other.x) && self.y.intersects(other.y)
    }

    pub fn contains(self, point: (T, T)) -> bool {
        self.x.contains(point.0) && self.y.contains(point.1)
    }

    pub fn contains_box(self, other: Self) -> bool {
        self.x.contains_box(other.x) && self.y.contains_box(other.y)
    }

    pub fn from_iter<I: IntoIterator<Item = (T, T)>>(iter: I) -> Self {
        let mut min_x = T::zero();
        let mut max_x = T::zero();
        let mut min_y = T::zero();
        let mut max_y = T::zero();
        for (tx, ty) in iter.into_iter() {
            min_x = min_x.min(tx);
            max_x = max_x.max(tx);
            min_y = min_y.min(ty);
            max_y = max_y.max(ty);
        }
        Self::new((min_x, min_y), (max_x, max_y))
    }
}

impl<T: PrimInt + AsPrimitive<f32>> Box2<T> {
    pub fn center(self) -> Vec2 {
        Vec2::new(self.x.center(), self.y.center())
    }
}

pub fn n_to_bool(n: f64) -> bool {
    n < 0.5
}

pub fn n_to_enum<T: Sequence + FromPrimitive>(n: f64) -> T {
    T::from_u32(((T::CARDINALITY as f64) * n).floor() as u32).unwrap()
}

pub fn n_to_range<T: PrimInt + FromPrimitive>(n: f64, top: T) -> T {
    T::from_f64(n.floor()).unwrap() % top
}

pub fn n_to_box1<T: PrimInt + FromPrimitive>(n: f64, box1: Box1<T>) -> T {
    assert!(box1.size() > T::zero());
    let out = T::from_f64((n * box1.size().to_f64().unwrap()).floor()).unwrap() + box1.lo_incl;
    assert!(box1.contains(out));
    out
}

pub fn n_to_fitted_box1<T: PrimInt + FromPrimitive>(
    n: f64,
    size: T,
    range: Box1<T>,
) -> Option<Box1<T>> {
    if range.size() < size {
        return None;
    }
    let start = n_to_range(n, range.size() - size + T::one());
    assert!(
        range.lo_incl + start >= range.lo_incl && range.lo_incl + start + size <= range.hi_excl
    );
    Some(Box1::new(
        range.lo_incl + start,
        range.lo_incl + start + size,
    ))
}

impl From<Box1<i32>> for AABB<(i32,)> {
    fn from(b: Box1<i32>) -> Self {
        AABB::from_corners((b.lo_incl,), (b.hi_excl - 1,))
    }
}

impl From<Box2<i32>> for AABB<(i32, i32)> {
    fn from(b: Box2<i32>) -> Self {
        AABB::from_corners(
            (b.x.lo_incl, b.y.lo_incl),
            (b.x.hi_excl - 1, b.y.hi_excl - 1),
        )
    }
}
