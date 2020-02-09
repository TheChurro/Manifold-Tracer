use crate::geometry::three_sphere::representation::*;

pub struct GreatCircle {
    pub start: Point,
    pub tangent: Point,
}

pub struct GreatArc {
    pub start: Point,
    pub end: Point,
    pub tangent: Point,
    pub length: f32,
}

pub struct Triangle {
    pub verticies: [Point; 3],
    pub edge_normals: [Point; 3],
    pub triangle_normal: Point,
}

#[derive(Debug, Clone, Copy)]
pub enum TriangleError {
    DegenerateTriangle,
}
impl std::fmt::Display for TriangleError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use TriangleError::DegenerateTriangle;
        match self {
            DegenerateTriangle => {
                write!(fmt, "All verticies of triangle on one plane")?;
            }
        }
        Ok(())
    }
}

impl Triangle {
    pub fn new(a: Point, b: Point, c: Point) -> Result<Triangle, TriangleError> {
        use TriangleError::DegenerateTriangle;
        let normal = Point::in_direction(a.cross(&b, &c)).ok_or(DegenerateTriangle)?;
        let mut e_ab = Point::in_direction(normal.cross(&a, &b)).ok_or(DegenerateTriangle)?;
        let mut e_bc = Point::in_direction(normal.cross(&b, &c)).ok_or(DegenerateTriangle)?;
        let mut e_ca = Point::in_direction(normal.cross(&c, &a)).ok_or(DegenerateTriangle)?;
        if e_ab.dot(&c) < 0.0 {
            e_ab = -e_ab;
        }
        if e_bc.dot(&c) < 0.0 {
            e_bc = -e_bc;
        }
        if e_ca.dot(&c) < 0.0 {
            e_ca = -e_ca;
        }
        Ok(Triangle {
            verticies: [a, b, c],
            edge_normals: [e_ab, e_bc, e_ca],
            triangle_normal: normal,
        })
    }
}

pub struct Ball {
    pub center: Point,
    pub radius: f32,
}

impl Ball {
    pub fn new(center: Point, radius: f32) -> Ball {
        Ball {
            center: center,
            radius: radius,
        }
    }
}
