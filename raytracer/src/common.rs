use std::io::{Write, stderr};

use crate::materials::{MaterialType, Material, ScatterData};
use crate::random::Random;
use crate::image::Framebuffer;
use crate::camera::Camera;
use crate::maths::{Vec3, Point, NVec3, IVector};
use crate::color::{ColorU8, Color};


// ----------------- RAY ----------------------
#[derive(Copy, Clone, Debug)]
pub struct Ray {
    pub origin: Point,
    pub direction: NVec3,
}

impl Ray {
    pub fn new(origin: Point, direction: NVec3) -> Self { Self { origin, direction } }
    pub fn at(&self, t: f32) -> Point { self.origin + self.direction * t }
}



// ----------------- OTHER ----------------------
fn lerp<T>(a: T, b: T, t: f32) -> T
    where T: std::ops::Mul<f32, Output=T> + std::ops::Add<T, Output=T> {
    a*(1.0-t) + b*t
}


pub fn random_unit_sphere(random: &mut Random) -> NVec3 {
    NVec3::new(
        random.random_bilateral_f32(),
        random.random_bilateral_f32(),
        random.random_bilateral_f32(),
    )
}


// ----------------- HITTABLES ----------------------
pub struct HitRecord<'a> {
    pub position: Point,
    pub normal: NVec3,
    pub t: f32,
    pub material: &'a MaterialType,
}

trait Renderable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub center: Point,
    pub radius: f32,
    pub material: MaterialType,
}
impl Renderable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // let oc = ray.origin - self.center;
        // let a  = ray.direction.length_squared();
        // let b  = 2.0 * oc.dot(&ray.direction);
        // let c  = oc.length_squared() - self.radius.powi(2);
        // let discriminant = b.powi(2) - 4.0*a*c;

        // if discriminant < 0.0 {
        //     return None;
        // }
        //
        // let root1 = (-b - discriminant.sqrt()) / (2.0*a);
        // let root2 = (-b + discriminant.sqrt()) / (2.0*a);

        let oc = ray.origin - self.center;
        let a  = ray.direction.length_squared();
        let half_b = oc.dot(&ray.direction);
        let c = oc.length_squared() - self.radius.powi(2);
        let discriminant = half_b*half_b - a*c;

        if discriminant < 0.0 {
            return None;
        }

        let discriminant_sqrt = discriminant.sqrt();
        let root1 = (-half_b - discriminant_sqrt) / a;
        let root2 = (-half_b + discriminant_sqrt) / a;

        let t = [root1, root2]
            .iter()
            .cloned()
            .filter(|&x| t_min < x && x < t_max)
            .min_by(|a, b| a.partial_cmp(b).expect("Tried to compare a NaN"))?;

        let position = ray.at(t);
        let normal   = ((position - self.center) / self.radius).normalize();

        return Some(HitRecord{ t, position, normal, material: &self.material });
    }
}

pub struct Triangle {
    v0 : Vec3,
    v1 : Vec3,
    v2 : Vec3,
    normal   : NVec3,
    material : MaterialType,
}
pub enum Intersection {
    Intersect,
    Beside,
    Parallel,
    Behind,
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, material: MaterialType) -> Self {
        let a = v1 - v0;
        let b = v2 - v0;
        let n = a.cross(&b).normalize();
        Self {
            v0, v1, v2, normal: n, material
        }
    }
    pub fn intersect(&self, ray: &Ray,  t_min: f32, t_max: f32) -> Option<HitRecord> {
        let Triangle { v0, v1, v2, .. } = *self;

        // -- Intersection with the triangle's coplanar plane.

        // NOTE: Not normalized as the length is significant, which
        //  is why we can't use the triangles normal field.
        let a = v1 - v0;
        let b = v2 - v0;
        let n = a.cross(&b);

        fn is_zero(a: f32) -> bool { -1e-8 < a && a < 1e-8 }

        let cos_angle_and_length = n.dot(&ray.direction);
        if is_zero(cos_angle_and_length) { return None; }  // Parallel

        let d = n.dot(&v0);
        let t = (n.dot(&ray.origin) + d) / cos_angle_and_length;
        if t < t_min || t > t_max { return None; }  // TODO: Might need to check intersection in this case.

        // -- Intersection with triangle.
        let p = ray.at(t);

        // Edge 0
        let e0  = v1 - v0;
        let vp0 = p  - v0;
        let n0  = e0.cross(&vp0);
        if n.dot(&n0) < 0.0 { return None; }

        // Edge 1
        let e1  = v2 - v1;
        let vp1 = p  - v1;
        let n1 = e1.cross(&vp1);
        if n.dot(&n1) < 0.0 { return None }

        // Edge 2
        let e2  = v0 - v2;
        let vp2 = p  - v2;
        let n2 = e2.cross(&vp2);
        if n.dot(&n2) < 0.0 { return None }

        return Some(HitRecord{ position: p, normal: self.normal, t: t, material: &self.material });
    }
}

pub struct Mesh {
    triangles: Vec<Triangle>,
}
impl Mesh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        Self { triangles }
    }
}
impl Renderable for Mesh {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut hit_record : Option<HitRecord> = None;
        let mut closest_intersection = f32::INFINITY;

