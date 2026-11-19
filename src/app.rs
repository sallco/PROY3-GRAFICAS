use crate::{
    config::Kind,
    flight::{Flight, Region},
    framebuffer::Framebuffer,
    math::*,
    renderer::{Metrics, Renderer},
    scene::Scene,
    shaders::Materials,
    sky::Skybox,
};
use std::time::Instant;
pub struct App {
    pub scene: Scene,
    pub flight: Flight,
    pub renderer: Renderer,
    pub fb: Framebuffer,
    pub sky: Skybox,
    pub materials: Materials,
    pub glow: crate::glow::SolarGlow,
}
impl App {
    pub fn new(scene: Scene) -> Self {
        let flight = Flight::new(&scene);
        let w = scene.config.width;
        let h = scene.config.height;
        let sky = Skybox::new(scene.config.seed);
        let materials = Materials::new(&scene.noise);
        Self {
            scene,
            flight,
            renderer: Renderer::new(w, h),
            fb: Framebuffer::new(w, h),
            sky,
            materials,
            glow: crate::glow::SolarGlow::new(w, h),
        }
    }
    pub fn render(&mut self) -> Metrics {
        self.render_view(
            self.flight.camera,
            Some(self.flight.ship.transform()),
            self.flight.region == Region::Nucleus,
            self.flight.time,
            self.flight.warp,
        )
    }
    pub fn render_view(
        &mut self,
        camera: Camera,
        ship: Option<Transform>,
        nucleus: bool,
        time: f64,
        warp: Option<f32>,
    ) -> Metrics {
        let start = Instant::now();
        self.scene.update_lod(camera, self.fb.height, time);
        self.sky.render(
            &mut self.fb,
            camera,
            nucleus,
            time as f32,
            warp,
            self.scene.config.star_radius,
        );
        let calls = self.scene.calls(time, ship, nucleus);
        self.renderer.rasterize(&mut self.fb, camera, &calls);
        self.renderer.shade(&mut self.fb, |f| {
            self.materials
                .shade(f, &self.scene, camera.eye, time as f32)
        });
        if !nucleus {
            self.glow.apply(&mut self.fb, &self.renderer.fragments);
        }
        self.renderer.metrics.render_ms = start.elapsed().as_secs_f64() * 1000.;
        self.renderer.metrics
    }
    pub fn showcase(&self, i: usize, time: f64) -> Camera {
        let center = self.scene.center(i, time);
        let w = &self.scene.config.worlds[i];
        let offset = if w.kind == Kind::Flat {
            v(1.25, 1.15, 1.6)
        } else {
            let toward_sun = unit(-center);
            unit(toward_sun + v(0., 0.35, 0.45)) * 3.2
        };
        Camera::new(center + offset * w.radius, center)
    }
}
