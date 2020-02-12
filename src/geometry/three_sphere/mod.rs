pub mod kernels;
pub mod mesh;
pub mod object;
pub mod orientation;
pub mod primitives;
pub mod representation;
pub mod scene_description;

pub const EPSILON: f32 = 0.00001;

pub use mesh::{MeshDescription, MeshInstance, Offset};
pub use object::{MaterialType, Object};
pub use orientation::Orientation;
pub use primitives::{Ball, Triangle};
pub use representation::Direction;
pub use representation::Point;