        for triangle in self.triangles.iter() {
            if let Some(hit) = triangle.intersect(&ray, t_min, t_max) {
                if hit.t < closest_intersection {
                    closest_intersection = hit.t;
                    hit_record = Some(
                        HitRecord {
                            position:  hit.position, normal: triangle.normal, t: hit.t, material: &triangle.material
                        }
                    );
                }
            }

            //
            // let v0 = triangle.v0;
            // let v1 = triangle.v1;
            // let v2 = triangle.v2;
            //
            // let e1 = v1 - v0;
            // let e2 = v2 - v0;
            //
            // let b = ray.origin - v0;
            // let a = Mat3::new((-ray.direction).into(), e1, e2);
            //
            // if let Some(inverse) = a.inverse() {
            //     let result = inverse.mul_vec3(&b);
            //     let t = result.x;
            //     let u = result.y;
            //     let v = result.z;
            //
            //     // CLARIFY! u and v should be able to be 0, right?
            //     if 0.0 <= u && 0.0 <= v && (u + v) <= 1.0 && 0.0 <= t && t < closest_intersection {
            //         closest_intersection = t;
            //         let position = ray.at(t);
            //         hit_record = Some(
            //             HitRecord{ position, normal: triangle.normal, t, material: &triangle.material }
            //         );
            //     }
            // }
        }

        return hit_record;
    }
}


pub struct World {
    spheres: Vec<Sphere>,
    meshes:  Vec<Mesh>,
}

impl World {
    pub fn new(spheres: Vec<Sphere>, meshes: Vec<Mesh>) -> Self {
        Self { spheres, meshes }
    }

    pub fn hit(&self, ray: &Ray) -> Option<HitRecord> {
        let mut closest = f32::INFINITY;
        let mut hit_record : Option<HitRecord> = None;

        for sphere in &self.spheres {
            let hit = sphere.hit(&ray, 0.001, closest);
            if let Some(h) = hit {
                closest = h.t;
                hit_record = Some(h);
            }
        }

        for mesh in &self.meshes {
            let hit = mesh.hit(&ray, 0.001, closest);
            if let Some(h) = hit {
                closest = h.t;
                hit_record = Some(h);
            }
        }

        hit_record
    }
}



fn ray_color(ray: &Ray, world: &World, random: &mut Random, depth: i32) -> Color {
    let mut ray = ray.clone();
    let mut final_color = Color::new(1.0, 1.0, 1.0);

    for _ in 0..depth {
        if let Some(hit) = world.hit(&ray) {
            let ScatterData { color, next_ray } = hit.material.scatter(&ray, &hit, random);
            if let Some(next_ray) = next_ray {
                final_color = final_color.mul_with_alpha(&color);
                ray = next_ray.clone();
            } else {
                return final_color.mul_with_alpha(&color);
            };
        } else {
            // Background
            let t = 0.5 * (ray.direction.normalize().y() + 1.0);
            let color = lerp(Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.5, 0.7, 1.0), t).into();
            return final_color.mul_with_alpha(&color);
        }
    }

    return Vec3::new_zero().into();
}


#[derive(Default)]
pub struct Options {
    pub samples_per_pixel: i32,
    pub max_ray_bounces:   i32,
    pub logger: Option<Box<dyn Write>>,
    pub positive_is_up:    bool,
}
impl Options {
    pub fn new(
        samples_per_pixel: i32,
        max_ray_bounces: i32,
        logger: Option<Box<dyn Write>>,
        positive_is_up: bool
    ) -> Self {
        Self {
            samples_per_pixel,
            max_ray_bounces,
            logger,
            positive_is_up,
        }
    }
    pub fn default() -> Self {
        Self {
            samples_per_pixel: 32,
            max_ray_bounces:    8,
            logger: Some(Box::new(stderr())),
            positive_is_up:  true,
        }
    }
}


pub fn ray_trace(world: &World, camera: &Camera, mut framebuffer: Framebuffer, options: &mut Options) -> Framebuffer {
    let mut random = Random::new();

    let width  = framebuffer.width;
    let height = framebuffer.height;

    // Image
    for row in 0..height {
        if let Some(logger) = &mut options.logger {
            write!(logger, "\rScanline: {:<4}", height-row-1).unwrap();
        }

        for column in 0..width {
            let mut color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..options.samples_per_pixel {
                let u = (column as f32 + random.random_f32()) / (width-1)  as f32;
                let v = (row    as f32 + random.random_f32()) / (height-1) as f32;
                let ray = camera.cast_ray(u, v);
                color = color.add_with_alpha(
                    &ray_color(&ray, world, &mut random, options.max_ray_bounces)
                );
            }

            // Gamma correction (approximate to sqrt).
            let rgba = Color::new_with_alpha(
                f32::sqrt(color.r * (1.0 / options.samples_per_pixel as f32)) * 255.999,
                f32::sqrt(color.g * (1.0 / options.samples_per_pixel as f32)) * 255.999,
                f32::sqrt(color.b * (1.0 / options.samples_per_pixel as f32)) * 255.999,
                color.a * (1.0 / options.samples_per_pixel as f32) * 255.999
            );

            framebuffer[[height - row - 1, column]] = ColorU8 {
                r: rgba.r as u8,
                g: rgba.g as u8,
                b: rgba.b as u8,
                a: rgba.a as u8
            };
        }
    }

    framebuffer
}




#[cfg(test)]
mod tests {

    #[test]
    fn bla() {

    }


}