use nalgebra::Point2;
use nalgebra::Vector2;
use std::ops::{Add, Sub};

#[derive(Copy, Clone)]
pub enum Direction {
    Absolute,
    Normalized
}

#[derive(Clone)]
pub struct LineSegment {
    pub a: Point2<f32>,
    pub b: Point2<f32>
}
impl LineSegment {
    pub fn direction(&self, direction: Direction) -> Vector2<f32> {
        let dir = self.b.clone() - self.a.clone();
        match direction {
            Direction::Absolute => dir,
            Direction::Normalized => dir.normalize()
        }
    }

    pub fn normal(&self) -> Vector2<f32> {
        let direction = self.direction(Direction::Normalized);
        [-direction.y, direction.x].into()
    }

    pub fn intersection(
        &self,
        rhs: Self
    ) -> Option<Point2<f32>> {
        let (x1, x2, x3, x4) = (self.a.x, self.b.x, rhs.a.x, rhs.b.x);
        let (y1, y2, y3, y4) = (self.a.y, self.b.y, rhs.a.y, rhs.b.y);

        let determinant = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
        if determinant.abs() < 0.0001 {
            None
        } else {
            let (k1, k2) = (x1*y2 - y1*x2, x3 * y4 - y3 * x4);
            Some(
                [
                    (k1 * (x3 - x4) - k2 * (x1 - x2)) / determinant,
                    (k1 * (y3 - y4) - k2 * (y1 - y2)) / determinant
                ].into()
            )
        }
    }
}

impl Add for LineSegment {
    type Output = LineSegment;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            a: [self.a.x + rhs.a.x, self.a.y + rhs.a.y].into(),
            b: [self.b.x + rhs.b.x, self.b.y + rhs.b.y].into()
        }
    }
}

impl Add<Vector2<f32>> for LineSegment {
    type Output = LineSegment;

    fn add(self, rhs: Vector2<f32>) -> Self::Output {
        Self::Output {
            a: [self.a.x + rhs.x, self.a.y + rhs.y].into(),
            b: [self.b.x + rhs.x, self.b.y + rhs.y].into()
        }
    }
}

impl Sub for LineSegment {
    type Output = LineSegment;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            a: [self.a.x - rhs.a.x, self.a.y - rhs.a.y].into(),
            b: [self.b.x - rhs.b.x, self.b.y - rhs.b.y].into()
        }
    }
}

impl Sub<Vector2<f32>> for LineSegment {
    type Output = LineSegment;

    fn sub(self, rhs: Vector2<f32>) -> Self::Output {
        Self::Output {
            a: [self.a.x - rhs.x, self.a.y - rhs.y].into(),
            b: [self.b.x - rhs.x, self.b.y - rhs.y].into()
        }
    }
}

#[derive(Clone)]
pub struct PolySegment {
    pub c: LineSegment,
    pub u_aa: LineSegment,
    pub u: LineSegment,
    pub l_aa: LineSegment,
    pub l: LineSegment
}

impl PolySegment {
    pub fn new(center: &LineSegment, thickness: f32) -> Self {
        let nrm = center.normal();
        let half_thickness = thickness / 2.0;
        Self {
            c: center.clone(),
            u_aa: center.clone() + nrm.clone() * (half_thickness + 1.0),
            u: center.clone() + nrm.clone() * half_thickness,
            l_aa: center.clone() - nrm.clone() * (half_thickness + 1.0),
            l: center.clone() - nrm.clone() * half_thickness
        }
    }
}