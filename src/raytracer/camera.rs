use glam::{vec3, Vec3};

pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

pub struct Viewport {
    pub origin: Vec3,
    pub resolution: Resolution,
    pub size: f32,
    pub u: Vec3,
    pub v: Vec3,
    pub delta_u: Vec3,
    pub delta_v: Vec3,
    pub pixel_origin: Vec3,
}

impl Viewport {
    pub fn new(size: f32, resolution: Resolution, camera: &Camera) -> Self {
        let u = camera.right * size;
        let v = camera.up * -(size / resolution.aspect_ratio());

        let origin = camera.position + camera.forward * camera.focal_length - u / 2.0 - v / 2.0;

        let delta_u = u / resolution.width as f32;
        let delta_v = v / resolution.height as f32;

        let pixel_origin = origin + 0.5 * (delta_u + delta_v);

        Viewport {
            origin,
            resolution,
            size,
            u,
            v,
            delta_u,
            delta_v,
            pixel_origin,
        }
    }
}

pub struct Camera {
    pub position: Vec3,
    pub size: f32,
    pub focal_length: f32,
    pub up: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
}

impl Camera {
    pub fn new(position: Vec3, size: f32, look_at: Vec3, focal_length: f32) -> Self {
        const GLOBAL_UP: Vec3 = vec3(0.0, 1.0, 0.0);

        let forward = (look_at - position).normalize();
        let right = forward.cross(GLOBAL_UP);
        let up = right.cross(forward);

        Self {
            position,
            size,
            focal_length,
            up,
            forward,
            right,
        }
    }
}
