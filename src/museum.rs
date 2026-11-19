use crate::{math::*, scene::Scene};
use raylib::math::Vector2;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exhibit {
    Sun,
    Planet(usize),
    Ship,
}
pub const EXHIBIT_COUNT: usize = 8;
/// Cámara y reloj propios: observar una figura no modifica el vuelo guardado.
pub struct Museum {
    selected: usize,
    pub time: f64,
    pub paused: bool,
    pub auto_rotate: bool,
    pub original_ship: bool,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub pan: Vector2,
}
impl Default for Museum {
    fn default() -> Self {
        Self {
            selected: 0,
            time: 12.,
            paused: false,
            auto_rotate: false,
            original_ship: true,
            yaw: 0.,
            pitch: 0.32,
            zoom: 1.,
            pan: Vector2::zero(),
        }
    }
}
impl Museum {
    pub fn selected(&self) -> usize {
        self.selected
    }
    pub fn exhibit(&self) -> Exhibit {
        match self.selected {
            0 => Exhibit::Sun,
            1..=6 => Exhibit::Planet(self.selected - 1),
            _ => Exhibit::Ship,
        }
    }
    pub fn select(&mut self, index: usize) {
        if index < EXHIBIT_COUNT && index != self.selected {
            self.selected = index;
            self.reset_view();
        }
    }
    pub fn next(&mut self, backwards: bool) {
        self.select(
            (self.selected + if backwards { EXHIBIT_COUNT - 1 } else { 1 }) % EXHIBIT_COUNT,
        );
    }
    pub fn reset_view(&mut self) {
        self.yaw = if self.selected == 7 { -0.65 } else { 0. };
        self.pitch = if self.selected == 6 { 0.5 } else { 0.32 };
        self.zoom = 1.;
        self.pan = Vector2::zero();
    }
    pub fn orbit(&mut self, yaw: f32, pitch: f32) {
        self.yaw = (self.yaw + yaw).rem_euclid(std::f32::consts::TAU);
        self.pitch = (self.pitch + pitch).clamp(-1.5, 1.5);
    }
    pub fn zoom_by(&mut self, ticks: f32) {
        self.zoom = (self.zoom * (ticks * 0.16).exp()).clamp(0.45, 8.);
    }
    pub fn pan_by(&mut self, x: f32, y: f32) {
        self.pan.x = (self.pan.x + x / self.zoom).clamp(-3., 3.);
        self.pan.y = (self.pan.y + y / self.zoom).clamp(-3., 3.);
    }
    pub fn advance(&mut self, dt: f32) {
        if !self.paused {
            let dt = dt.clamp(0., 0.1);
            self.time += dt as f64;
            if self.auto_rotate {
                self.orbit(dt * 0.25, 0.);
            }
        }
    }
    pub fn label<'a>(&self, scene: &'a Scene) -> &'a str {
        Self::label_at(scene, self.selected)
    }
    pub fn label_at(scene: &Scene, index: usize) -> &str {
        match index {
            0 => "Sol",
            1..=6 => &scene.config.worlds[index - 1].name,
            _ => "Nave",
        }
    }
    pub fn description(&self, scene: &Scene) -> &str {
        use crate::config::Kind;
        match self.exhibit() {
            Exhibit::Sun => "Granulacion, emision y resplandor solar.",
            Exhibit::Ship => "Modelo original y malla de vuelo.",
            Exhibit::Planet(i) => match scene.config.worlds[i].kind {
                Kind::Forest => "Bosque anular y lagunas violetas.",
                Kind::Volcanic => "Relieve fracturado y lava en movimiento.",
                Kind::Ice => "Glaciares blancos y cristales facetados.",
                Kind::Gas => "Bandas atmosfericas, tormenta y anillos.",
                Kind::Desert => "Dunas, cuenca salina y observatorios.",
                Kind::Flat => "Continentes, nubes y base conica.",
            },
        }
    }
    pub fn center_radius(&self, scene: &Scene) -> (V3, f32) {
        match self.exhibit() {
            Exhibit::Sun => (V3::zero(), scene.config.star_radius),
            Exhibit::Planet(i) => (
                scene.center(i, self.time),
                scene.config.worlds[i].radius
                    * crate::terrain::surface_bound(scene.config.worlds[i].kind),
            ),
            Exhibit::Ship => (v(-20., 0., 0.), 2.),
        }
    }
    pub fn camera(&self, scene: &Scene) -> Camera {
        let (center, radius) = self.center_radius(scene);
        let base = if self.exhibit() == Exhibit::Sun {
            v(0., 0., 1.)
        } else {
            unit(v(-center.x, 0., -center.z))
        };
        let horizontal = rotate_y(base, self.yaw) * self.pitch.cos();
        let direction = unit(horizontal + v(0., self.pitch.sin(), 0.));
        let fit_distance = if self.exhibit() == Exhibit::Ship {
            2.2
        } else {
            3.4
        };
        let mut camera = Camera::new(center + direction * (radius * fit_distance), center);
        let (right, up, _) = camera.basis();
        let offset = (right * self.pan.x + up * self.pan.y) * radius;
        camera.eye += offset;
        camera.target += offset;
        camera.fov = 2. * ((55_f32.to_radians() * 0.5).tan() / self.zoom).atan();
        camera
    }
}
