use crate::{math::*, obj::Obj};
use std::{collections::HashMap, io::Write, path::Path};
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: V3,
    pub normal: V3,
}
#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<[usize; 3]>,
}
impl Mesh {
    pub fn from_obj(obj: Obj) -> Self {
        Self {
            vertices: obj
                .vertices
                .into_iter()
                .zip(obj.normals)
                .map(|(position, normal)| Vertex { position, normal })
                .collect(),
            triangles: obj
                .indices
                .as_chunks::<3>()
                .0
                .iter()
                .map(|x| [x[0], x[1], x[2]])
                .collect(),
        }
    }
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        Ok(Self::from_obj(Obj::load(path)?))
    }
    pub fn normalize(&mut self) {
        let mut lo = v(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut hi = -lo;
        for p in &self.vertices {
            let p = p.position;
            lo = v(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
            hi = v(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
        }
        let center = (lo + hi) * 0.5;
        let s = (hi - lo).length() * 0.5;
        for p in &mut self.vertices {
            p.position = (p.position - center) / s;
        }
    }
    pub fn recalculate_normals(&mut self) {
        for p in &mut self.vertices {
            p.normal = V3::zero();
        }
        for &[a, b, c] in &self.triangles {
            let n = (self.vertices[b].position - self.vertices[a].position)
                .cross(self.vertices[c].position - self.vertices[a].position);
            for i in [a, b, c] {
                self.vertices[i].normal += n;
            }
        }
        for p in &mut self.vertices {
            p.normal = unit(p.normal);
        }
    }
    pub fn sphere(mut self) -> Self {
        for p in &mut self.vertices {
            p.position = unit(p.position);
            p.normal = p.position;
        }
        for t in &mut self.triangles {
            let [a, b, c] = *t;
            if (self.vertices[b].position - self.vertices[a].position)
                .cross(self.vertices[c].position - self.vertices[a].position)
                .dot(self.vertices[a].position)
                < 0.
            {
                t.swap(1, 2);
            }
        }
        self
    }
    pub fn subdivide(&self) -> Self {
        let mut out = self.clone();
        out.triangles.clear();
        let mut edges = HashMap::new();
        for &[a, b, c] in &self.triangles {
            let mut mid = |a: usize, b: usize| -> usize {
                *edges.entry((a.min(b), a.max(b))).or_insert_with(|| {
                    let p = unit((self.vertices[a].position + self.vertices[b].position) * 0.5);
                    let i = out.vertices.len();
                    out.vertices.push(Vertex {
                        position: p,
                        normal: p,
                    });
                    i
                })
            };
            let ab = mid(a, b);
            let bc = mid(b, c);
            let ca = mid(c, a);
            out.triangles
                .extend([[a, ab, ca], [ab, b, bc], [ca, bc, c], [ab, bc, ca]]);
        }
        out
    }
    pub fn append(&mut self, other: &Mesh, t: Transform) {
        let start = self.vertices.len();
        self.vertices.extend(other.vertices.iter().map(|p| Vertex {
            position: t.point(p.position),
            normal: unit(t.vector(p.normal)),
        }));
        self.triangles
            .extend(other.triangles.iter().map(|p| p.map(|i| i + start)));
    }
    pub fn export(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
        writeln!(f, "# Nadir | malla generada por sallco")?;
        for p in &self.vertices {
            writeln!(f, "v {} {} {}", p.position.x, p.position.y, p.position.z)?;
        }
        for &[a, b, c] in &self.triangles {
            writeln!(f, "f {} {} {}", a + 1, b + 1, c + 1)?;
        }
        Ok(())
    }
}
pub fn disk(segments: usize, rings: usize) -> Mesh {
    let mut m = Mesh::default();
    for side in 0..2 {
        let offset = m.vertices.len();
        for j in 0..=rings {
            let r = j as f32 / rings as f32;
            for i in 0..segments {
                let a = i as f32 / segments as f32 * std::f32::consts::TAU;
                let h = if side == 0 {
                    0.07 + 0.055 * (a * 9.).sin().abs() * r.powi(8)
                } else {
                    -0.17 - 0.18 * (1. - r * r) + 0.018 * (a * 13.).sin() * r
                };
                m.vertices.push(Vertex {
                    position: v(r * a.cos(), h, r * a.sin()),
                    normal: v(0., if side == 0 { 1. } else { -1. }, 0.),
                });
            }
        }
        for j in 0..rings {
            for i in 0..segments {
                let a = offset + j * segments + i;
                let b = offset + j * segments + (i + 1) % segments;
                let c = a + segments;
                let d = b + segments;
                let ts = if side == 0 {
                    [[a, b, c], [b, d, c]]
                } else {
                    [[a, c, b], [b, c, d]]
                };
                for [a, b, c] in ts {
                    let pa = m.vertices[a].position;
                    let pb = m.vertices[b].position;
                    let pc = m.vertices[c].position;
                    let cross = (pb - pa).cross(pc - pa);
                    if cross.dot(cross) > 1e-12 {
                        m.triangles.push([a, b, c]);
                    }
                }
            }
        }
    }
    let bottom = (rings + 1) * segments;
    for i in 0..segments {
        let a = rings * segments + i;
        let b = rings * segments + (i + 1) % segments;
        let c = a + bottom;
        let d = b + bottom;
        m.triangles.extend([[a, c, b], [b, c, d]]);
    }
    m.recalculate_normals();
    m
}
pub fn crystal(sides: usize, height: f32) -> Mesh {
    let mut m = Mesh::default();
    for i in 0..sides {
        let a = i as f32 / sides as f32 * std::f32::consts::TAU;
        let b = (i + 1) as f32 / sides as f32 * std::f32::consts::TAU;
        let p = v(a.cos() * 0.22, 0., a.sin() * 0.22);
        let q = v(b.cos() * 0.22, 0., b.sin() * 0.22);
        let top = v(0., height, 0.);
        let n = unit((q - p).cross(top - p));
        let start = m.vertices.len();
        m.vertices.extend([
            Vertex {
                position: p,
                normal: n,
            },
            Vertex {
                position: q,
                normal: n,
            },
            Vertex {
                position: top,
                normal: n,
            },
        ]);
        m.triangles.push([start, start + 2, start + 1]);
    }
    m.recalculate_normals();
    m
}
pub fn ring(segments: usize, inner: f32, outer: f32) -> Mesh {
    let mut m = Mesh::default();
    for i in 0..segments {
        let a = i as f32 / segments as f32 * std::f32::consts::TAU;
        for r in [inner, outer] {
            m.vertices.push(Vertex {
                position: v(a.cos() * r, 0., a.sin() * r),
                normal: v(0., 1., 0.),
            });
        }
    }
    for i in 0..segments {
        let a = i * 2;
        let b = ((i + 1) % segments) * 2;
        m.triangles.extend([[a, b, a + 1], [a + 1, b, b + 1]]);
    }
    m
}
pub fn surface_models() -> Vec<Mesh> {
    let mut tree = crystal(7, 1.3);
    let mut crown = ring(16, 0.35, 0.8);
    for p in &mut crown.vertices {
        p.position.y = 0.9 + 0.16 * (p.position.x * 3.).cos();
    }
    tree.append(&crown, Transform::new(V3::zero(), 1., 0.));
    let basalt = crystal(5, 1.1);
    let mut ice = crystal(6, 1.8);
    ice.append(&crystal(5, 1.), Transform::new(v(0.2, 0., 0.1), 0.7, 0.4));
    let mut monolith = Mesh::default();
    let box_vertices = [
        v(-0.22, 0., -0.12),
        v(0.22, 0., -0.12),
        v(0.22, 1.4, -0.12),
        v(-0.22, 1.4, -0.12),
        v(-0.22, 0., 0.12),
        v(0.22, 0., 0.12),
        v(0.22, 1.4, 0.12),
        v(-0.22, 1.4, 0.12),
    ];
    monolith.vertices = box_vertices
        .map(|position| Vertex {
            position,
            normal: unit(position),
        })
        .to_vec();
    monolith.triangles = vec![
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [3, 7, 6],
        [3, 6, 2],
        [0, 4, 7],
        [0, 7, 3],
        [1, 2, 6],
        [1, 6, 5],
    ];
    monolith.recalculate_normals();
    vec![tree, basalt, ice, monolith]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disk_normals_face_outward() {
        let m = disk(32, 8);
        for p in &m.vertices {
            if p.position.x.abs() < 0.5 && p.position.z.abs() < 0.5 {
                if p.position.y > 0. {
                    assert!(p.normal.y > 0.5);
                } else {
                    assert!(p.normal.y < -0.5);
                }
            }
        }
        assert!(m.triangles.iter().all(|&[a, b, c]| {
            let n = (m.vertices[b].position - m.vertices[a].position)
                .cross(m.vertices[c].position - m.vertices[a].position);
            n.dot(n) > 1e-12
        }));
    }
}
