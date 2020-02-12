use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Object<T> {
    pub geometry: T,
    pub color: [f32; 3],
    pub material: MaterialType,
}

impl<T> Object<T> {
    pub fn new(geom: T, color: [f32; 3], material: MaterialType) -> Object<T> {
        Object {
            geometry: geom,
            color: color,
            material: material,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MaterialType {
    Lambertian,
    Emissive,
}
