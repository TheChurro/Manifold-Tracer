use crate::math::vectors::Vec3;

pub struct Camera {
    pub location: Vec3,
    pub extents: Vec3,
}

impl Camera {
    pub fn horizontal(&self) -> Vec3 {
        Vec3 {
            x: 2f32 * self.extents.x,
            y: 0f32,
            z: 0f32,
        }
    }

    pub fn vertical(&self) -> Vec3 {
        Vec3 {
            x: 0f32,
            y: 2f32 * self.extents.y,
            z: 0f32,
        }
    }

    pub fn out(&self) -> Vec3 {
        Vec3 {
            x: 0f32,
            y: 0f32,
            z: self.extents.z,
        }
    }

    pub fn bottom_left(&self) -> Vec3 {
        self.location - 0.5f32 * self.horizontal() - 0.5f32 * self.vertical()
    }

    /// Convert a point in camera space into world space.
    pub fn world_point(&self, u: f32, v: f32) -> Vec3 {
        // We want to start at top left corner. So the vertical needs to flip how it computes
        self.location - self.out()
            + (u - 0.5f32) * self.horizontal()
            + (0.5f32 - v) * self.vertical()
    }
}
