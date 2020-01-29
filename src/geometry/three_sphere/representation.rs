use na::{Quaternion, UnitQuaternion};

#[derive(Debug, Clone, Copy)]
pub struct Direction(pub Quaternion<f32>);
#[derive(Debug, Clone, Copy)]
pub struct Point(pub UnitQuaternion<f32>);

use std::ops::Deref;
impl Deref for Point {
    type Target = UnitQuaternion<f32>;
    fn deref(&self) -> &UnitQuaternion<f32> {
        &self.0
    }
}
impl From<Point> for Direction {
    fn from(p: Point) -> Direction {
        Direction(p.into_inner())
    }
}
impl From<&Point> for Direction {
    fn from(p: &Point) -> Direction {
        Direction(p.into_inner())
    }
}
impl Deref for Direction {
    type Target = Quaternion<f32>;
    fn deref(&self) -> &Quaternion<f32> {
        &self.0
    }
}

impl Direction {
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Direction {
        Direction(Quaternion::new(a, b, c, d))
    }

    pub fn one() -> Direction {
        Direction(Quaternion::new(1.0, 0.0, 0.0, 0.0))
    }
    pub fn i() -> Direction {
        Direction(Quaternion::new(0.0, 1.0, 0.0, 0.0))
    }
    pub fn j() -> Direction {
        Direction(Quaternion::new(0.0, 0.0, 1.0, 0.0))
    }
    pub fn k() -> Direction {
        Direction(Quaternion::new(0.0, 0.0, 0.0, 1.0))
    }

    pub fn length(&self) -> f32 {
        self.magnitude()
    }

    pub fn cross<'a, 'b, A, B>(&self, a: &'a A, b: &'b B) -> Direction
    where
        &'a A: Into<Direction>,
        &'b B: Into<Direction>,
    {
        let a: Direction = a.into();
        let b: Direction = b.into();
        use na::Matrix3;
        let mat_w = Matrix3::new(self.i, self.j, self.k, a.i, a.j, a.k, b.i, b.j, b.k);
        let mat_i = Matrix3::new(self.w, self.j, self.k, a.w, a.j, a.k, b.w, b.j, b.k);
        let mat_j = Matrix3::new(self.w, self.i, self.k, a.w, a.i, a.k, b.w, b.i, b.k);
        let mat_k = Matrix3::new(self.w, self.i, self.j, a.w, a.i, a.j, b.w, b.i, b.j);
        Direction::new(
            -mat_w.determinant(),
            mat_i.determinant(),
            -mat_j.determinant(),
            mat_k.determinant(),
        )
    }
}

pub const EPSILON: f32 = 0.00001;
impl Point {
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Point {
        Point(UnitQuaternion::new_normalize(Quaternion::new(a, b, c, d)))
    }
    pub fn one() -> Point {
        Point(UnitQuaternion::new_unchecked(Quaternion::new(
            1.0, 0.0, 0.0, 0.0,
        )))
    }
    pub fn i() -> Point {
        Point(UnitQuaternion::new_unchecked(Quaternion::new(
            0.0, 1.0, 0.0, 0.0,
        )))
    }
    pub fn j() -> Point {
        Point(UnitQuaternion::new_unchecked(Quaternion::new(
            0.0, 0.0, 1.0, 0.0,
        )))
    }
    pub fn k() -> Point {
        Point(UnitQuaternion::new_unchecked(Quaternion::new(
            0.0, 0.0, 0.0, 1.0,
        )))
    }
    pub fn in_direction(dir: Direction) -> Option<Point> {
        match UnitQuaternion::try_new(*dir, EPSILON) {
            Some(vector) => Some(Point(vector)),
            None => None,
        }
    }
    pub fn cross(&self, a: &Point, b: &Point) -> Direction {
        Direction::from(self).cross(a, b)
    }
    pub fn dir(&self) -> Direction {
        Direction::from(self)
    }
}

use std::ops::{Add, Mul, Neg, Sub};

impl Mul<f32> for Direction {
    type Output = Direction;
    fn mul(self, rhs: f32) -> Direction {
        Direction(rhs * self.0)
    }
}

impl Mul<Direction> for f32 {
    type Output = Direction;
    fn mul(self, rhs: Direction) -> Direction {
        Direction(self * *rhs)
    }
}

impl Neg for Direction {
    type Output = Direction;
    fn neg(self) -> Direction {
        Direction((*self).neg())
    }
}

impl Add<Direction> for Direction {
    type Output = Direction;
    fn add(self, rhs: Direction) -> Direction {
        Direction(*self + *rhs)
    }
}

impl Sub<Direction> for Direction {
    type Output = Direction;
    fn sub(self, rhs: Direction) -> Direction {
        Direction(*self - *rhs)
    }
}

impl Mul<Direction> for Direction {
    type Output = Direction;
    fn mul(self, rhs: Direction) -> Direction {
        Direction(*self * *rhs)
    }
}

impl Mul<Point> for Direction {
    type Output = Direction;
    fn mul(self, rhs: Point) -> Direction {
        Direction(*self * rhs.into_inner())
    }
}

impl Mul<Direction> for Point {
    type Output = Direction;
    fn mul(self, rhs: Direction) -> Direction {
        Direction(self.into_inner() * *rhs)
    }
}

impl Mul<Point> for Point {
    type Output = Point;
    fn mul(self, rhs: Point) -> Point {
        Point(*self * *rhs)
    }
}

impl Neg for Point {
    type Output = Point;
    fn neg(self) -> Point {
        Point((*self).neg())
    }
}

use std::fmt::{Display, Error, Formatter};
impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "Direction[{} + {}i + {}j + {}k]",
            self.w, self.i, self.j, self.k
        )?;
        Ok(())
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "Point[{} + {}i + {}j + {}k]",
            self.w, self.i, self.j, self.k
        )?;
        Ok(())
    }
}
