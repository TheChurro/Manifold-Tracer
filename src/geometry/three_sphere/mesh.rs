use crate::geometry::three_sphere::orientation::*;
use crate::geometry::three_sphere::primitives::*;
use crate::geometry::three_sphere::representation::*;
use na::Vector3;

pub struct MeshInstance {
    pub triangles: Vec<Triangle>,
}

pub struct Offset {
    tangent: Vector3<f32>,
    distance: f32,
}

impl Offset {
    pub fn orient(&self, orientation: &Orientation) -> Point {
        let tangent =
            orientation * Direction::new(0.0, self.tangent.x, self.tangent.y, self.tangent.z);
        let start = orientation * Direction::new(1.0, 0.0, 0.0, 0.0);
        let oriented_point =
            start.coords * self.distance.cos() + tangent.coords * self.distance.sin();
        Point::new(
            oriented_point.x,
            oriented_point.y,
            oriented_point.z,
            oriented_point.w,
        )
    }
}

pub struct MeshDescription {
    pub triangles: Vec<(usize, usize, usize)>,
    pub verticies_offsets: Vec<Offset>,
}

impl MeshDescription {
    pub fn instantiate(&self, oriented_position: Orientation) -> MeshInstance {
        let mut new_triangles = Vec::new();
        for &(a, b, c) in &self.triangles {
            if let Ok(triangle) = Triangle::new(
                self.verticies_offsets[a].orient(&oriented_position),
                self.verticies_offsets[b].orient(&oriented_position),
                self.verticies_offsets[c].orient(&oriented_position),
            ) {
                new_triangles.push(triangle);
            }
        }
        MeshInstance {
            triangles: new_triangles,
        }
    }
}
