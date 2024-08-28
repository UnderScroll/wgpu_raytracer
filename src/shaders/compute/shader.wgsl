//Structs
struct Args {
    width: u32,
    height: u32,
    samples: u32,
};

struct Material {
    mat_type: u32,
    color: vec3<f32>,
    ior: f32
}

struct Viewport {
    origin: vec3<f32>,
    args: Args,
    size: f32,
    u: vec3<f32>,
    v: vec3<f32>,
    delta_u: vec3<f32>,
    delta_v: vec3<f32>,
    pixel_origin: vec3<f32>,
}

struct Camera {
    position: vec3<f32>,
    up: vec3<f32>,
    forward: vec3<f32>,
    right: vec3<f32>,
    focal_length: f32
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct Sphere {
    position: vec3<f32>,
    radius: f32,
    material: Material,
}

struct RaycastHit {
    has_hit: bool,
    distance: f32,
    point: vec3<f32>,
    normal: vec3<f32>,
    material: Material,
}

//Bindings
@group(0) @binding(0) 
var<uniform> args: Args; 
@group(0) @binding(1) 
var output_texture: texture_storage_2d<rgba8unorm, write>;

//Utils
//https://gist.github.com/munrocket/236ed5ba7e409b8bdf1ff6eca5dcdc39
//Based on : http://www.jcgt.org/published/0009/03/02/
fn pcg3d(p: vec3<u32>) -> vec3<u32> {
    var v = p * 1664525u + 1013904223u;
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    v ^= v >> vec3<u32>(16u);
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    return v;
}

//State
const CAMERA_POSITION = vec3<f32>(0.0, 0.0, 1.0);
const CAMERA_LOOK_AT = vec3<f32>(0.0, 0.0, 0.0);
const CAMERA_FOCAL_LENGTH = 1.0;

const GLOBAL_UP = vec3<f32>(0.0, 1.0, 0.0);

const MAX_BOUNCE: u32 = 4u;

const MAT_TYPE_DIFFUSE: u32 = 1u << 0u;
const MAT_TYPE_METALIC: u32 = 1u << 1u;
const MAT_TYPE_TRANSPARENT: u32 = 1u << 2u;

var<private> world: array<Sphere, 4> = array<Sphere, 4>(
    Sphere(
        vec3<f32>(0, 0, -1.4), 
        0.5, 
        Material(MAT_TYPE_DIFFUSE, vec3<f32>(0.1, 0.1, 0.5), 1.0)),
    Sphere(
        vec3<f32>(-1, 0, -1), 
        0.5, 
        Material(MAT_TYPE_TRANSPARENT, vec3<f32>(0.8, 0.8, 0.8), 1.5)),
    Sphere(
        vec3<f32>(1, 0, -1), 
        0.5, 
        Material(MAT_TYPE_METALIC, vec3<f32>(0.8, 0.6, 0.2), 1.0)),
    Sphere(
        vec3<f32>(0, -20000.5, -1), 
        20000.0, 
        Material(MAT_TYPE_DIFFUSE, vec3<f32>(0.8, 0.8, 0.0), 1.0))
);

//Entry point
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_ix: vec3<u32>) {
    let uv = vec2<f32>(vec2<i32>(global_ix.xy)) / vec2<f32>(f32(args.width), f32(args.height));

    let camera = new_camera(CAMERA_POSITION, CAMERA_LOOK_AT, CAMERA_FOCAL_LENGTH);
    let viewport = new_viewport(2.0, args, camera);

    let frag_color = get_color(camera.position, vec2<f32>(f32(global_ix.x), f32(global_ix.y)), viewport);
    textureStore(output_texture, vec2<i32>(global_ix.xy), frag_color);
}

fn new_camera(position: vec3<f32>, look_at: vec3<f32>, focal_length: f32) -> Camera {
    let forward = normalize(look_at - position);
    let right = cross(forward, GLOBAL_UP);
    let up = cross(right, forward);

    return Camera(position, up, forward, right, focal_length);
}

fn new_viewport(size: f32, args: Args, camera: Camera) -> Viewport {
    let u: vec3<f32> = camera.right * size;
    let v: vec3<f32> = camera.up * -(size * (f32(args.height) / f32(args.width)));

    let origin: vec3<f32> = camera.position + camera.forward * camera.focal_length - u / 2.0 - v / 2.0;

    let delta_u: vec3<f32> = u / f32(args.width);
    let delta_v: vec3<f32> = v / f32(args.height);

    let pixel_origin: vec3<f32> = origin + 0.5 * (delta_u * delta_v);

    return Viewport(origin, args, size, u, v, delta_u, delta_v, pixel_origin);
}

