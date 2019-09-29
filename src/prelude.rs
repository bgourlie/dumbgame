use custom_derive::custom_derive;
use nalgebra::{Point2, Vector2};
use newtype_derive::*;
use quicksilver::prelude::*;

type Point = Point2<f32>;
type Vec2 = Vector2<f32>;

custom_derive! {
    #[derive(
        Clone, Copy, Debug, PartialEq, NewtypeAdd, NewtypeAddAssign, NewtypeAdd(Vec2),
        NewtypeAddAssign(Vec2)
    )]
    pub struct Velocity(Vec2);
}

custom_derive! {
    #[derive(
        Clone, Copy, Debug, PartialEq, NewtypeAdd(Velocity), NewtypeAddAssign(Velocity),
        NewtypeFrom
    )]
    pub struct Position(Point);
}

impl Position {
    pub fn new(x: f32, y: f32) -> Position {
        Position(Point::new(x, y))
    }

    pub fn x(self) -> f32 {
        self.0.x
    }

    pub fn y(self) -> f32 {
        self.0.y
    }
}

impl From<Position> for Vector {
    fn from(pos: Position) -> Self {
        Vector::new(pos.x(), pos.y())
    }
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Velocity {
        Velocity(Vector2::new(x, y))
    }
}

impl std::ops::Add<Velocity> for Point {
    type Output = Point;

    fn add(self, rhs: Velocity) -> Self::Output {
        self.add(rhs.0)
    }
}

impl std::ops::AddAssign<Velocity> for Point {
    fn add_assign(&mut self, rhs: Velocity) {
        self.add_assign(rhs.0)
    }
}
