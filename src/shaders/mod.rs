pub mod procedural;
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
        let width = 128;
        let height = 64;
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
        if self.normal_mapping && (1..=5).contains(&f.material) {
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
            normal = unit(t * mapped.x * 0.3 + b * mapped.y * 0.3 + normal * mapped.z.max(0.2));
        }
        let light = unit(-f.world);
        let view = unit(eye - f.world);
        let day = normal.dot(light).max(0.);
        let half = unit(light + view);
        let diffuse = 0.13 + 0.87 * day;
        let intensity = scene.config.shader_intensity;
        let color = match f.material {
            0 => {
                let cells = n.evolving(q * 5., time * 0.09, 2);
                let fil = 1.
                    - smooth(
                        0.015,
                        0.12,
                        n.sample(q * 15. + v(time * 0.07, 0., 0.)).abs(),
                    );
                mix(rgb(0x251b67), rgb(0xd0e6ff), cells.powf(0.7)) + rgb(0xe9d4ff) * fil * 0.65
            }
            1 => {
                let land = n.fbm(q * 4., 2);
                let threads = 1. - smooth(0.015, 0.065, n.sample(q * 22.).abs());
                let pulse = 0.6 + 0.4 * (q.y * 14. + time * 0.8).sin();
                mix(rgb(0x081d28), rgb(0x2a5c51), land) * diffuse
                    + rgb(0x45f4ce) * threads * pulse * (0.25 + (1. - day) * 0.65) * intensity
            }
            2 => {
                let w = n.warp(q * 4.3, 0.6);
                let fault = n.sample(w).abs();
                let lava = 1. - smooth(0.018, 0.085, fault);
                let flow = 0.5
                    + 0.5 * (q.dot(v(14., -9., 17.)) - time * 2.5 + n.sample(q * 9.) * 2.).sin();
                let rock =
                    mix(rgb(0x151b29), rgb(0x504658), n.sample(q * 29.) * 0.5 + 0.5) * diffuse;
                mix(
                    rock,
                    mix(rgb(0xb62c4c), rgb(0xffd69a), flow) * intensity,
                    lava,
                )
            }
            3 => {
                let cracks = 1.
                    - smooth(
                        0.008,
                        0.05,
                        (n.sample(q * 7.) + 0.12 * (q.y * 45.).sin()).abs(),
                    );
                let sheen = normal.dot(half).max(0.).powf(45.) * day;
                mix(rgb(0x25466a), rgb(0xa7dce8), n.fbm(q * 3., 2)) * diffuse
                    + rgb(0x8af9f0) * cracks * 0.25
                    + rgb(0xf0efff) * sheen * 0.8
            }
            4 => {
                let mut a = q.z.atan2(q.x);
                let lat = q.y;
                let da = (a - 0.6 + PI).rem_euclid(TAU) - PI;
                let dy = lat - 0.2;
                let r2 = da * da + dy * dy * 5.;
                let vortex = (-r2 * 6.).exp() * 2.5 * (0.7 + 0.3 * (time * 0.5).sin());
                a += vortex;
                let pp = v(
                    a.cos() * (1. - lat * lat).sqrt(),
                    lat,
                    a.sin() * (1. - lat * lat).sqrt(),
                );
                let bands = 0.5
                    + 0.5
                        * (lat * 48.
                            + n.sample(pp * 7. + v(time * 0.08, 0., time * 0.03)) * 3.
                            + vortex * 4.)
                            .sin();
                let base = mix(rgb(0x34402b), rgb(0xe1ce65), bands);
                base * (0.3 + 0.7 * day) + rgb(0xdba660) * (-r2 * 14.).exp() * 0.35
            }
            5 => {
                let dune = 0.5 + 0.5 * (q.y * 75. + n.sample(q * 4.) * 8.).sin();
                let interference = (q.dot(v(22., 4., 11.)) + time * 0.12).sin().abs().powi(18);
                mix(
                    rgb(0x67475e),
                    rgb(0xdeb08d),
                    dune * 0.55 + n.fbm(q * 5., 2) * 0.45,
                ) * diffuse
                    + rgb(0xa99aec) * interference * 0.08
            }
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
            9 => rgb(0x123b42) * diffuse + rgb(0x54d9bd) * (0.35 + 0.2 * (p.y * 8. - time).sin()),
            10 => mix(rgb(0x282338), rgb(0xe07668), smooth(0.65, 1.1, p.y)) * diffuse,
            11 => {
                rgb(0x9bdce5) * (0.3 + 0.7 * day)
                    + rgb(0xd1d1ff) * normal.dot(half).max(0.).powf(18.) * 0.4
            }
            _ => rgb(0x88718d) * diffuse + rgb(0xc8b5ff) * (p.y * 24.).sin().abs().powi(24) * 0.15,
        };
        v(color.x.max(0.), color.y.max(0.), color.z.max(0.))
    }
}
fn polygon(x: f32, y: f32, points: &[(f32, f32)]) -> bool {
    let mut inside = false;
    let mut j = points.len() - 1;
    for i in 0..points.len() {
        let (a, b) = points[i];
        let (c, d) = points[j];
        if (b > y) != (d > y) && x < (c - a) * (y - b) / (d - b) + a {
            inside = !inside;
        }
        j = i;
    }
    inside
}
fn land(lon: f32, lat: f32) -> bool {
    const NA: &[(f32, f32)] = &[
        (-168., 70.),
        (-140., 60.),
        (-125., 48.),
        (-117., 32.),
        (-100., 16.),
        (-83., 10.),
        (-80., 26.),
        (-60., 47.),
        (-54., 60.),
        (-80., 73.),
        (-110., 77.),
    ];
    const SA: &[(f32, f32)] = &[
        (-80., 10.),
        (-62., 10.),
        (-35., -8.),
        (-45., -25.),
        (-68., -56.),
        (-75., -35.),
    ];
    const AFR: &[(f32, f32)] = &[
        (-17., 35.),
        (10., 38.),
        (33., 30.),
        (51., 10.),
        (40., -15.),
        (20., -35.),
        (8., -25.),
        (-6., 5.),
        (-17., 15.),
    ];
    const EURASIA: &[(f32, f32)] = &[
        (-10., 36.),
        (-10., 58.),
        (20., 70.),
        (75., 75.),
        (140., 65.),
        (179., 67.),
        (179., 52.),
        (140., 45.),
        (130., 25.),
        (108., 0.),
        (98., 15.),
        (80., 8.),
        (65., 25.),
        (40., 35.),
        (30., 42.),
        (15., 37.),
    ];
    const AUS: &[(f32, f32)] = &[
        (113., -22.),
        (130., -11.),
        (145., -14.),
        (153., -28.),
        (142., -39.),
        (115., -34.),
    ];
    polygon(lon, lat, NA)
        || polygon(lon, lat, SA)
        || polygon(lon, lat, AFR)
        || polygon(lon, lat, EURASIA)
        || polygon(lon, lat, AUS)
        || ((lon + 42.) / 18.).powi(2) + ((lat - 74.) / 10.).powi(2) < 1.
}
fn earth(p: V3, normal: V3, light: V3, view: V3, time: f32, n: &Noise) -> V3 {
    let r = (p.x * p.x + p.z * p.z).sqrt();
    let day = normal.dot(light).max(0.);
    if p.y < 0.04 {
        let grain = n.sample(p * 30.) * 0.5 + 0.5;
        return mix(rgb(0x141a28), rgb(0x5a6171), grain) * (0.2 + 0.8 * day);
    }
    let lon = p.z.atan2(p.x).to_degrees();
    let lat = 90. - r * 180.;
    let ground = land(lon + n.sample(p * 18.) * 2., lat + n.sample(p * 23.) * 1.5);
    let base = if r > 0.91 {
        mix(rgb(0x9caebf), rgb(0xebfaff), n.sample(p * 52.) * 0.5 + 0.5)
    } else if ground {
        let dry = smooth(0.2, 0.6, n.fbm(p * 5., 2));
        mix(rgb(0x327a5c), rgb(0xc1aa70), dry)
    } else {
        mix(rgb(0x065078), rgb(0x147eaa), n.sample(p * 19.) * 0.5 + 0.5)
    };
    let cloud = smooth(
        0.25,
        0.65,
        n.sample(v(p.x * 13. + time * 0.022, p.z * 13., time * 0.01)),
    ) * (0.55 + 0.45 * (p.z * 60. + n.sample(p * 17.) * 3.).sin());
    let spec = normal.dot(unit(light + view)).max(0.).powf(50.) * if ground { 0.05 } else { 0.5 };
    mix(base, rgb(0xe3f4ff), cloud * 0.7) * (0.22 + 0.78 * day) + rgb(0x9acde5) * spec * day
}
