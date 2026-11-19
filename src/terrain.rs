use crate::math::*;
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
pub fn land(lon: f32, lat: f32) -> bool {
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

pub fn flat_height(p: V3) -> f32 {
    let r = (p.x * p.x + p.z * p.z).sqrt();
    let a = p.z.atan2(p.x);
    let continent = land(a.to_degrees(), 90. - r * 180.);
    let relief = if continent {
        0.008 + 0.009 * (p.x * 43. + (p.z * 21.).sin() * 2.).sin().abs()
    } else {
        0.
    };
    0.07 + relief + 0.055 * (a * 9.).sin().abs() * r.powi(8)
}
