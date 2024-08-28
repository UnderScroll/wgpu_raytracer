#![allow(dead_code)]

pub mod camera;
pub mod material;
pub mod ray;
pub mod shape;

pub use crate::{
    colors::{Rgb, Rgba},
    texture::Texture,
};

use clap::ValueEnum;
#[allow(unused_imports)]
use log::{debug, info, log, trace, warn};

use camera::{Camera, Resolution, Viewport};
use material::{DiffuseMaterial, Material, MetalMaterial, TransparentMaterial};
use ray::Ray;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use shape::Sphere;

use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

use anyhow::Result;
use glam::{vec3, Vec3};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressFinish, ProgressStyle};

pub struct RaycastHit<'a> {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub object: &'a dyn Raytrace,
}

pub trait RayBounce {
    fn ray_bounce(&self, incident: &Vec3, normal: &Vec3, position: &Vec3) -> Option<Ray>;
}

pub trait RayCast {
    fn ray_cast(&self, ray: &Ray) -> Option<RaycastHit>;
}

pub trait Raytrace: RayBounce + RayCast + Material {}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum RenderMode {
    SingleThread,
    MultiThread,
    Gpu,
}

pub fn render(texture: &mut Texture, sample_count: u32, mode: RenderMode) -> Result<()> {
    let camera = Camera::new(vec3(0.0, 0.0, 1.0), 2.0, vec3(0.0, 0.0, 0.0), 1.0);

    let resolution = Resolution {
        width: texture.width as u32,
        height: texture.height as u32,
    };

    let world: Vec<Box<dyn Raytrace + Sync>> = vec![
        //Diffuse
        Box::new(Sphere {
            position: vec3(0.0, 0.0, -1.4),
            radius: 0.5,
            material: Box::new(DiffuseMaterial {
                color: Rgb([25, 52, 125]),
            }),
        }),
        //Transparent
        Box::new(Sphere {
            position: vec3(-1.0, 0.0, -1.0),
            radius: 0.5,
            material: Box::new(TransparentMaterial {
                color: Rgb([200, 200, 200]),
                ior: 1.5,
            }),
        }),
        //Metal
        Box::new(Sphere {
            position: vec3(1.0, 0.0, -1.0),
            radius: 0.5,
            material: Box::new(MetalMaterial {
                color: Rgb([200, 150, 50]),
            }),
        }),
        //Ground
        Box::new(Sphere {
            position: vec3(0.0, -20000.5, -1.0),
            radius: 20000.0,
            material: Box::new(DiffuseMaterial {
                color: Rgb([205, 205, 0]),
            }),
        }),
    ];

    match mode {
        RenderMode::SingleThread => {
            render_single_thread(texture, &camera, resolution, &world, sample_count)
        }
        RenderMode::MultiThread => render_multi_thread(
            Arc::new(Mutex::new(texture)),
            &camera,
            resolution,
            &world,
            sample_count,
        ),
        RenderMode::Gpu => todo!(),
    }
}

fn render_single_thread(
    texture: &mut Texture,
    camera: &Camera,
    resolution: Resolution,
    world: &Vec<Box<dyn Raytrace + Sync>>,
    sample_count: u32,
) -> Result<()> {
    let progress_bar = ProgressBar::new(texture.width as u64).with_style(
        ProgressStyle::with_template(
            "RENDERING : {bar:100.green/black} [elapsed : {elapsed_precise}, eta: {eta_precise}] {msg}",
        )
        .expect("Setting template"),
    );

    let viewport = Viewport::new(camera.size, resolution, camera);

    info!("Starting Single-thread CPU Rendering...");
    let start_time = SystemTime::now();

    for i in progress_bar.wrap_iter(0..texture.width) {
        for j in 0..texture.height {
            let (x, y) = (i as f32, j as f32);

            let mut sum_color = Rgb::<f32>::default();
            for _ in 0..sample_count {
                let (random_x_offset, random_y_offset) =
                    ((fastrand::f32() - 0.5) * 2.0, (fastrand::f32() - 0.5) * 2.0);

                let viewport_pixel_position = viewport.origin
                    + (x + random_x_offset) * viewport.delta_u
                    + (y + random_y_offset) * viewport.delta_v;

                let ray = Ray::new(
                    camera.position,
                    viewport_pixel_position - camera.position,
                    1.0,
                );
                let pixel_color = render_pixel_sample(&ray, world);

                sum_color = Rgb([
                    sum_color[0] + pixel_color[0],
                    sum_color[1] + pixel_color[1],
                    sum_color[2] + pixel_color[2],
                ]);
            }

            let avg_color = (Rgb([
                sum_color[0] / sample_count as f32,
                sum_color[1] / sample_count as f32,
                sum_color[2] / sample_count as f32,
            ]))
            .into();
            texture.set_pixel(i, j, Rgba::<u8>::from_rgb(&avg_color, 255))?;
        }
    }
    progress_bar.finish();

    match start_time.elapsed() {
        Ok(elapsed) => info!("Finished rendering in {}ms", elapsed.as_millis()),
        Err(e) => {
            info!("Finished rendering");
            warn!("Failed to get rendertime {e}");
        }
    }

    Ok(())
}

