use std::ops::{Add, Div, Mul, Sub};

/// trait to encompass basic arithmetic operations
pub trait Numeric<T>:
    Clone + Copy + Add<T, Output = T> + Div<T, Output = T> + Mul<T, Output = T> + Sub<T, Output = T>
{
}

/// implement Numeric<T> for all T satisfying the basic arithmetic operations.
impl<
        T: Clone
            + Copy
            + Add<T, Output = T>
            + Div<T, Output = T>
            + Mul<T, Output = T>
            + Sub<T, Output = T>,
    > Numeric<T> for T
{
}

/// generic point type that supports adding, subtracting, multiplying, and dividing points
/// as well as scaling points by a provided T.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point<T>
where
    T: Numeric<T>,
{
    pub x: T,
    pub y: T,
}

impl Point<f32> {
    pub fn lerp(self, other: Point<f32>, amount: f32) -> Point<f32> {
        Point::new(
            self.x * amount + other.x * (1.0 - amount),
            self.y * amount + other.y * (1.0 - amount),
        )
    }
}

impl Point<f64> {
    pub fn lerp(self, other: Point<f64>, amount: f64) -> Point<f64> {
        Point::new(
            self.x * amount + other.x * (1.0 - amount),
            self.y * amount + other.y * (1.0 - amount),
        )
    }
}

impl<T> Point<T>
where
    T: Numeric<T>,
{
    pub const fn new(x: T, y: T) -> Point<T> {
        Point { x, y }
    }
}

impl<T> Add<Point<T>> for Point<T>
where
    T: Numeric<T>,
{
    type Output = Point<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Sub<Point<T>> for Point<T>
where
    T: Numeric<T>,
{
    type Output = Point<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> Mul<T> for Point<T>
where
    T: Numeric<T>,
{
    type Output = Point<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> Mul<Point<T>> for Point<T>
where
    T: Numeric<T>,
{
    type Output = Point<T>;

    fn mul(self, rhs: Point<T>) -> Self::Output {
        Point {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<T> Div<T> for Point<T>
where
    T: Numeric<T>,
{
    type Output = Point<T>;

    fn div(self, rhs: T) -> Self::Output {
        Point {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T> Div<Point<T>> for Point<T>
where
    T: Numeric<T>,
{
    type Output = Point<T>;

    fn div(self, rhs: Point<T>) -> Self::Output {
        Point {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl<T, U> From<(Point<T>,)> for Point<U>
where
    U: From<T> + Numeric<U>,
    T: Numeric<T>,
{
    fn from(value: (Point<T>,)) -> Self {
        Point::new(value.0.x.into(), value.0.y.into())
    }
}
