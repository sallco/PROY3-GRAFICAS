use serde::Deserialize;
use std::path::{Path, PathBuf};
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub seed: u32,
    pub width: usize,
    pub height: usize,
    pub star_radius: f32,
    pub shader_intensity: f32,
    pub worlds: Vec<WorldConfig>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldConfig {
    pub name: String,
    pub kind: Kind,
    pub radius: f32,
    pub orbit: f32,
    pub orbit_speed: f32,
    pub spin: f32,
    pub phase: f32,
    pub inclination: f32,
    pub instances: usize,
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Forest,
    Volcanic,
    Ice,
    Gas,
    Desert,
    Flat,
}
impl Kind {
    pub fn material(self) -> usize {
        match self {
            Self::Forest => 1,
            Self::Volcanic => 2,
            Self::Ice => 3,
            Self::Gas => 4,
            Self::Desert => 5,
            Self::Flat => 6,
        }
    }
}
impl Config {
    pub fn default_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("config/scene.toml")
    }
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let c: Self = toml::from_str(&std::fs::read_to_string(path)?)?;
        c.validate()?;
        Ok(c)
    }
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !(320..=1920).contains(&self.width) || !(180..=1080).contains(&self.height) {
            return Err("Resolución interna fuera de límites: 320..1920 × 180..1080".into());
        }
        if !self.star_radius.is_finite()
            || !(1.0..=12.).contains(&self.star_radius)
            || !self.shader_intensity.is_finite()
            || !(0.1..=3.).contains(&self.shader_intensity)
        {
            return Err("Radio estelar o intensidad inválidos".into());
        }
        if self.worlds.len() != 6 {
            return Err("La escena requiere seis mundos".into());
        }
        for (i, w) in self.worlds.iter().enumerate() {
            if w.name.trim().is_empty()
                || ![
                    w.radius,
                    w.orbit,
                    w.orbit_speed,
                    w.spin,
                    w.phase,
                    w.inclination,
                ]
                .iter()
                .all(|x| x.is_finite())
                || !(1.0..=10.).contains(&w.radius)
                || w.orbit <= self.star_radius + w.radius * 2.
                || w.orbit > 180.
                || w.orbit_speed.abs() > 0.5
                || w.spin.abs() > 2.
                || w.inclination.abs() > 0.5
                || w.instances > 200
            {
                return Err(format!("Parámetros inválidos del mundo {}", i + 1).into());
            }
            for p in &self.worlds[..i] {
                if p.name == w.name
                    || p.kind == w.kind
                    || (p.orbit - w.orbit).abs() < p.radius * 1.9 + w.radius * 1.9 + 1.
                {
                    return Err("Nombres/tipos repetidos u órbitas demasiado próximas".into());
                }
            }
        }
        Ok(())
    }
}
