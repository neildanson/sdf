use std::f32;

use image::RgbImage;
use rayon::prelude::*;

type Vec3 = glam::Vec3A;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const IMAGE_SIZE: u32 = WIDTH * HEIGHT;
const MAX_DEPTH: f32 = 50.0f32;
const INV_WIDTH: f32 = 1.0 / WIDTH as f32;
const INV_HEIGHT: f32 = 1.0 / HEIGHT as f32;

const MIN_DISTANCE : f32 = 0.001;
const VEC3_EPSILON_X : Vec3 = Vec3::new(MIN_DISTANCE, 0.0, 0.0);
const VEC3_EPSILON_Y : Vec3 = Vec3::new(0.0, MIN_DISTANCE, 0.0);
const VEC3_EPSILON_Z : Vec3 = Vec3::new(0.0, 0.0, MIN_DISTANCE);

#[derive(Debug)]
struct Ray {
    position: Vec3,
    direction: Vec3,
}

impl Ray {
    fn new(position: Vec3, direction: Vec3) -> Ray {
        Ray {
            position,
            direction,
        }
    }
}

trait Sdf: Sync + Send {
    fn distance(&self, point: Vec3) -> f32;
    fn normal(&self, point: Vec3) -> Vec3 {        
        let normal = Vec3::new(
            self.distance(point + VEC3_EPSILON_X) - self.distance(point - VEC3_EPSILON_X),
            self.distance(point + VEC3_EPSILON_Y) - self.distance(point - VEC3_EPSILON_Y),
            self.distance(point + VEC3_EPSILON_Z) - self.distance(point - VEC3_EPSILON_Z),
        );
        normal.normalize()
    }
}

struct Sphere {
    center: Vec3,
    radius: f32,
}

impl Sdf for Sphere {
    fn distance(&self, point: Vec3) -> f32 {
        (point - self.center).length() - self.radius
    }
}

struct Cube {
    center: Vec3,
    size: f32,
}

impl Sdf for Cube {
    fn distance(&self, point: Vec3) -> f32 {
        let q = (point - self.center).abs() - Vec3::splat(self.size);
        q.max(Vec3::ZERO).length() + q.max_element().min(0.0)
    }
}


fn to_color(col: Vec3) -> [u8; 3] {
    let ir = (255.99 * col.x) as u8;
    let ig = (255.99 * col.y) as u8;
    let ib = (255.99 * col.z) as u8;

    [ir, ig, ib]
}

fn trace_ray(ray: Ray, shapes: &Vec<&dyn Sdf>) -> Vec3 {
    let mut p = ray.position;
    loop {
        let mut min_distance = f32::MAX;
        for shape in shapes {
            let d = shape.distance(p);
            if d < min_distance {
                min_distance = d;
            }
        }
        if min_distance > MAX_DEPTH {
            break;
        }
        p = p + ray.direction * min_distance;
        if min_distance < MIN_DISTANCE {
            let normal = shapes[0].normal(p);
            let color_vec = (normal + Vec3::ONE) * 0.5;
            return color_vec;
        }
    }
    Vec3::ZERO
}
fn main() {
    let sphere = Sphere {
        center: Vec3::new(1.0, 0.0, 3.0),
        radius: 1.0,
    };

    let cube = Cube {
        center: Vec3::new(-1.0, 0.0, 3.0),
        size: 0.75,
    };

    let shapes: Vec<&dyn Sdf> = vec![&sphere, &cube];
    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
    let start = std::time::Instant::now();

    let result: Vec<_> = (0..IMAGE_SIZE)
        .into_par_iter()
        //.chunks(8)
        .map(|pos| {
            let x = pos % WIDTH;
            let y = pos / WIDTH;
            let x = (x as f32) * INV_WIDTH * 2.0 - 1.0;
            let y = (y as f32) * INV_HEIGHT * 2.0 - 1.0;
            let x = x * aspect_ratio;
            let ray = Ray::new(Vec3::ZERO, Vec3::new(x, y, 1.0).normalize());
            trace_ray(ray, &shapes)
        })
        .collect();

    let elapsed = start.elapsed();

    let mut img = RgbImage::new(WIDTH, HEIGHT);
    for (i, pixel) in result.iter().enumerate() {
        let x = i as u32 % WIDTH;
        let y = i as u32 / WIDTH;
        let pixel = to_color(*pixel);
        img.put_pixel(x, y, image::Rgb(pixel));
    }
    println!("Elapsed: {:?}", elapsed);
    img.save("output.png").unwrap();
}
