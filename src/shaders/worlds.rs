use super::procedural::{Noise, mix, rgb, smooth};
use crate::{
    math::*,
    terrain::{salt_basin, volcanic_vents},
};
use std::f32::consts::{PI, TAU};

pub fn forest(q: V3, n: &Noise, time: f32, day: f32, intensity: f32) -> V3 {
    let continents = n.fbm(q * 2.8, 3);
    let shore = smooth(0.43, 0.51, continents);
    let moss = n.sample(q * 46.) * 0.5 + 0.5;
    let canopy = mix(
        rgb(0x164a35),
        rgb(0x90b95f),
        moss * 0.45 + continents * 0.55,
    );
    let water = mix(rgb(0x201b44), rgb(0x645077), n.sample(q * 15.) * 0.5 + 0.5);
    let roots = (1. - smooth(0.008, 0.027, n.sample(q * 28.).abs())) * shore;
    let pulse = 0.55 + 0.45 * (q.y * 17. + time * 0.85).sin();
    let coast = (1. - (shore * 2. - 1.).abs()).powi(3);
    mix(water, canopy, shore) * (0.30 + 0.70 * day)
        + rgb(0x9de5a0) * roots * pulse * (0.12 + 0.25 * (1. - day)) * intensity
        + rgb(0x9ebc83) * coast * 0.10
}
pub fn volcanic(q: V3, n: &Noise, time: f32, day: f32, intensity: f32) -> V3 {
    let warped = n.warp(q * 4.7, 0.7);
    let fractures = n.sample(warped).abs();
    let (cone, crater) = volcanic_vents(q);
    let lava = (1. - smooth(0.012, 0.067, fractures)).max(crater * 0.95);
    let grain = n.sample(q * 74.) * 0.5 + 0.5;
    let ash = n.sample(q * 3.) * 0.5 + 0.5;
    let rock = mix(rgb(0x17191c), rgb(0x634a3e), grain * 0.45 + ash * 0.55) * (0.26 + 0.74 * day);
    let rim = cone * (1. - crater);
    let flow = 0.5 + 0.5 * (q.dot(v(24., -15., 13.)) - time * 2.6 + grain * 2.).sin();
    let molten = mix(rgb(0xb62b12), rgb(0xffdc71), flow.powf(0.7)) * intensity;
    mix(rock + rgb(0x8b4b30) * rim * 0.2, molten, lava)
}
pub fn ice(q: V3, n: &Noise, normal: V3, light: V3, view: V3) -> V3 {
    let field = n.sample(q * 6.7);
    let fissure = 1. - smooth(0.012, 0.075, field.abs());
    let terraces = 0.5 + 0.5 * (q.y * 90. + n.sample(q * 12.) * 3.).sin();
    let snow = mix(
        rgb(0xa5cfda),
        rgb(0xf5f1e9),
        n.sample(q * 26.) * 0.25 + 0.65,
    );
    let deep = mix(rgb(0x122e62), rgb(0x438aaf), terraces);
    let day = normal.dot(light).max(0.);
    let specular = normal.dot(unit(light + view)).max(0.).powf(60.) * day;
    let fresnel = (1. - normal.dot(view).abs()).powi(4);
    mix(snow, deep, fissure * 0.90) * (0.35 + 0.65 * day)
        + rgb(0xfff6eb) * specular * 0.85
        + rgb(0x9c8cd9) * fresnel * 0.18
}
pub fn gas(q: V3, n: &Noise, time: f32, day: f32) -> V3 {
    let longitude = q.z.atan2(q.x);
    let dx = (longitude - 1.9 + PI).rem_euclid(TAU) - PI;
    let dy = (q.y - 0.23) * 2.5;
    let r2 = dx * dx + dy * dy;
    let twist = (-r2 * 3.).exp() * (3.3 + 0.25 * (time * 0.4).sin());
    let (s, c) = twist.sin_cos();
    let latitude = (0.23 + (dx * s + dy * c) / 2.5).clamp(-1., 1.);
    let angle = longitude + time * 0.035 * (1. + q.y * 2.).sin();
    let radial = (1. - latitude * latitude).max(0.).sqrt();
    let p = v(angle.cos() * radial, latitude, angle.sin() * radial);
    let clouds = n.sample(p * 9. + v(time * 0.035, 0., time * 0.02));
    let bands = 0.5 + 0.5 * (latitude * 47. + clouds * 3.).sin();
    let fine = 0.5 + 0.5 * (latitude * 155. + clouds * 5.).sin();
    let amber = mix(rgb(0x825166), rgb(0xe4a36c), bands);
    let cream = smooth(0.63, 0.94, bands);
    let atmosphere = mix(amber, rgb(0xf4dbad), cream * 0.85) + rgb(0xc69b90) * fine * 0.06;
    let eye = (-r2 * 38.).exp();
    let storm_rim = (-((r2.sqrt() - 0.28) * 13.).powi(2)).exp();
    mix(atmosphere, rgb(0x6d302d), eye * 0.85) * (0.36 + 0.64 * day)
        + rgb(0xf9d6a0) * storm_rim * 0.17
}
pub fn desert(q: V3, n: &Noise, time: f32, day: f32) -> V3 {
    let wind = n.sample(q * 3.4);
    let exposed = n.fbm(q * 3., 3);
    let dune = 0.5 + 0.5 * (q.dot(v(0.8, 0.3, -0.5)) * 105. + wind * 12.).sin();
    let ripples = 0.5 + 0.5 * (q.dot(v(0.8, 0.3, -0.5)) * 320. + wind * 26.).sin();
    let sand = mix(
        rgb(0xb76c37),
        rgb(0xf0c17c),
        dune * 0.3 + ripples * 0.1 + exposed * 0.45,
    );
    let rock = mix(rgb(0x6f392b), rgb(0xb36e4c), n.sample(q * 31.) * 0.5 + 0.5);
    let dune_seas = smooth(0.40, 0.56, exposed);
    let salt = smooth(0.32, 0.70, salt_basin(q));
    let deposits = mix(
        rgb(0xc3c1a7),
        rgb(0xeee3c7),
        0.55 + 0.15 * (salt_basin(q) * 48. + wind * 2.).sin(),
    );
    let shimmer = (q.dot(v(31., 8., 17.)) + time * 0.2).sin().abs().powi(40) * salt * 0.025;
    mix(mix(rock, sand, dune_seas), deposits, salt) * (0.34 + 0.66 * day) + rgb(0xffdb90) * shimmer
}
