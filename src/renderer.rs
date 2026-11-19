use crate::{
    framebuffer::{Framebuffer, rgba},
    geometry::Mesh,
    math::*,
};
#[derive(Clone, Copy, Debug)]
pub struct Fragment {
    pub local: V3,
    pub world: V3,
    pub normal: V3,
    pub material: usize,
    pub object: usize,
}
#[derive(Clone, Copy)]
struct ClipVertex {
    clip: [f32; 4],
    local: V3,
    world: V3,
    normal: V3,
}
impl ClipVertex {
    fn lerp(self, b: Self, t: f32) -> Self {
        Self {
            clip: std::array::from_fn(|i| self.clip[i] + (b.clip[i] - self.clip[i]) * t),
            local: self.local + (b.local - self.local) * t,
            world: self.world + (b.world - self.world) * t,
            normal: self.normal + (b.normal - self.normal) * t,
        }
    }
}
fn plane(p: ClipVertex, i: usize) -> f32 {
    let [x, y, z, w] = p.clip;
    match i {
        0 => w + x,
        1 => w - x,
        2 => w + y,
        3 => w - y,
        4 => w + z,
        _ => w - z,
    }
}
fn clip_triangle(tri: [ClipVertex; 3]) -> Vec<ClipVertex> {
    let mut input = tri.to_vec();
    let mut output = Vec::with_capacity(9);
    for i in 0..6 {
        output.clear();
        if input.is_empty() {
            break;
        }
        let mut prev = *input.last().unwrap();
        let mut dp = plane(prev, i);
        for &cur in &input {
            let dc = plane(cur, i);
            if (dp >= 0.) != (dc >= 0.) {
                output.push(prev.lerp(cur, dp / (dp - dc)));
            }
            if dc >= 0. {
                output.push(cur);
            }
            prev = cur;
            dp = dc;
        }
        std::mem::swap(&mut input, &mut output);
    }
    input
}
pub struct DrawCall<'a> {
    pub mesh: &'a Mesh,
    pub transform: Transform,
    pub material: usize,
    pub object: usize,
    pub radius: f32,
    pub double_sided: bool,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Metrics {
    pub objects: usize,
    pub culled: usize,
    pub triangles_available: usize,
    pub triangles: usize,
    pub rasterized: usize,
    pub fragments: usize,
    pub render_ms: f64,
}
pub struct Renderer {
    pub fragments: Vec<Option<Fragment>>,
    pub culling: bool,
    pub metrics: Metrics,
}
impl Renderer {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            fragments: vec![None; w * h],
            culling: true,
            metrics: Metrics::default(),
        }
    }
    pub fn visible(camera: Camera, center: V3, radius: f32, aspect: f32) -> bool {
        let (r, u, f) = camera.basis();
        let q = center - camera.eye;
        let z = q.dot(f);
        let ty = (camera.fov * 0.5).tan();
        let tx = ty * aspect;
        z + radius >= camera.near
            && z - radius <= camera.far
            && q.dot(r).abs() <= z * tx + radius * (1. + tx * tx).sqrt()
            && q.dot(u).abs() <= z * ty + radius * (1. + ty * ty).sqrt()
    }
    pub fn rasterize(&mut self, fb: &mut Framebuffer, camera: Camera, calls: &[DrawCall<'_>]) {
        self.fragments.fill(None);
        fb.depths.fill(f32::INFINITY);
        self.metrics = Metrics::default();
        let aspect = fb.width as f32 / fb.height as f32;
        for call in calls {
            self.metrics.objects += 1;
            self.metrics.triangles_available += call.mesh.triangles.len();
            if self.culling && !Self::visible(camera, call.transform.position, call.radius, aspect)
            {
                self.metrics.culled += 1;
                continue;
            }
            let verts: Vec<_> = call
                .mesh
                .vertices
                .iter()
                .map(|p| {
                    let world = call.transform.point(p.position);
                    ClipVertex {
                        clip: camera.clip(world, aspect),
                        local: p.position,
                        world,
                        normal: unit(call.transform.vector(p.normal)),
                    }
                })
                .collect();
            for &[a, b, c] in &call.mesh.triangles {
                self.metrics.triangles += 1;
                let poly = clip_triangle([verts[a], verts[b], verts[c]]);
                for i in 1..poly.len().saturating_sub(1) {
                    self.triangle(fb, [poly[0], poly[i], poly[i + 1]], call);
                }
            }
        }
        self.metrics.fragments = self.fragments.iter().flatten().count();
    }
    fn triangle(&mut self, fb: &mut Framebuffer, tri: [ClipVertex; 3], call: &DrawCall<'_>) {
        let mut p = tri.map(|p| {
            let w = p.clip[3];
            [
                (p.clip[0] / w * 0.5 + 0.5) * fb.width as f32,
                (0.5 - p.clip[1] / w * 0.5) * fb.height as f32,
                p.clip[2] / w,
                1. / w,
            ]
        });
        let mut t = tri;
        let edge = |a: [f32; 4], b: [f32; 4], x: f32, y: f32| {
            (b[0] - a[0]) * (y - a[1]) - (b[1] - a[1]) * (x - a[0])
        };
        let mut area = edge(p[0], p[1], p[2][0], p[2][1]);
        if !area.is_finite() || area.abs() < 1e-6 {
            return;
        }
        if area >= 0. && !call.double_sided {
            return;
        }
        if area < 0. {
            p.swap(1, 2);
            t.swap(1, 2);
            area = -area;
        }
        self.metrics.rasterized += 1;
        let minx = p
            .iter()
            .map(|a| a[0])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.) as usize;
        let maxx = p
            .iter()
            .map(|a| a[0])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(fb.width as f32 - 1.) as usize;
        let miny = p
            .iter()
            .map(|a| a[1])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.) as usize;
        let maxy = p
            .iter()
            .map(|a| a[1])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(fb.height as f32 - 1.) as usize;
        for y in miny..=maxy {
            for x in minx..=maxx {
                let x0 = x as f32 + 0.5;
                let y0 = y as f32 + 0.5;
                let a = edge(p[1], p[2], x0, y0) / area;
                let b = edge(p[2], p[0], x0, y0) / area;
                let c = 1. - a - b;
                if a < 0. || b < 0. || c < 0. {
                    continue;
                }
                let index = y * fb.width + x;
                let depth = a * p[0][2] + b * p[1][2] + c * p[2][2];
                if depth >= fb.depths[index] {
                    continue;
                }
                let den = a * p[0][3] + b * p[1][3] + c * p[2][3];
                let wa = a * p[0][3] / den;
                let wb = b * p[1][3] / den;
                let wc = c * p[2][3] / den;
                fb.depths[index] = depth;
                self.fragments[index] = Some(Fragment {
                    local: t[0].local * wa + t[1].local * wb + t[2].local * wc,
                    world: t[0].world * wa + t[1].world * wb + t[2].world * wc,
                    normal: unit(t[0].normal * wa + t[1].normal * wb + t[2].normal * wc),
                    material: call.material,
                    object: call.object,
                });
            }
        }
    }
    pub fn shade(&self, fb: &mut Framebuffer, shader: impl Fn(&Fragment) -> V3 + Sync) {
        let workers = std::thread::available_parallelism()
            .map_or(1, |n| n.get())
            .min(8);
        let chunk = self.fragments.len().div_ceil(workers);
        std::thread::scope(|s| {
            for (pixels, frags) in fb
                .pixels
                .chunks_mut(chunk * 4)
                .zip(self.fragments.chunks(chunk))
            {
                let shader = &shader;
                s.spawn(move || {
                    for (pixel, fragment) in pixels.chunks_exact_mut(4).zip(frags) {
                        if let Some(f) = fragment {
                            pixel.copy_from_slice(&rgba(shader(f)));
                        }
                    }
                });
            }
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clipping_preserves_visible_polygon() {
        let cv = |x, y, z, w| ClipVertex {
            clip: [x, y, z, w],
            local: v(x, y, z),
            world: v(x, y, z),
            normal: v(0., 0., 1.),
        };
        let p = clip_triangle([
            cv(-0.5, 0., 0., 1.),
            cv(0.5, 0., 0., 1.),
            cv(0., 1., -2., 1.),
        ]);
        assert_eq!(p.len(), 4);
        for v in p {
            for i in 0..6 {
                assert!(plane(v, i) >= -1e-5);
            }
        }
    }
    #[test]
    fn frustum_rejects_behind_camera() {
        let c = Camera::new(v(0., 0., 0.), v(0., 0., -1.));
        assert!(!Renderer::visible(c, v(0., 0., 4.), 1., 1.));
        assert!(Renderer::visible(c, v(0., 0., -4.), 1., 1.));
    }
}
