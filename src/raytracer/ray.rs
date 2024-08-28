use glam::Vec3;

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub ior: f32,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3, ior: f32) -> Ray {
        Ray {
            origin,
            direction,
            ior,
        }
    }

    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}
