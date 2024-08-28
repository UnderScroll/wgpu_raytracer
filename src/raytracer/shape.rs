use glam::Vec3;

use super::{
    material::{Material, RaytraceMaterial},
    Ray, RayBounce, RayCast, RaycastHit, Raytrace, Rgb,
};

pub struct Sphere {
    pub position: Vec3,
    pub radius: f32,
    pub material: Box<dyn RaytraceMaterial + Sync>,
}

impl Raytrace for Sphere {}

impl RayCast for Sphere {
    fn ray_cast(&self, ray: &Ray) -> Option<RaycastHit> {
        let ray_sphere = self.position - ray.origin;

        let a = ray.direction.dot(ray.direction);
        let h = ray.direction.dot(ray_sphere);
        let c = ray_sphere.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        let t = (h - discriminant.sqrt()) / a;
        if discriminant < 0.0 || t < 0.001 {
            return None;
        }

        let point = ray.point_at(t);
        let normal = (point - self.position).normalize_or_zero();

        Some(RaycastHit {
            distance: t,
            point,
            normal,
            object: self,
        })
    }
}

impl RayBounce for Sphere {
    fn ray_bounce(&self, incident: &Vec3, normal: &Vec3, position: &Vec3) -> Option<Ray> {
        self.material.ray_bounce(incident, normal, position)
    }
}

impl Material for Sphere {
    fn get_color(&self) -> Rgb<u8> {
        self.material.get_color()
    }

    fn get_ior(&self) -> Option<f32> {
        self.material.get_ior()
    }
}
