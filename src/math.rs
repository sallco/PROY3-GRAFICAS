use raylib::prelude::*;
pub type V3 = Vector3;
pub fn v(x: f32, y: f32, z: f32) -> V3 {
    V3::new(x, y, z)
}
pub fn unit(a: V3) -> V3 {
    if a.dot(a) < 1e-12 {
        v(0., 1., 0.)
    } else {
        a.normalized()
    }
}
pub fn rotate_y(p: V3, angle: f32) -> V3 {
    let (s, c) = angle.sin_cos();
    v(c * p.x + s * p.z, p.y, -s * p.x + c * p.z)
}
#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub position: V3,
    pub x: V3,
    pub y: V3,
    pub z: V3,
    pub scale: f32,
}
impl Transform {
    pub fn new(position: V3, scale: f32, angle: f32) -> Self {
        Self {
            position,
            scale,
            x: rotate_y(v(1., 0., 0.), angle),
            y: v(0., 1., 0.),
            z: rotate_y(v(0., 0., 1.), angle),
        }
    }
    pub fn vector(self, p: V3) -> V3 {
        self.x * p.x + self.y * p.y + self.z * p.z
    }
    pub fn point(self, p: V3) -> V3 {
        self.position + self.vector(p) * self.scale
    }
    pub fn inverse(self, p: V3) -> V3 {
        let q = (p - self.position) / self.scale;
        v(q.dot(self.x), q.dot(self.y), q.dot(self.z))
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub eye: V3,
    pub target: V3,
    pub up: V3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}
impl Camera {
    pub fn new(eye: V3, target: V3) -> Self {
        Self {
            eye,
            target,
            up: v(0., 1., 0.),
            fov: 55_f32.to_radians(),
            near: 0.08,
            far: 650.,
        }
    }
    pub fn basis(self) -> (V3, V3, V3) {
        let f = unit(self.target - self.eye);
        let axis = if f.dot(unit(self.up)).abs() > 0.999 {
            v(0., 0., 1.)
        } else {
            self.up
        };
        let r = unit(f.cross(axis));
        (r, unit(r.cross(f)), f)
    }
    pub fn clip(self, p: V3, aspect: f32) -> [f32; 4] {
        let (r, u, f) = self.basis();
        let q = p - self.eye;
        let z = q.dot(f);
        let k = 1. / (self.fov * 0.5).tan();
        [
            q.dot(r) * k / aspect,
            q.dot(u) * k,
            (self.far + self.near) / (self.far - self.near) * z
                - 2. * self.far * self.near / (self.far - self.near),
            z,
        ]
    }
    pub fn ray(self, x: f32, y: f32, aspect: f32) -> V3 {
        let (r, u, f) = self.basis();
        let t = (self.fov * 0.5).tan();
        unit(f + r * (x * t * aspect) + u * (y * t))
    }
}
