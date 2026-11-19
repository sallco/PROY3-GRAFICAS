//! Volumen convexo que acompaña la base cónica del disco terrestre.
use crate::math::*;
/// Devuelve punto resuelto y normal. Expande cono, borde y tapa por el radio de nave.
pub fn contact(from: V3, to: V3, radius: f32, padding: f32) -> Option<(V3, V3)> {
    let slope = 0.54;
    let length = (1_f32 + slope * slope).sqrt();
    let tip = -0.73 * radius - padding * length;
    let top = 0.15 * radius + padding;
    let outer = radius + padding;
    let radial = |p: V3| (p.x * p.x + p.z * p.z).sqrt();
    let cone_normal = |p: V3| {
        let r = radial(p);
        if r > 1e-7 {
            v(slope * p.x / r, -1., slope * p.z / r) / length
        } else {
            v(slope, -1., 0.) / length
        }
    };
    let r = radial(from);
    if r < outer && from.y < top && from.y > tip + slope * r {
        let cap_distance = top - from.y;
        let wall_distance = outer - r;
        let cone_distance = (from.y - tip - slope * r) / length;
        let (distance, normal) = if cap_distance < wall_distance && cap_distance < cone_distance {
            (cap_distance, v(0., 1., 0.))
        } else if wall_distance < cone_distance {
            (wall_distance, unit(v(from.x, 0., from.z)))
        } else {
            (cone_distance, cone_normal(from))
        };
        return Some((from + normal * (distance + 0.002), normal));
    }
    let d = to - from;
    let mut best: Option<(f32, V3)> = None;
    let mut add = |t: f32, n: V3| {
        if (0.0..=1.).contains(&t) && d.dot(n) <= 1e-6 && best.is_none_or(|b| t < b.0) {
            best = Some((t, n));
        }
    };
    if d.y.abs() > 1e-8 {
        let t = (top - from.y) / d.y;
        if radial(from + d * t) <= outer {
            add(t, v(0., 1., 0.));
        }
    }
    let roots = |a: f64, b: f64, c: f64| -> Vec<f32> {
        if a.abs() < 1e-9 {
            if b.abs() > 1e-9 {
                vec![(-c / (2. * b)) as f32]
            } else {
                vec![]
            }
        } else {
            let disc = b * b - a * c;
            let tolerance = (b * b + (a * c).abs()) * 16. * f64::EPSILON;
            if disc < -tolerance {
                vec![]
            } else {
                let s = disc.max(0.).sqrt();
                vec![((-b - s) / a) as f32, ((-b + s) / a) as f32]
            }
        }
    };
    // El discriminante en doble precisión evita perder impactos axiales en la punta.
    let (dx, dy, dz) = (d.x as f64, d.y as f64, d.z as f64);
    let (fx, fy, fz) = (from.x as f64, from.y as f64, from.z as f64);
    for t in roots(
        dx * dx + dz * dz,
        fx * dx + fz * dz,
        fx * fx + fz * fz - (outer as f64).powi(2),
    ) {
        let p = from + d * t;
        if p.y >= tip + slope * outer && p.y <= top {
            add(t, unit(v(p.x, 0., p.z)));
        }
    }
    let k2 = (slope as f64).powi(2);
    let height = fy - tip as f64;
    for t in roots(
        dx * dx + dz * dz - dy * dy / k2,
        fx * dx + fz * dz - height * dy / k2,
        fx * fx + fz * fz - height * height / k2,
    ) {
        let p = from + d * t;
        if p.y >= tip - 1e-4 && p.y <= top && radial(p) <= outer + 1e-4 {
            add(t, cone_normal(p));
        }
    }
    best.map(|(t, n)| (from + d * t + n * 0.002, n))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn top_rim_and_tip_block_fast_segments() {
        for (from, to) in [
            (v(0., 15., 0.), v(0., -15., 0.)),
            (v(15., 0., 0.), v(-15., 0., 0.)),
            (v(0., -15., 0.), v(0., 15., 0.)),
            (v(0., -20., 0.), v(0., 20., 0.)),
            (v(0., -100., 0.), v(0., 100., 0.)),
        ] {
            let (p, n) = contact(from, to, 6., 0.32).unwrap();
            assert!(p.y >= 1.21 || p.x > 6.31 || p.y < -4.7);
            assert!(n.dot(n) > 0.99);
        }
    }
    #[test]
    fn empty_space_under_outer_rim_remains_accessible() {
        assert!(contact(v(5., -3., 1.), v(5., -3., -1.), 6., 0.1).is_none());
    }
    #[test]
    fn interior_is_ejected_to_a_boundary() {
        let (p, _) = contact(v(0., 0., 0.), v(0., 0., 0.), 6., 0.32).unwrap();
        assert!(contact(p, p, 6., 0.32).is_none());
    }
}
