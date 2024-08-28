use super::{ray::Ray, RayBounce};
use crate::raytracer::Rgb;

use glam::{vec3, Vec3};

//Matrial
pub trait Material {
    fn get_color(&self) -> Rgb<u8>;
    fn get_ior(&self) -> Option<f32>;
}

pub trait RaytraceMaterial: RayBounce + Material {}

//Utility
trait Reflect {
    fn reflect(self, rhs: &Self) -> Self;
}

impl Reflect for Vec3 {
    fn reflect(self, rhs: &Self) -> Self {
        self - 2.0 * self.dot(*rhs) * (*rhs)
    }
}

trait Refract {
    fn refract(self, normal: &Vec3, r: f32) -> Self;
}

// Formula from [https://en.wikipedia.org/wiki/Snell%27s_law]
impl Refract for Vec3 {
    fn refract(self, normal: &Vec3, r: f32) -> Self {
        let c = -normal.dot(self);
        let c_sq = c * c;
        let r_sq = r * r;

        let p_1 = r * self;
        let p_2 = r * c;
        let p_3 = (1.0 - r_sq * (1.0 - c_sq)).sqrt();

        let sin_a = (1.0 - c * c).sqrt();

        if r * sin_a > 1.0 {
            self.reflect(normal)
        } else {
            p_1 + (p_2 - p_3) * (*normal)
        }
    }
}

//Diffuse
pub struct DiffuseMaterial {
    pub color: Rgb<u8>,
}

impl RaytraceMaterial for DiffuseMaterial {}

impl Material for DiffuseMaterial {
    fn get_color(&self) -> Rgb<u8> {
        self.color
    }

    fn get_ior(&self) -> Option<f32> {
        None
    }
}

impl RayBounce for DiffuseMaterial {
    fn ray_bounce(&self, _incident: &Vec3, normal: &Vec3, position: &Vec3) -> Option<Ray> {
        let out = vec3(
            fastrand::f32() * 2.0 - 1.0,
            fastrand::f32() * 2.0 - 1.0,
            fastrand::f32() * 2.0 - 1.0,
        )
        .normalize_or_zero();

        Some(Ray {
            origin: *position,
            direction: *normal + out,
            ior: 1.0,
        })
    }
}

//Metal
pub struct MetalMaterial {
    pub color: Rgb<u8>,
}

impl RaytraceMaterial for MetalMaterial {}

impl Material for MetalMaterial {
    fn get_color(&self) -> Rgb<u8> {
        self.color
    }

    fn get_ior(&self) -> Option<f32> {
        None
    }
}

impl RayBounce for MetalMaterial {
    fn ray_bounce(&self, incident: &Vec3, normal: &Vec3, position: &Vec3) -> Option<Ray> {
        let out = incident.reflect(normal);

        Some(Ray {
            origin: *position,
            direction: out,
            ior: 1.0,
        })
    }
}

//Metal
pub struct TransparentMaterial {
    pub color: Rgb<u8>,
    pub ior: f32,
}

impl RaytraceMaterial for TransparentMaterial {}

impl Material for TransparentMaterial {
    fn get_color(&self) -> Rgb<u8> {
        self.color
    }

    fn get_ior(&self) -> Option<f32> {
        Some(self.ior)
    }
}

impl RayBounce for TransparentMaterial {
    fn ray_bounce(&self, incident: &Vec3, normal: &Vec3, position: &Vec3) -> Option<Ray> {
        let position = *position;

        let is_inside = incident.dot(*normal) > 0.0;
        let ior_ratio = if is_inside { self.ior } else { 1.0 / self.ior };

        let refract_direction = incident.refract(normal, ior_ratio);

        Some(Ray {
            origin: position,
            direction: refract_direction,
            ior: self.ior,
        })
    }
}
