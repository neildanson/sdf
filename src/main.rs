use minifb::{Key, Window, WindowOptions};
use rand::prelude::*;
use rayon::prelude::*;

type Vec3 = glam::Vec3A;
type FLOAT = f32;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const IMAGE_SIZE: u32 = WIDTH * HEIGHT;
const MAX_DEPTH: FLOAT = 50.0;
const INV_WIDTH: FLOAT = 1.0 / WIDTH as FLOAT;
const INV_HEIGHT: FLOAT = 1.0 / HEIGHT as FLOAT;

const MIN_DISTANCE: FLOAT = 0.001;
const VEC3_EPSILON_X: Vec3 = Vec3::new(MIN_DISTANCE, 0.0, 0.0);
const VEC3_EPSILON_Y: Vec3 = Vec3::new(0.0, MIN_DISTANCE, 0.0);
const VEC3_EPSILON_Z: Vec3 = Vec3::new(0.0, 0.0, MIN_DISTANCE);

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

fn trace_ray(ray: Ray, shapes: &Vec<&dyn Sdf>) -> Vec3 {
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
            let color_vec = (normal + Vec3::ONE) * 0.5;
            return color_vec;
        }
    }
    Vec3::ZERO
}
fn main() {
    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    let sphere = Sphere {
        center: Vec3::new(0.0, 0.0, 3.0),
        radius: 1.0,
    };

    let cube = Cube {
        center: Vec3::new(0.0, 0.0, 3.0),
        size: 0.75,
    };

    let and = And { t: cube, u: sphere };

    let shapes: Vec<&dyn Sdf> = vec![&and];
    let aspect_ratio = WIDTH as FLOAT / HEIGHT as FLOAT;    
    let mut buffer: Vec<u32> = vec![0; IMAGE_SIZE as usize];
    let mut backbuffer: Vec<Vec3> = vec![Vec3::ZERO; IMAGE_SIZE as usize];
    let d_time = std::time::Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let start = std::time::Instant::now();
        let origin = Vec3::new((d_time.elapsed().as_secs_f32() * 10.0).sin() * 2.0, 0.0, 0.0);
        (0..IMAGE_SIZE)
            .into_par_iter()
            .map(|pos| {
                let x = pos % WIDTH;
                let y = pos / WIDTH;
                let x = (x as FLOAT) * (INV_WIDTH * 2.0) - 1.0;
                let y = (y as FLOAT) * (INV_HEIGHT * 2.0) - 1.0;
                let x = x * aspect_ratio;

                let ray = Ray::new(origin, Vec3::new(x, y, 1.0).normalize());
                trace_ray(ray, &shapes)
            })
            .collect_into_vec(&mut backbuffer);

        let elapsed = start.elapsed();
        println!("Elapsed: {}ms", elapsed.as_millis());
        for (idx, i) in buffer.iter_mut().enumerate() {
            let color = backbuffer[idx];
            *i = to_color(color);
        }

        window
            .update_with_buffer(&buffer, WIDTH as usize, HEIGHT as usize)
            .unwrap();
    }
}
