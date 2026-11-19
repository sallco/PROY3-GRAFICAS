use crate::{config::Kind, math::*, scene::Scene};
pub const STEP: f32 = 1. / 120.;
const FIXED_DT: f64 = 1. / 120.;
pub const SHIP_RADIUS: f32 = 0.32;
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub thrust: f32,
    pub strafe: f32,
    pub lift: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub boost: bool,
    pub brake: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Ship {
    pub position: V3,
    pub velocity: V3,
    pub forward: V3,
    pub up: V3,
    pub angular: V3,
}
impl Ship {
    pub fn new(position: V3, forward: V3) -> Self {
        Self {
            position,
            velocity: V3::zero(),
            forward: unit(forward),
            up: v(0., 1., 0.),
            angular: V3::zero(),
        }
    }
    pub fn transform(self) -> Transform {
        let x = unit(self.forward.cross(self.up));
        let y = unit(x.cross(self.forward));
        Transform {
            position: self.position,
            x,
            y,
            z: -self.forward,
            scale: SHIP_RADIUS,
        }
    }
}
fn rotate(p: V3, axis: V3, angle: f32) -> V3 {
    let (s, c) = angle.sin_cos();
    p * c + axis.cross(p) * s + axis * (axis.dot(p) * (1. - c))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    System,
    Nucleus,
}
pub struct Flight {
    pub time: f64,
    pub ship: Ship,
    pub previous: Ship,
    pub camera: Camera,
    pub region: Region,
    pub paused: bool,
    pub local: bool,
    pub target: usize,
    pub warp: Option<f32>,
    pub accumulator: f64,
    saved: Option<Ship>,
    pub contacts: u64,
}
impl Flight {
    pub fn new(scene: &Scene) -> Self {
        let center = scene.center(0, 0.);
        let ship = Ship::new(center + v(0., 2.8, 9.5), v(0., -0.16, -1.));
        let camera = Camera::new(
            ship.position - ship.forward * 2.8 + ship.up * 1.05,
            ship.position + ship.forward * 5.,
        );
        Self {
            time: 0.,
            ship,
            previous: ship,
            camera,
            region: Region::System,
            paused: false,
            local: false,
            target: 0,
            warp: None,
            accumulator: 0.,
            saved: None,
            contacts: 0,
        }
    }
    pub fn jump(&mut self) {
        if self.warp.is_none() {
            self.warp = Some(0.);
        }
    }
    pub fn nearest(&self, scene: &Scene) -> usize {
        (0..scene.config.worlds.len())
            .min_by(|&a, &b| {
                let da = (self.ship.position - scene.center(a, self.time)).length()
                    - scene.config.worlds[a].radius;
                let db = (self.ship.position - scene.center(b, self.time)).length()
                    - scene.config.worlds[b].radius;
                da.total_cmp(&db)
            })
            .unwrap_or(0)
    }
    pub fn advance(&mut self, scene: &Scene, dt: f32, input: Input) {
        let dt = dt.clamp(0., 0.1);
        if !self.paused {
            self.accumulator += dt as f64;
            while self.accumulator + 1e-10 >= FIXED_DT {
                self.previous = self.ship;
                self.tick(scene, input);
                self.accumulator -= FIXED_DT;
            }
            if let Some(t) = self.warp.as_mut() {
                *t += dt;
                if *t >= 1.5 && *t - dt < 1.5 {
                    match self.region {
                        Region::System => {
                            self.saved = Some(self.ship);
                            self.region = Region::Nucleus;
                            self.ship = Ship::new(v(0., 8., 35.), v(0., -0.16, -1.));
                        }
                        Region::Nucleus => {
                            self.region = Region::System;
                            if let Some(s) = self.saved {
                                self.ship = s;
                            }
                            self.ship.velocity = V3::zero();
                        }
                    }
                    let p = self.ship.position;
                    self.ship.position =
                        resolve_scene(scene, self.region, self.time, self.time, p, p, SHIP_RADIUS)
                            .0;
                    self.previous = self.ship;
                    self.camera = Camera::new(
                        self.ship.position - self.ship.forward * 2.8 + self.ship.up * 1.05,
                        self.ship.position + self.ship.forward * 5.,
                    );
                }
                if self.warp.is_some_and(|t| t >= 3.) {
                    self.warp = None;
                }
            }
        }
        let alpha = (self.accumulator / FIXED_DT).clamp(0., 1.) as f32;
        let position =
            self.previous.position + (self.ship.position - self.previous.position) * alpha;
        let desired = position - self.ship.forward * 2.8 + self.ship.up * 1.05;
        let desired = resolve_scene(
            scene,
            self.region,
            self.time,
            self.time,
            position,
            desired,
            0.16,
        )
        .0;
        self.camera.eye += (desired - self.camera.eye) * (1. - (-dt * 9.).exp());
        self.camera.eye = resolve_scene(
            scene,
            self.region,
            self.time,
            self.time,
            self.camera.eye,
            self.camera.eye,
            0.16,
        )
        .0;
        self.camera.target = position + self.ship.forward * 5.;
        self.camera.up =
            unit(self.camera.up + (self.ship.up - self.camera.up) * (1. - (-dt * 7.).exp()));
    }
    fn tick(&mut self, scene: &Scene, input: Input) {
        let old_time = self.time;
        self.time += FIXED_DT;
        if self.warp.is_some() {
            self.ship.velocity *= (-STEP * 3.).exp();
            return;
        }
        let nearest = self.nearest(scene);
        let center = scene.center(nearest, self.time);
        let w = &scene.config.worlds[nearest];
        let radial = unit(self.ship.position - center);
        let near =
            self.region == Region::System && (self.ship.position - center).length() < w.radius * 4.;
        let assist = self.local && near;
        let reference = if assist { radial } else { self.ship.up };
        let desired_angular = v(input.pitch, input.yaw, input.roll) * 1.4;
        self.ship.angular += (desired_angular - self.ship.angular) * (1. - (-STEP * 7.).exp());
        let right = unit(self.ship.forward.cross(self.ship.up));
        self.ship.forward = unit(rotate(
            self.ship.forward,
            reference,
            -self.ship.angular.y * STEP,
        ));
        self.ship.forward = unit(rotate(self.ship.forward, right, self.ship.angular.x * STEP));
        self.ship.up = unit(rotate(self.ship.up, right, self.ship.angular.x * STEP));
        self.ship.up = unit(rotate(
            self.ship.up,
            self.ship.forward,
            self.ship.angular.z * STEP,
        ));
        if assist {
            self.ship.up =
                unit(self.ship.up + (radial - self.ship.up) * (1. - (-STEP * 1.5).exp()));
        }
        let right = unit(self.ship.forward.cross(self.ship.up));
        self.ship.up = unit(right.cross(self.ship.forward));
        let max_speed = if assist {
            4.5
        } else if input.boost {
            45.
        } else {
            15.
        };
        let drive = self.ship.forward * input.thrust
            + right * input.strafe
            + if assist {
                radial * input.lift
            } else {
                self.ship.up * input.lift
            };
        let mut acceleration = if drive.dot(drive) > 1. {
            unit(drive)
        } else {
            drive
        } * if assist { 4. } else { 10. };
        if self.region == Region::System {
            for i in 0..scene.config.worlds.len() {
                let d = scene.center(i, self.time) - self.ship.position;
                let r = scene.config.worlds[i].radius;
                let d2 = d.dot(d).max(r * r);
                acceleration += unit(d) * (r * r * 0.35 / d2).min(0.5);
            }
        } else {
            let d = -self.ship.position;
            acceleration += unit(d) * (25. / d.dot(d).max(100.));
        }
        self.ship.velocity += acceleration * STEP;
        let reference_velocity = if assist {
            scene.velocity(nearest, self.time)
        } else {
            V3::zero()
        };
        let mut relative = self.ship.velocity - reference_velocity;
        relative *= (-STEP
            * if input.brake {
                6.
            } else if assist {
                0.55
            } else {
                0.12
            })
        .exp();
        if relative.length() > max_speed {
            relative = unit(relative) * max_speed;
        }
        self.ship.velocity = reference_velocity + relative;
        let from = self.ship.position;
        let to = from + self.ship.velocity * STEP;
        let (position, normal) = resolve_scene(
            scene,
            self.region,
            old_time,
            self.time,
            from,
            to,
            SHIP_RADIUS,
        );
        self.ship.position = position;
        if let Some(n) = normal {
            self.contacts += 1;
            let body_velocity = if self.region == Region::System {
                scene.velocity(nearest, self.time)
            } else {
                V3::zero()
            };
            let inward = (self.ship.velocity - body_velocity).dot(n);
            if inward < 0. {
                self.ship.velocity -= n * inward;
            }
        }
    }
}
/// Segmento contra esfera expandida: incluye el caso de origen interior.
pub fn sphere_contact(from: V3, to: V3, radius: f32) -> Option<(f32, V3)> {
    if from.dot(from) < radius * radius {
        return Some((0., unit(from)));
    }
    let d = to - from;
    let a = d.dot(d);
    if a < 1e-12 {
        return None;
    }
    let b = from.dot(d);
    let c = from.dot(from) - radius * radius;
    let disc = b * b - a * c;
    if disc < 0. {
        return None;
    }
    let t = (-b - disc.sqrt()) / a;
    if (0.0..=1.).contains(&t) {
        Some((t, unit(from + d * t)))
    } else {
        None
    }
}
/// Cilindro cerrado expandido: tapas, pared y expulsión por la salida más próxima.
pub fn cylinder_contact(from: V3, to: V3, r: f32, bottom: f32, top: f32) -> Option<(f32, V3)> {
    let radial = (from.x * from.x + from.z * from.z).sqrt();
    if radial < r && from.y > bottom && from.y < top {
        let side = r - radial;
        let up = top - from.y;
        let down = from.y - bottom;
        return Some((
            0.,
            if up < side && up < down {
                v(0., 1., 0.)
            } else if down < side {
                v(0., -1., 0.)
            } else {
                unit(v(from.x, 0., from.z))
            },
        ));
    }
    let d = to - from;
    let mut best: Option<(f32, V3)> = None;
    let mut add = |t: f32, n: V3| {
        if (0.0..=1.).contains(&t) && best.is_none_or(|b| t < b.0) {
            best = Some((t, n));
        }
    };
    if d.y.abs() > 1e-8 {
        for (h, n) in [(bottom, v(0., -1., 0.)), (top, v(0., 1., 0.))] {
            let t = (h - from.y) / d.y;
            let p = from + d * t;
            if p.x * p.x + p.z * p.z <= r * r {
                add(t, n);
            }
        }
    }
    let a = d.x * d.x + d.z * d.z;
    let b = from.x * d.x + from.z * d.z;
    let c = from.x * from.x + from.z * from.z - r * r;
    let disc = b * b - a * c;
    if a > 1e-10 && disc >= 0. {
        for t in [(-b - disc.sqrt()) / a, (-b + disc.sqrt()) / a] {
            let p = from + d * t;
            if p.y >= bottom && p.y <= top {
                add(t, unit(v(p.x, 0., p.z)));
            }
        }
    }
    best
}
pub fn resolve_scene(
    scene: &Scene,
    region: Region,
    old_time: f64,
    time: f64,
    from: V3,
    to: V3,
    padding: f32,
) -> (V3, Option<V3>) {
    if region == Region::Nucleus {
        return resolve_sphere(from, to, V3::zero(), V3::zero(), 7.5 + padding);
    }
    let mut position = to;
    let mut contact = None;
    for _ in 0..2 {
        let (p, n) = resolve_sphere(
            from,
            position,
            V3::zero(),
            V3::zero(),
            scene.config.star_radius + padding,
        );
        position = p;
        contact = contact.or(n);
        for i in 0..scene.config.worlds.len() {
            let w = &scene.config.worlds[i];
            let old = scene.center(i, old_time);
            let now = scene.center(i, time);
            if w.kind == Kind::Flat {
                let r = w.radius + padding;
                let bottom = -0.38 * w.radius - padding;
                let top = 0.14 * w.radius + padding;
                let f = from - old;
                let t = position - now;
                if let Some((hit, n)) = cylinder_contact(f, t, r, bottom, top) {
                    let q = f + (t - f) * hit;
                    let boundary = if n.y > 0.5 {
                        v(q.x, top + 0.002, q.z)
                    } else if n.y < -0.5 {
                        v(q.x, bottom - 0.002, q.z)
                    } else {
                        n * (r + 0.002) + v(0., q.y, 0.)
                    };
                    position = now + boundary;
                    contact = Some(n);
                }
            } else {
                let radius = w.radius * if w.kind == Kind::Volcanic { 1.08 } else { 1. };
                let (p, n) = resolve_sphere(from, position, old, now, radius + padding);
                position = p;
                contact = contact.or(n);
            }
            if (position - now).length() < w.radius + 2. {
                let transform = scene.transform(i, time);
                let old_transform = scene.transform(i, old_time);
                for instance in &scene.instances[i] {
                    let p = instance.transform;
                    let local = p.position + p.y * (p.scale * 0.7 / w.radius);
                    let now_p = transform.point(local);
                    let old_p = old_transform.point(local);
                    let (p, n) =
                        resolve_sphere(from, position, old_p, now_p, p.scale * 1.2 + padding);
                    position = p;
                    contact = contact.or(n);
                }
            }
        }
    }
    (position, contact)
}
fn resolve_sphere(from: V3, to: V3, old_center: V3, center: V3, radius: f32) -> (V3, Option<V3>) {
    let f = from - old_center;
    let t = to - center;
    if let Some((_, normal)) = sphere_contact(f, t, radius) {
        let n = if t.length() < radius { unit(t) } else { normal };
        (center + n * (radius + 0.002), Some(n))
    } else {
        (to, None)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fast_segment_hits_sphere() {
        let hit = sphere_contact(v(-20., 0., 0.), v(20., 0., 0.), 2.).unwrap();
        assert!((hit.0 - 0.45).abs() < 1e-5);
        assert_eq!(hit.1, v(-1., 0., 0.));
    }
    #[test]
    fn cylinder_handles_caps_sides_and_inside() {
        assert_eq!(
            cylinder_contact(v(0., 10., 0.), v(0., -10., 0.), 2., -1., 1.)
                .unwrap()
                .1,
            v(0., 1., 0.)
        );
        assert_eq!(
            cylinder_contact(v(10., 0., 0.), v(-10., 0., 0.), 2., -1., 1.)
                .unwrap()
                .1,
            v(1., 0., 0.)
        );
        assert!(cylinder_contact(v(0., 0., 0.), v(0., 0., 0.), 2., -1., 1.).is_some());
        assert!(cylinder_contact(v(3., 2., 0.), v(3., -2., 0.), 2., -1., 1.).is_none());
    }
}
