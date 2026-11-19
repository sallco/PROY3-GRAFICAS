use crate::{
    config::{Config, Kind},
    geometry::*,
    math::*,
    renderer::DrawCall,
    shaders::procedural::Noise,
};
use std::path::Path;
#[derive(Clone, Copy, Debug)]
pub struct Instance {
    pub transform: Transform,
    pub model: usize,
}
pub struct Scene {
    pub config: Config,
    pub spheres: Vec<Mesh>,
    pub terrain_meshes: Vec<Vec<Mesh>>,
    pub flat: Vec<Mesh>,
    pub models: Vec<Mesh>,
    pub ship: Mesh,
    pub original_ship: Mesh,
    pub rings: Mesh,
    pub instances: Vec<Vec<Instance>>,
    pub noise: Noise,
    pub lod: bool,
    pub levels: Vec<usize>,
}
impl Scene {
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/models");
        let base = Mesh::load(root.join("sphere.obj"))?.sphere();
        let middle = base.subdivide();
        let high = middle.subdivide();
        let close = high.subdivide();
        let spheres = vec![base, middle, high, close];
        let noise = Noise::new(config.seed);
        let terrain_meshes = config
            .worlds
            .iter()
            .map(|w| {
                spheres
                    .iter()
                    .map(|base| {
                        let mut mesh = base.clone();
                        for vertex in &mut mesh.vertices {
                            vertex.position *=
                                crate::terrain::world_height(&noise, w.kind, vertex.position);
                        }
                        if w.kind != Kind::Gas && w.kind != Kind::Flat {
                            mesh.recalculate_normals();
                        }
                        mesh
                    })
                    .collect()
            })
            .collect();
        let mut original_ship = Mesh::load(root.join("nave-original.obj"))?;
        original_ship.normalize();
        let ship_path = root.join("nave-vuelo.obj");
        let mut ship = Mesh::load(if ship_path.exists() {
            ship_path
        } else {
            root.join("nave-original.obj")
        })?;
        ship.normalize();
        let instances = config
            .worlds
            .iter()
            .map(|w| generate_instances(&noise, w.kind, w.instances))
            .collect();
        let levels = vec![0; config.worlds.len()];
        Ok(Self {
            config,
            spheres,
            terrain_meshes,
            flat: vec![disk(32, 8), disk(64, 16), disk(128, 32)],
            models: surface_models(),
            ship,
            original_ship,
            rings: {
                let mut rings = Mesh::default();
                for (inner, outer) in [(1.27, 1.43), (1.49, 1.63), (1.71, 1.85)] {
                    rings.append(&ring(256, inner, outer), Transform::new(V3::zero(), 1., 0.));
                }
                rings
            },
            instances,
            noise,
            lod: true,
            levels,
        })
    }
    pub fn center(&self, i: usize, time: f64) -> V3 {
        let w = &self.config.worlds[i];
        let a = time as f32 * w.orbit_speed + w.phase;
        v(
            a.cos() * w.orbit,
            (a.sin() * w.inclination) * w.orbit,
            a.sin() * w.orbit,
        )
    }
    pub fn velocity(&self, i: usize, time: f64) -> V3 {
        let w = &self.config.worlds[i];
        let a = time as f32 * w.orbit_speed + w.phase;
        v(-a.sin(), a.cos() * w.inclination, a.cos()) * (w.orbit * w.orbit_speed)
    }
    pub fn transform(&self, i: usize, time: f64) -> Transform {
        let w = &self.config.worlds[i];
        Transform::new(self.center(i, time), w.radius, time as f32 * w.spin)
    }
    pub fn update_lod(&mut self, camera: Camera, h: usize, time: f64) {
        for i in 0..self.config.worlds.len() {
            let w = &self.config.worlds[i];
            let pixels = w.radius / (self.center(i, time) - camera.eye).length().max(0.1)
                * h as f32
                / (camera.fov * 0.5).tan();
            let old = self.levels[i];
            self.levels[i] = if !self.lod || pixels > if old >= 3 { 180. } else { 220. } {
                3
            } else if pixels > if old >= 2 { 80. } else { 105. } {
                2
            } else if pixels > if old >= 1 { 25. } else { 34. } {
                1
            } else {
                0
            };
        }
    }
    pub fn calls(&self, time: f64, ship: Option<Transform>, nucleus: bool) -> Vec<DrawCall<'_>> {
        let mut calls = Vec::new();
        if !nucleus {
            calls.push(DrawCall {
                mesh: &self.spheres[3],
                transform: Transform::new(V3::zero(), self.config.star_radius, time as f32 * 0.04),
                material: 0,
                object: usize::MAX,
                radius: self.config.star_radius,
                double_sided: false,
            });
            for i in 0..self.config.worlds.len() {
                self.add_body_calls(&mut calls, i, time, self.levels[i]);
            }
        }
        if let Some(transform) = ship {
            calls.push(DrawCall {
                mesh: &self.ship,
                transform,
                material: 7,
                object: usize::MAX,
                radius: transform.scale,
                double_sided: false,
            });
        }
        calls
    }
    fn add_body_calls<'a>(
        &'a self,
        calls: &mut Vec<DrawCall<'a>>,
        i: usize,
        time: f64,
        level: usize,
    ) {
        let w = &self.config.worlds[i];
        let transform = self.transform(i, time);
        let mesh = match w.kind {
            Kind::Flat => &self.flat[level.min(2)],
            _ => &self.terrain_meshes[i][level],
        };
        calls.push(DrawCall {
            mesh,
            transform,
            material: w.kind.material(),
            object: i,
            radius: w.radius * crate::terrain::surface_bound(w.kind),
            double_sided: false,
        });
        if w.kind == Kind::Gas {
            let mut rt = transform;
            let (s, c) = 0.35_f32.sin_cos();
            rt.y = v(0., c, s);
            rt.z = unit(rt.x.cross(rt.y));
            calls.push(DrawCall {
                mesh: &self.rings,
                transform: rt,
                material: 8,
                object: i,
                radius: w.radius * 1.9,
                double_sided: true,
            });
        }
        if level > 0 {
            for instance in &self.instances[i] {
                let local = instance.transform;
                let global = Transform {
                    position: transform.point(local.position),
                    x: transform.vector(local.x),
                    y: transform.vector(local.y),
                    z: transform.vector(local.z),
                    scale: local.scale,
                };
                calls.push(DrawCall {
                    mesh: &self.models[instance.model],
                    transform: global,
                    material: 9 + instance.model,
                    object: i,
                    radius: local.scale * 2.,
                    double_sided: true,
                });
            }
        }
    }
    pub fn exhibit_calls(
        &self,
        exhibit: crate::museum::Exhibit,
        time: f64,
        original_ship: bool,
    ) -> Vec<DrawCall<'_>> {
        use crate::museum::Exhibit;
        let mut calls = Vec::new();
        match exhibit {
            Exhibit::Sun => calls.push(DrawCall {
                mesh: self.spheres.last().unwrap(),
                transform: Transform::new(V3::zero(), self.config.star_radius, time as f32 * 0.04),
                material: 0,
                object: usize::MAX,
                radius: self.config.star_radius,
                double_sided: false,
            }),
            Exhibit::Planet(i) => self.add_body_calls(&mut calls, i, time, self.spheres.len() - 1),
            Exhibit::Ship => calls.push(DrawCall {
                mesh: if original_ship {
                    &self.original_ship
                } else {
                    &self.ship
                },
                transform: Transform::new(v(-20., 0., 0.), 2., 0.),
                material: 7,
                object: usize::MAX,
                radius: 2.,
                double_sided: false,
            }),
        }
        calls
    }
}
pub fn terrain_height(noise: &Noise, p: V3) -> f32 {
    crate::terrain::world_height(noise, Kind::Volcanic, p)
}
pub fn generate_instances(noise: &Noise, kind: Kind, count: usize) -> Vec<Instance> {
    let model = match kind {
        Kind::Forest => 0,
        Kind::Volcanic => 1,
        Kind::Ice => 2,
        Kind::Desert => 3,
        _ => return Vec::new(),
    };
    let mut out = Vec::new();
    let candidate_count = count * 5;
    let mut candidates: Vec<_> = (0..candidate_count)
        .map(|i| {
            let y = 1. - 2. * (i as f32 + 0.5) / candidate_count as f32;
            let a = i as f32 * 2.399963;
            let r = (1. - y * y).sqrt();
            let direction = v(a.cos() * r, y, a.sin() * r);
            (direction, noise.sample(direction * 53.))
        })
        .collect();
    // Seleccionar por ruido antes de truncar evita concentrar instancias en el norte.
    candidates.sort_by(|a, b| a.1.total_cmp(&b.1));
    for (dir, _) in candidates {
        let suitable = match kind {
            Kind::Forest => noise.fbm(dir * 2.8, 3) >= 0.51,
            Kind::Desert => {
                crate::terrain::salt_basin(dir) < 0.65 && noise.sample(dir * 3.7) >= -0.15
            }
            _ => noise.sample(dir * 3.7) >= -0.15,
        };
        if !suitable {
            continue;
        }
        let radius = crate::terrain::world_height(noise, kind, dir);
        let up = {
            let e = 0.001;
            let axis = if dir.y.abs() < 0.9 {
                v(0., 1., 0.)
            } else {
                v(1., 0., 0.)
            };
            let t = unit(dir.cross(axis));
            let b = dir.cross(t);
            let sample = |d| {
                let d = unit(d);
                d * crate::terrain::world_height(noise, kind, d)
            };
            unit(
                (sample(dir + b * e) - sample(dir - b * e))
                    .cross(sample(dir + t * e) - sample(dir - t * e)),
            )
        };
        let up = if up.dot(dir) < 0. { -up } else { up };
        let axis = if up.y.abs() < 0.9 {
            v(0., 1., 0.)
        } else {
            v(1., 0., 0.)
        };
        let x = unit(axis.cross(up));
        let z = unit(x.cross(up));
        out.push(Instance {
            transform: Transform {
                position: dir * (radius - 0.004),
                x,
                y: up,
                z,
                scale: {
                    let variation = noise.sample(dir * 17.) * 0.5 + 0.5;
                    match kind {
                        Kind::Ice => 0.65 + 0.38 * variation,
                        Kind::Forest => 0.40 + 0.25 * variation,
                        Kind::Desert => 0.40 + 0.30 * variation,
                        _ => 0.32 + 0.26 * variation,
                    }
                },
            },
            model,
        });
        if out.len() == count {
            break;
        }
    }
    out
}
