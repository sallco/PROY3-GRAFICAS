use crate::{
    framebuffer::{Framebuffer, rgba},
    math::*,
    shaders::procedural::{Noise, mix, rgb, smooth},
};
use std::f32::consts::TAU;
pub struct Skybox {
    size: usize,
    faces: [Vec<V3>; 6],
}
fn direction(face: usize, u: f32, vv: f32) -> V3 {
    unit(match face {
        0 => v(1., vv, -u),
        1 => v(-1., vv, u),
        2 => v(u, 1., -vv),
        3 => v(u, -1., vv),
        4 => v(u, vv, 1.),
        _ => v(-u, vv, -1.),
    })
}
fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846ca68b);
    x ^ (x >> 16)
}
impl Skybox {
    pub fn new(seed: u32) -> Self {
        let size = 512;
        let noise = Noise::new(seed.wrapping_add(71));
        let mut faces = std::array::from_fn(|_| vec![V3::zero(); size * size]);
        for (face, pixels) in faces.iter_mut().enumerate() {
            for y in 0..size {
                for x in 0..size {
                    let d = direction(
                        face,
                        2. * (x as f32 + 0.5) / size as f32 - 1.,
                        1. - 2. * (y as f32 + 0.5) / size as f32,
                    );
                    let band = (-((d.y + d.x * 0.28) * 5.).powi(2)).exp();
                    let neb = noise.fbm(d * 4., 3);
                    let fine = noise.sample(d * 21.) * 0.5 + 0.5;
                    pixels[y * size + x] = rgb(0x02050c)
                        + mix(rgb(0x102336), rgb(0x342348), neb) * band * neb * 0.65
                        + rgb(0x587383) * band * fine.powi(4) * 0.12;
                }
            }
        }
        // Estrellas distribuidas sobre direcciones, proyectadas a las seis caras.
        for i in 0..3500_u32 {
            let h = hash(i ^ seed);
            let y = 2. * h as f32 / u32::MAX as f32 - 1.;
            let a = hash(h) as f32 / u32::MAX as f32 * TAU;
            let r = (1. - y * y).sqrt();
            let d = v(a.cos() * r, y, a.sin() * r);
            let (face, u, vv) = Self::coordinates(d);
            let x = ((u * 0.5 + 0.5) * size as f32) as i32;
            let y = ((0.5 - vv * 0.5) * size as f32) as i32;
            let bright = 0.15 + 0.65 * (hash(h.wrapping_add(1)) as f32 / u32::MAX as f32).powi(6);
            let color = mix(
                rgb(0x8db9ff),
                rgb(0xffdfb3),
                hash(h.wrapping_add(2)) as f32 / u32::MAX as f32,
            );
            let radius = if bright > 0.75 { 1 } else { 0 };
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let xx = x + dx;
                    let yy = y + dy;
                    if xx >= 0 && yy >= 0 && xx < size as i32 && yy < size as i32 {
                        faces[face][yy as usize * size + xx as usize] +=
                            color * bright / if dx == 0 && dy == 0 { 1. } else { 7. };
                    }
                }
            }
        }
        Self { size, faces }
    }
    fn coordinates(d: V3) -> (usize, f32, f32) {
        let a = v(d.x.abs(), d.y.abs(), d.z.abs());
        if a.x >= a.y && a.x >= a.z {
            if d.x >= 0. {
                (0, -d.z / a.x, d.y / a.x)
            } else {
                (1, d.z / a.x, d.y / a.x)
            }
        } else if a.y >= a.z {
            if d.y >= 0. {
                (2, d.x / a.y, -d.z / a.y)
            } else {
                (3, d.x / a.y, d.z / a.y)
            }
        } else if d.z >= 0. {
            (4, d.x / a.z, d.y / a.z)
        } else {
            (5, -d.x / a.z, d.y / a.z)
        }
    }
    pub fn sample(&self, d: V3) -> V3 {
        let (face, u, vv) = Self::coordinates(d);
        let x = ((u * 0.5 + 0.5) * self.size as f32 - 0.5).clamp(0., self.size as f32 - 1.);
        let y = ((0.5 - vv * 0.5) * self.size as f32 - 0.5).clamp(0., self.size as f32 - 1.);
        let ix = x as usize;
        let iy = y as usize;
        let nx = (ix + 1).min(self.size - 1);
        let ny = (iy + 1).min(self.size - 1);
        let tx = x - x.floor();
        let ty = y - y.floor();
        let p = &self.faces[face];
        (p[iy * self.size + ix] * (1. - tx) + p[iy * self.size + nx] * tx) * (1. - ty)
            + (p[ny * self.size + ix] * (1. - tx) + p[ny * self.size + nx] * tx) * ty
    }
    pub fn render(
        &self,
        fb: &mut Framebuffer,
        camera: Camera,
        nucleus: bool,
        time: f32,
        warp: Option<f32>,
        star_radius: f32,
    ) {
        let w = fb.width;
        let h = fb.height;
        let aspect = w as f32 / h as f32;
        let (r, u, f) = camera.basis();
        let tan = (camera.fov * 0.5).tan();
        let eye = camera.eye;
        let phase = warp.map_or(0., |t| (t / 3. * std::f32::consts::PI).sin().powi(4));
        let chunk = h.div_ceil(8) * w * 4;
        std::thread::scope(|s| {
            for (part, pixels) in fb.pixels.chunks_mut(chunk).enumerate() {
                s.spawn(move || {
                    let start = part * chunk / 4;
                    for (offset, pixel) in pixels.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                        let i = start + offset;
                        let x = (2. * (i % w) as f32 / w as f32 - 1.) * aspect;
                        let y = 1. - 2. * (i / w) as f32 / h as f32;
                        let ray = unit(
                            f + r * (x * tan * (1. + phase * 1.6))
                                + u * (y * tan * (1. + phase * 1.6)),
                        );
                        let mut color = if nucleus {
                            self.black_hole(eye, ray, time, camera)
                        } else {
                            let closest = eye + ray * (-eye.dot(ray)).max(0.);
                            let impact = closest.length() / star_radius;
                            let halo = if eye.dot(ray) < 0. {
                                (-impact * impact * 0.38).exp() * 0.75
                                    + (-impact * impact * 0.06).exp() * 0.045
                            } else {
                                0.
                            };
                            self.sample(ray) + rgb(0xffd79a) * halo
                        };
                        if phase > 0.001 {
                            let angle = y.atan2(x);
                            let radius = (x * x + y * y).sqrt();
                            let band = (angle * 173. + time * 0.8).sin().abs().powi(65);
                            let streak = band
                                * smooth(0.15, 1., radius)
                                * (0.45 + 0.55 * (radius * 11. - time * 24.).sin().abs());
                            color +=
                                rgb(0x75b8e8) * streak * phase * 1.7 + rgb(0x456aa4) * phase * 0.12;
                        }
                        pixel.copy_from_slice(&rgba(color));
                    }
                });
            }
        });
    }
    fn black_hole(&self, eye: V3, ray: V3, time: f32, camera: Camera) -> V3 {
        let t = -eye.dot(ray);
        if t <= 0. {
            return self.sample(ray);
        }
        let impact = eye + ray * t;
        let b = impact.length();
        let distance = eye.length();
        let inward = -unit(impact);
        let bend = if b > 0.01 {
            (5.2 / b).powi(2).min(1.8)
        } else {
            0.
        };
        let lensed = unit(ray + inward * bend);
        let mut color = self.sample(lensed);
        let hole_radius = 4.6;
        let hit = b < hole_radius;
        let sphere_t = if hit {
            t - (hole_radius * hole_radius - b * b).sqrt()
        } else {
            f32::INFINITY
        };
        if hit {
            color = rgb(0x010208);
        }
        let n = unit(v(0., 1., 0.24));
        let den = ray.dot(n);
        let td = if den.abs() > 1e-6 {
            -eye.dot(n) / den
        } else {
            -1.
        };
        if td > 0. {
            let p = eye + ray * td;
            let rr = p.length();
            if (5.1..17.).contains(&rr) && td < sphere_t {
                let angle = p.z.atan2(p.x);
                let streams = 0.5
                    + 0.5
                        * (rr * 12. + angle * 4. - time * 1.3 + 0.5 * (angle * 7. + rr * 3.).sin())
                            .sin();
                let hot = (1. - (rr - 5.) / 12.).clamp(0., 1.);
                let doppler = 0.55 + 0.45 * unit(v(-p.z, 0., p.x)).dot(-ray);
                color = mix(rgb(0xa44539), rgb(0xffedc2), hot.powf(0.45))
                    * (0.45 + streams * 0.55)
                    * (0.65 + doppler * 0.7);
            }
        }
        // Imagen secundaria artística del disco: arco alrededor de la sombra.
        let (right, up, _) = camera.basis();
        let bx = impact.dot(right);
        let by = impact.dot(up);
        let ellipse = (bx * bx + (by * 0.94).powi(2)).sqrt();
        let arc = (-((ellipse - 5.1) * 3.8).powi(2)).exp();
        let side = 0.65 + 0.35 * (bx / distance * 4.).clamp(-1., 1.);
        color += rgb(0xffc98c) * arc * side * 0.9;
        let outer = (-((b - 5.5) * 0.8).powi(2)).exp() * 0.1;
        color + rgb(0xc87c91) * outer
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cube_coordinates_round_trip() {
        for f in 0..6 {
            for (u, v) in [(-0.7, 0.6), (0.2, -0.4)] {
                let (face, x, y) = Skybox::coordinates(direction(f, u, v));
                assert_eq!(face, f);
                assert!((x - u).abs() < 1e-5 && (y - v).abs() < 1e-5);
            }
        }
    }
}
