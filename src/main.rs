
type Vec3 = glam::Vec3A;

#[derive(Debug)]
struct Ray {
    position : Vec3,
    direction : Vec3,
}

impl Ray {
    fn new(position: Vec3, direction : Vec3) -> Ray {
        Ray { position, direction }
    }
}


fn main() {
    let ray = Ray::new(Vec3::ZERO, Vec3::Y);
    println!("Ray position: {:?}", ray);
}