fn render_multi_thread(
    texture: Arc<Mutex<&mut Texture>>,
    camera: &Camera,
    resolution: Resolution,
    world: &Vec<Box<dyn Raytrace + Sync>>,
    sample_count: u32,
) -> Result<()> {
    let texture_width = texture.lock().unwrap().width;
    let texture_height = texture.lock().unwrap().height;

    let progress_bar = ProgressBar::new(texture_width as u64).with_style(
        ProgressStyle::with_template(
            "RENDERING : {bar:100.green/black} [elapsed : {elapsed_precise}, eta: {eta_precise}] {msg}",
        )
        .expect("Setting template"),
    ).with_finish(ProgressFinish::AndLeave);

    let viewport = Viewport::new(camera.size, resolution, camera);

    info!("Starting Multi-Thread CPU Rendering...");
    let start_time = SystemTime::now();

    (0..texture_width)
        .into_par_iter()
        .progress_with(progress_bar)
        .for_each(|i| {
            for j in 0..texture_height {
                let (x, y) = (i as f32, j as f32);

                let mut sum_color = Rgb::<f32>::default();
                for _ in 0..sample_count {
                    let (random_x_offset, random_y_offset) =
                        ((fastrand::f32() - 0.5) * 2.0, (fastrand::f32() - 0.5) * 2.0);

                    let viewport_pixel_position = viewport.origin
                        + (x + random_x_offset) * viewport.delta_u
                        + (y + random_y_offset) * viewport.delta_v;

                    let ray = Ray::new(
                        camera.position,
                        viewport_pixel_position - camera.position,
                        1.0,
                    );
                    let pixel_color = render_pixel_sample(&ray, world);

                    sum_color = Rgb([
                        sum_color[0] + pixel_color[0],
                        sum_color[1] + pixel_color[1],
                        sum_color[2] + pixel_color[2],
                    ]);
                }

                let avg_color = (Rgb([
                    sum_color[0] / sample_count as f32,
                    sum_color[1] / sample_count as f32,
                    sum_color[2] / sample_count as f32,
                ]))
                .into();

                texture
                    .lock()
                    .unwrap()
                    .set_pixel(i, j, Rgba::<u8>::from_rgb(&avg_color, 255))
                    .unwrap();
            }
        });

    match start_time.elapsed() {
        Ok(elapsed) => info!("Finished rendering in {}ms", elapsed.as_millis()),
        Err(e) => {
            info!("Finished rendering");
            warn!("Failed to get rendertime {e}");
        }
    }

    Ok(())
}

fn render_pixel_sample(ray: &Ray, objects: &Vec<Box<dyn Raytrace + Sync>>) -> Rgb<f32> {
    let max_ray_bounce = 1024;

    get_ray_color(ray, objects, 0, max_ray_bounce)
}

fn get_ray_color(
    ray: &Ray,
    objects: &Vec<Box<dyn Raytrace + Sync>>,
    iteration_count: u32,
    max_iteration: u32,
) -> Rgb<f32> {
    if iteration_count > max_iteration {
        return Rgb::<f32>::BLACK;
    };

    let mut closest_hit: Option<RaycastHit> = Option::None;
    let mut min_distance = f32::INFINITY;
    for object in objects {
        if let Some(hit) = object.ray_cast(ray) {
            if min_distance > hit.distance {
                min_distance = hit.distance;
                closest_hit = Some(hit);
            }
        }
    }

    if let Some(closest_hit) = closest_hit {
        let object_color = Rgb::<f32>::from(closest_hit.object.get_color());
        let bounce_ray =
            closest_hit
                .object
                .ray_bounce(&ray.direction, &closest_hit.normal, &closest_hit.point);

        return if let Some(bounce_ray) = bounce_ray {
            let out_ray_color =
                get_ray_color(&bounce_ray, objects, iteration_count + 1, max_iteration);

            object_color * out_ray_color
        } else {
            background_color(ray, 1.0)
        };
    }

    background_color(ray, 1.0)
}

fn background_color(ray: &Ray, blend_factor: f32) -> Rgb<f32> {
    let blend = 0.5 * (ray.direction.normalize().y + 1.0) * blend_factor;

    const SKY_COLOR: Rgb<f32> = Rgb([125.0 / 255.0, 178.0 / 247.0, 1.0]);
    const GROUND_COLOR: Rgb<f32> = Rgb([1.0, 1.0, 1.0]);

    GROUND_COLOR.blend(&SKY_COLOR, blend)
}
