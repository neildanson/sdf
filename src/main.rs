use std::cell::RefCell;

use minifb::{Key, Scale, Window, WindowOptions};
use rand::prelude::*;
use rayon::prelude::*;

type Vec3 = glam::Vec3A;
type FLOAT = f32;

const WIDTH: usize = 320;
const HEIGHT: usize = 160;
const IMAGE_SIZE: usize = WIDTH * HEIGHT;
const MAX_DEPTH: FLOAT = 50.0;
const INV_WIDTH: FLOAT = 1.0 / WIDTH as FLOAT;
const INV_HEIGHT: FLOAT = 1.0 / HEIGHT as FLOAT;

const MIN_DISTANCE: FLOAT = 0.001;
const VEC3_EPSILON_X: Vec3 = Vec3::new(MIN_DISTANCE, 0.0, 0.0);
const VEC3_EPSILON_Y: Vec3 = Vec3::new(0.0, MIN_DISTANCE, 0.0);
const VEC3_EPSILON_Z: Vec3 = Vec3::new(0.0, 0.0, MIN_DISTANCE);
const SAMPLES: usize = 10;

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(rand::rng());
}

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

struct HitRecord {
    t: f32,
    p: Vec3,
    normal: Vec3,
}

impl HitRecord {
    fn new(t: f32, p: Vec3, normal: Vec3) -> HitRecord {
        HitRecord { t, p, normal }
    }
}

trait Sdf: Sync + Send {
    fn distance(&self, point: Vec3) -> FLOAT;
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
    radius: FLOAT,
}

impl Sdf for Sphere {
    fn distance(&self, point: Vec3) -> FLOAT {
        (point - self.center).length() - self.radius
    }
}

struct Cube {
    center: Vec3,
    size: FLOAT,
}

impl Sdf for Cube {
    fn distance(&self, point: Vec3) -> FLOAT {
        let q = (point - self.center).abs() - Vec3::splat(self.size);
        q.max(Vec3::ZERO).length() + q.max_element().min(0.0)
    }
}

struct And<T: Sdf, U: Sdf> {
    t: T,
    u: U,
}

impl<T: Sdf, U: Sdf> Sdf for And<T, U> {
    fn distance(&self, point: Vec3) -> FLOAT {
        self.t.distance(point).max(self.u.distance(point))
    }
}

struct Not<T: Sdf, U: Sdf> {
    t: T,
    u: U,
}

impl<T: Sdf, U: Sdf> Sdf for Not<T, U> {
    fn distance(&self, point: Vec3) -> FLOAT {
        self.t.distance(point).max(-self.u.distance(point))
    }
}

fn to_color(col: Vec3) -> u32 {
    let ir = (255.99 * col.x) as u32;
    let ig = (255.99 * col.y) as u32;
    let ib = (255.99 * col.z) as u32;

    let color = (ir << 16) | (ig << 8) | ib;
    color
}

fn random_in_unit_sphere() -> Vec3 {
    RNG.with_borrow_mut(|rng| loop {
        let p = Vec3::new(rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0));
        if p.length_squared() < 1.0 {
            return p;
        }
    })
}


fn trace_ray(ray: Ray, shapes: &[Box<dyn Sdf>], depth : usize) -> Vec3 {
    if depth > 5 {
        return Vec3::ZERO;
    }
    let mut p = ray.position;
    loop {
        let mut min_distance = FLOAT::MAX;
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
            let hit = HitRecord::new(min_distance, p, normal);
            let target = hit.p + hit.normal + random_in_unit_sphere();
            return 0.5 * trace_ray(Ray::new(hit.p, target - hit.p), shapes, depth + 1);
        }

    }
    let unit_direction = ray.direction.normalize();
    let t = 0.5 * (unit_direction.y + 1.0);
    (1.0 - t) * Vec3::ONE + t * Vec3::new(0.5, 0.7, 1.0)
    //Vec3::ZERO
}
fn main() {
    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions { 
            scale: Scale::X4,
            .. WindowOptions::default() 
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut shapes :Vec<Box<dyn Sdf> >= Vec::new();


    for z in 3 .. 6   {
        let sphere = Sphere {
            center: Vec3::new(0.0, 0.0, z as FLOAT),
            radius: 1.0,
        };
    
        let cube = Cube {
            center: Vec3::new(0.0, 0.0, z as FLOAT),
            size: 0.75,
        };
    
        let and = And { t: cube, u: sphere };
        shapes.push(Box::new(and));
    }

    let aspect_ratio = WIDTH as FLOAT / HEIGHT as FLOAT;    
    let mut buffer: Vec<u32> = vec![0; IMAGE_SIZE];
    let mut backbuffer: Vec<Vec3> = vec![Vec3::ZERO; IMAGE_SIZE];
    let d_time = std::time::Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let start = std::time::Instant::now();
        let origin = Vec3::new((d_time.elapsed().as_secs_f32() * 10.0).sin() * 2.0, (d_time.elapsed().as_secs_f32() * 10.0).cos(), 0.0);
        (0..IMAGE_SIZE)
            .into_par_iter()
            .map(|pos| {
                let x = pos % WIDTH;
                let y = pos / WIDTH;
                let x = (x as FLOAT) * (INV_WIDTH * 2.0) - 1.0;
                let y = (y as FLOAT) * (INV_HEIGHT * 2.0) - 1.0;
                let x = x * aspect_ratio;
                let color = (0 .. SAMPLES).into_iter().fold(Vec3::ZERO, |c, _| {
                    let (x, y) = RNG.with_borrow_mut(|rng| {
                        let u = (x as f32 + rng.random::<f32>());
                        let v = (y as f32 + rng.random::<f32>());
                        (u, v)});
                    let ray = Ray::new(origin, Vec3::new(x, y, 1.0).normalize());
                    trace_ray(ray, &shapes, 0) + c
                });
    
                color / SAMPLES as f32
                //let ray = Ray::new(origin, Vec3::new(x, y, 1.0).normalize());
                //trace_ray(ray, &shapes, 0)
            })
            .collect_into_vec(&mut backbuffer);

        let elapsed = start.elapsed();
        println!("Elapsed: {}ms", elapsed.as_millis());
        for (idx, i) in buffer.iter_mut().enumerate() {
            let color = backbuffer[idx];
            *i = to_color(color);
        }

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
