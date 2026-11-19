pub mod procedural;
mod worlds;
use crate::terrain::land;
use crate::{math::*, renderer::Fragment, scene::Scene};
use procedural::{Noise, mix, rgb, smooth};
use std::f32::consts::{PI, TAU};
pub struct NormalMap {
    pub width: usize,
    pub height: usize,
    pub values: Vec<V3>,
}
impl NormalMap {
    pub fn generate(noise: &Noise) -> Self {
        let width = 256;
        let height = 128;
        let height_at = |u: f32, v: f32| {
            let a = u * TAU;
            let lat = (0.5 - v) * PI;
            let p = crate::math::v(lat.cos() * a.cos(), lat.sin(), lat.cos() * a.sin());
            noise.sample(p * 24.) * 0.015 + noise.sample(p * 57.) * 0.004
        };
        let mut values = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let u = x as f32 / width as f32;
                let v = y as f32 / (height - 1) as f32;
                let e = 0.002;
                let dx = (height_at(u + e, v) - height_at(u - e, v)) / (2. * e);
                let dy = (height_at(u, v + e) - height_at(u, v - e)) / (2. * e);
                values.push(unit(crate::math::v(-dx, -dy, 1.)));
            }
        }
        Self {
            width,
            height,
            values,
        }
    }
    pub fn sample(&self, u: f32, v: f32) -> V3 {
        let x = u.rem_euclid(1.) * self.width as f32;
        let y = v.clamp(0., 1.) * (self.height - 1) as f32;
        let ix = x as usize % self.width;
        let iy = y as usize;
        let nx = (ix + 1) % self.width;
        let ny = (iy + 1).min(self.height - 1);
        let tx = x - x.floor();
        let ty = y - y.floor();
        unit(
            (self.values[iy * self.width + ix] * (1. - tx)
                + self.values[iy * self.width + nx] * tx)
                * (1. - ty)
                + (self.values[ny * self.width + ix] * (1. - tx)
                    + self.values[ny * self.width + nx] * tx)
                    * ty,
        )
    }
}
pub struct Materials {
    pub normals: NormalMap,
    pub normal_mapping: bool,
}
impl Materials {
    pub fn new(noise: &Noise) -> Self {
        Self {
            normals: NormalMap::generate(noise),
            normal_mapping: true,
        }
    }
    pub fn shade(&self, f: &Fragment, scene: &Scene, eye: V3, time: f32) -> V3 {
        let p = f.local;
        let q = unit(p);
        let n = &scene.noise;
        let mut normal = f.normal;
        if self.normal_mapping && (1..=3).contains(&f.material) {
            let u = (q.z.atan2(q.x) / TAU).rem_euclid(1.);
            let vv = 0.5 - q.y.clamp(-1., 1.).asin() / PI;
            let mapped = self.normals.sample(u, vv);
            let local_t = if q.y.abs() < 0.999 {
                unit(v(-q.z, 0., q.x))
            } else {
                v(1., 0., 0.)
            };
            let t = if f.object < scene.config.worlds.len() {
                scene.transform(f.object, time as f64).vector(local_t)
            } else {
                local_t
            };
            let b = unit(normal.cross(t));
            let strength = match f.material {
                1 => 0.10,
                2 => 0.35,
                _ => 0.45,
            };
            normal = unit(
                t * mapped.x * strength + b * mapped.y * strength + normal * mapped.z.max(0.2),
            );
        }
        let light = unit(-f.world);
        let view = unit(eye - f.world);
        let day = normal.dot(light).max(0.);
        let half = unit(light + view);
        let diffuse = 0.13 + 0.87 * day;
        let intensity = scene.config.shader_intensity;
        let color = match f.material {
            0 => {
                let granules = n.evolving(q * 28., time * 0.09, 2);
                let hot = smooth(0.36, 0.69, granules);
                let activity = (n.sample(q * 7. + v(time * 0.035, 0., 0.)) * 0.5 + 0.5).powi(5);
                (mix(rgb(0xf1b74d), rgb(0xfff8df), hot) * 1.06 + rgb(0xfff1bd) * activity * 0.3)
                    * intensity.sqrt()
            }
            1 => worlds::forest(q, n, time, day, intensity),
            2 => worlds::volcanic(q, n, time, day, intensity),
            3 => worlds::ice(q, n, normal, light, view),
            4 => worlds::gas(q, n, time, day),
            5 => worlds::desert(q, n, time, day),
            6 => earth(p, normal, light, view, time, n),
            7 => {
                let panel = (p.x * 30.).sin().abs().min((p.z * 24.).sin().abs());
                let metal = mix(rgb(0x718394), rgb(0xc4d5dc), smooth(0.04, 0.1, panel));
                let trim = if p.x.abs() > 0.3 && p.y > 0. { 0.5 } else { 0. };
                metal * (0.27 + day * 0.73)
                    + rgb(0x61f4e2) * trim
                    + rgb(0xd8e8ff) * normal.dot(half).max(0.).powf(30.) * day * 0.7
            }
            8 => {
                let r = (p.x * p.x + p.z * p.z).sqrt();
                let stripes = 0.5 + 0.5 * (r * 185.).sin();
                mix(rgb(0x64517f), rgb(0xd5bd91), stripes) * (0.4 + 0.6 * day)
            }
            9 => {
                let radius = (p.x * p.x + p.z * p.z).sqrt();
                if p.y < 0.45 {
                    rgb(0x5f4935) * diffuse
                } else {
                    let edge = (radius * 30.).sin().abs();
                    mix(rgb(0x34662e), rgb(0xc1d67a), edge * 0.7) * (0.4 + 0.6 * day)
                        + rgb(0xa8e8ab) * edge.powi(14) * (0.15 + 0.06 * (p.y * 8. - time).sin())
                }
            }
            10 => mix(rgb(0x302823), rgb(0xcb5724), smooth(0.8, 1.1, p.y)) * diffuse,
            11 => {
                let facets = normal.dot(half).max(0.).powf(25.);
                mix(rgb(0x829bbb), rgb(0xc6eaf2), p.y.clamp(0., 1.)) * (0.4 + 0.6 * day)
                    + rgb(0xd5bdf5) * facets * 0.55
            }
            _ => {
                let glyph = (p.y * 28.).sin().abs().powi(24) * smooth(0.05, 0.15, p.y);
                rgb(0x3b2f37) * (0.4 + 0.6 * day) + rgb(0xf5c878) * glyph * 0.55
            }
        };
        v(color.x.max(0.), color.y.max(0.), color.z.max(0.))
    }
}
fn earth(p: V3, normal: V3, light: V3, view: V3, time: f32, n: &Noise) -> V3 {
    let r = (p.x * p.x + p.z * p.z).sqrt();
    let day = normal.dot(light).max(0.);
    if p.y < 0.04 {
        let grain =
            n.sample(p * 30.) * 0.3 + 0.35 + 0.2 * (p.y * 120. + n.sample(p * 8.) * 3.).sin();
        return mix(rgb(0x141a28), rgb(0x5a6171), grain) * (0.38 + 0.62 * day);
    }
    let lon = p.z.atan2(p.x).to_degrees();
    let lat = 90. - r * 180.;
    let ground = land(lon + n.sample(p * 18.) * 4., lat + n.sample(p * 23.) * 3.);
    let base = if r > 0.91 {
        mix(rgb(0x9caebf), rgb(0xebfaff), n.sample(p * 52.) * 0.5 + 0.5)
    } else if ground {
        let dry = smooth(0.2, 0.6, n.fbm(p * 5., 2));
        mix(rgb(0x327a5c), rgb(0xc1aa70), dry)
    } else {
        mix(rgb(0x065078), rgb(0x147eaa), n.sample(p * 19.) * 0.5 + 0.5)
    };
    let cloud = smooth(
        -0.15,
        0.4,
        n.sample(v(p.x * 13. + time * 0.022, p.z * 13., time * 0.01)),
    ) * (0.55 + 0.45 * (p.z * 60. + n.sample(p * 17.) * 3.).sin());
    let spec = normal.dot(unit(light + view)).max(0.).powf(50.) * if ground { 0.05 } else { 0.5 };
    mix(base, rgb(0xe3f4ff), cloud * 0.7) * (0.55 + 0.45 * day) + rgb(0x9acde5) * spec * day
}