fn get_color(ray_origin: vec3<f32>, pixel_position: vec2<f32>, viewport: Viewport) -> vec4<f32> {
    var color = vec3<f32>(0.0, 0.0, 0.0);
    
    for (var i: u32 = 0; i < args.samples; i++) 
    {
        color += render_pixel_sample(ray_origin, pixel_position, viewport, i) / f32(args.samples);
    }

    return vec4<f32>(color, 1.0);
}

fn render_pixel_sample(ray_origin: vec3<f32>, pixel_position: vec2<f32>, viewport: Viewport, sample: u32) -> vec3<f32> {
    let random_vec = pcg3d(vec3<u32>(u32(pixel_position.x), u32(pixel_position.y), sample));
    var random_vec_n = normalize(vec3<f32>(f32(random_vec.x), f32(random_vec.y), f32(random_vec.z)));
    random_vec_n *= 2.0;
    random_vec_n -= 1.0;
    
    let viewport_point = viewport.origin + (pixel_position.x + random_vec_n.x) * viewport.delta_u + (pixel_position.y + random_vec_n.y) * viewport.delta_v;
    var ray = Ray(ray_origin, viewport_point - ray_origin);

    return get_ray_color(ray, sample);
}

fn get_ray_color(ray: Ray, sample: u32) -> vec3<f32> {
    var ray_hits = array<RaycastHit, MAX_BOUNCE>();

    var n_ray = Ray(ray.origin, ray.direction);

    //Casting ray
    var i: u32 = 0;
    for (;i < MAX_BOUNCE; i++)
    {
        //Get closest hit
        var hit: RaycastHit = intersect_sphere(world[0], n_ray);
        for (var i: u32 = 1; i < 4; i++) 
        {
            let o_hit = intersect_sphere(world[i], n_ray);
            if !hit.has_hit || o_hit.has_hit && o_hit.distance <= hit.distance { hit = o_hit; }
        }

        ray_hits[i] = hit;

        //Handle hit
        if hit.has_hit 
        {
            //Handle bounce
            switch hit.material.mat_type
            {
                case MAT_TYPE_DIFFUSE: 
                {
                    let random_u32vec = pcg3d(vec3<u32>(i, i * sample, sample));
                    var random_n_f32vec = normalize(vec3<f32>(f32(random_u32vec.x), f32(random_u32vec.y), f32(random_u32vec.z)));
                    random_n_f32vec *= 2.0;
                    random_n_f32vec -= 1.0;
                    let new_dir = hit.normal + random_n_f32vec;
                    n_ray = Ray(hit.point, hit.normal + new_dir);
                }
                case MAT_TYPE_METALIC: 
                {
                    n_ray = Ray(hit.point, reflect(n_ray.direction, hit.normal));
                }
                case MAT_TYPE_TRANSPARENT:
                {
                    if dot(hit.normal, n_ray.direction) < 0
                    {
                        n_ray = Ray(hit.point, refract(n_ray.direction, hit.normal, 1.0 / hit.material.ior));
                    }
                    else
                    {
                        n_ray = Ray(hit.point, refract(n_ray.direction, hit.normal, hit.material.ior));
                    }
                } 
                default: { break; }
            }
            continue;
        }

        break;
    }

    //Mixing color of ray bounce
    var color = vec3<f32>(1.0, 1.0, 1.0);
    for (var j: u32 = 0; j <= i; j++) 
    {
        color *= ray_hits[j].material.color;
    }

    return color;
}

fn background_color(ray_direction: vec3<f32>) -> vec3<f32> {
    let blend = 0.5 * (normalize(ray_direction).y + 1.0);

    let SKY_COLOR: vec3<f32> = vec3<f32>(125.0 / 255.0, 178.0 / 247.0, 1.0);
    let GROUND_COLOR: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
    return mix(GROUND_COLOR, SKY_COLOR, blend);
}

fn intersect_sphere(sphere: Sphere, ray: Ray) -> RaycastHit {
    let ray_sphere = sphere.position - ray.origin;

    let a = dot(ray.direction, ray.direction);
    let h = dot(ray.direction, ray_sphere);
    let c = dot(ray_sphere, ray_sphere) - sphere.radius * sphere.radius;

    let discriminant = h * h - a * c;
    let t = (h - sqrt(discriminant)) / a;
    if discriminant < 0.0 || t < 0.001 {
        let skyMat = Material(MAT_TYPE_DIFFUSE, background_color(ray.direction), 1.0);
        return RaycastHit(false, bitcast<f32>(0x7F800000), ray.origin, ray.direction, skyMat);
    }

    let point = ray.origin + ray.direction * t;
    let normal = normalize(point - sphere.position);

    return RaycastHit(true, t, point, normal, sphere.material);
}