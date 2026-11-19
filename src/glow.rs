use crate::{
    framebuffer::{Framebuffer, rgba},
    math::*,
    renderer::Fragment,
};
/// Resplandor solar a baja resolución, con máscara de visibilidad resuelta en CPU.
pub struct SolarGlow {
    width: usize,
    height: usize,
    light: Vec<f32>,
    scratch: Vec<f32>,
}
impl SolarGlow {
    pub fn new(width: usize, height: usize) -> Self {
        let width = width.div_ceil(4);
        let height = height.div_ceil(4);
        Self {
            width,
            height,
            light: vec![0.; width * height],
            scratch: vec![0.; width * height],
        }
    }
    pub fn apply(&mut self, fb: &mut Framebuffer, fragments: &[Option<Fragment>]) {
        self.light.fill(0.);
        let mut visible = false;
        for (i, f) in fragments.iter().enumerate() {
            if f.is_some_and(|f| f.material == 0) {
                visible = true;
                let index = (i / fb.width / 4) * self.width + (i % fb.width / 4);
                self.light[index] += 1. / 16.;
            }
        }
        if !visible {
            return;
        }
        for _ in 0..3 {
            blur(
                &self.light,
                &mut self.scratch,
                self.width,
                self.height,
                true,
            );
            blur(
                &self.scratch,
                &mut self.light,
                self.width,
                self.height,
                false,
            );
        }
        for (i, pixel) in fb.pixels.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            if fragments[i].is_some_and(|f| f.material != 0) {
                continue;
            }
            let x = ((i % fb.width) as f32 * 0.25).min((self.width - 1) as f32);
            let y = ((i / fb.width) as f32 * 0.25).min((self.height - 1) as f32);
            let ix = x as usize;
            let iy = y as usize;
            let nx = (ix + 1).min(self.width - 1);
            let ny = (iy + 1).min(self.height - 1);
            let tx = x - x.floor();
            let ty = y - y.floor();
            let strength = (self.light[iy * self.width + ix] * (1. - tx)
                + self.light[iy * self.width + nx] * tx)
                * (1. - ty)
                + (self.light[ny * self.width + ix] * (1. - tx)
                    + self.light[ny * self.width + nx] * tx)
                    * ty;
            let color = v(pixel[0] as f32, pixel[1] as f32, pixel[2] as f32) / 255.;
            let gain = if fragments[i].is_some() { 0.1 } else { 2.2 };
            *pixel = rgba(color + v(1., 0.78, 0.38) * strength * gain);
        }
    }
}
fn blur(source: &[f32], target: &mut [f32], width: usize, height: usize, horizontal: bool) {
    let radius = 5;
    for y in 0..height {
        for x in 0..width {
            let (coord, limit) = if horizontal { (x, width) } else { (y, height) };
            let start = coord.saturating_sub(radius);
            let end = (coord + radius).min(limit - 1);
            let mut sum = 0.;
            for tap in start..=end {
                sum += source[if horizontal {
                    y * width + tap
                } else {
                    tap * width + x
                }];
            }
            target[y * width + x] = sum / (end - start + 1) as f32;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn glow_respects_depth_and_foreground() {
        let mut fb = Framebuffer::new(64, 64);
        fb.pixels.fill(0);
        let mut fragments = vec![None; 4096];
        for y in 25..39 {
            for x in 25..39 {
                fragments[y * 64 + x] = Some(Fragment {
                    local: V3::zero(),
                    world: V3::zero(),
                    normal: v(0., 1., 0.),
                    material: 0,
                    object: usize::MAX,
                });
            }
        }
        let blocked = 32 * 64 + 40;
        fragments[blocked] = Some(Fragment {
            material: 1,
            ..fragments[32 * 64 + 32].unwrap()
        });
        let depths = fb.depths.clone();
        SolarGlow::new(64, 64).apply(&mut fb, &fragments);
        assert!(fb.pixels[(32 * 64 + 41) * 4] > 0);
        assert_eq!(fb.pixels[blocked * 4], 0);
        assert_eq!(fb.depths, depths);
    }
}
