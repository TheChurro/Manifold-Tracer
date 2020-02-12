use crate::geometry::three_sphere::orientation::*;
use crate::geometry::three_sphere::primitives::*;
use crate::geometry::three_sphere::representation::*;

pub struct MeshInstance {
    pub triangles: Vec<Triangle>,
}

#[derive(Serialize, Deserialize)]
pub struct Offset {
    pub tangent: [f32; 3],
    pub distance: f32,
}

impl Offset {
    pub fn orient(&self, orientation: &Orientation) -> Point {
        let tangent =
            orientation * Direction::new(0.0, self.tangent[0], self.tangent[1], self.tangent[2]);
        let start = orientation * Direction::new(1.0, 0.0, 0.0, 0.0);
        let oriented_point =
            start.coords * self.distance.cos() + tangent.coords * self.distance.sin();
        // println!("START: {} | TANGENT: {} | OUT: {}", start, tangent, oriented_point);
        Point::new(
            oriented_point.w,
            oriented_point.x,
            oriented_point.y,
            oriented_point.z,
        )
    }
}

#[derive(Serialize, Deserialize)]
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
