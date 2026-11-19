use nadir::{
    app::App,
    config::{Config, Kind},
    flight::{Flight, Input, Region, SHIP_RADIUS, resolve_scene},
    framebuffer::Framebuffer,
    geometry::{Mesh, Vertex},
    math::*,
    renderer::{DrawCall, Renderer},
    scene::{Scene, generate_instances},
    shaders::procedural::Noise,
};
use std::sync::OnceLock;
fn scene() -> &'static Scene {
    static SCENE: OnceLock<Scene> = OnceLock::new();
    SCENE.get_or_init(|| Scene::new(Config::load(Config::default_path()).unwrap()).unwrap())
}
#[test]
fn configuration_rejects_invalid_values_and_intersecting_orbits() {
    let mut c = scene().config.clone();
    c.worlds[0].radius = f32::NAN;
    assert!(c.validate().is_err());
    let mut c = scene().config.clone();
    c.worlds[1].orbit = c.worlds[0].orbit;
    assert!(c.validate().is_err());
    let mut c = scene().config.clone();
    c.width = 0;
    assert!(c.validate().is_err());
}
#[test]
fn normalized_ship_and_all_resources_load() {
    let s = scene();
    assert_eq!(s.ship.triangles.len(), 3261);
    assert!(
        s.ship
            .vertices
            .iter()
            .all(|p| p.position.length() <= 1.001 && p.normal.dot(p.normal) > 0.9)
    );
    assert_eq!(s.config.worlds.len(), 6);
    assert!(s.config.worlds.iter().any(|w| w.kind == Kind::Flat));
}
#[test]
fn subdivision_adds_geometry_and_preserves_unit_radius() {
    let s = scene();
    for pair in s.spheres.windows(2) {
        assert_eq!(pair[1].triangles.len(), pair[0].triangles.len() * 4);
        assert!(
            pair[1]
                .vertices
                .iter()
                .all(|p| (p.position.length() - 1.).abs() < 1e-5)
        );
    }
    assert!(
        s.volcanic[2]
            .vertices
            .iter()
            .any(|p| (p.position.length() - 1.).abs() > 0.01)
    );
}
#[test]
fn distribution_is_reproducible_and_seed_sensitive() {
    let a = generate_instances(&Noise::new(42), Kind::Forest, 48);
    let b = generate_instances(&Noise::new(42), Kind::Forest, 48);
    let c = generate_instances(&Noise::new(43), Kind::Forest, 48);
    assert_eq!(a.len(), 48);
    for (x, y) in a.iter().zip(&b) {
        assert_eq!(x.transform.position, y.transform.position);
        assert_eq!(x.transform.scale, y.transform.scale);
        assert!(x.transform.y.dot(unit(x.transform.position)) > 0.99);
    }
    assert!(
        a.iter()
            .zip(c)
            .any(|(x, y)| x.transform.position != y.transform.position)
    );
}
#[test]
fn instances_follow_curvature_and_rotation() {
    let s = scene();
    let p = s.instances[0][0].transform;
    let a = s.transform(0, 0.);
    let b = s.transform(0, 15.);
    let position = b.point(p.position);
    assert!((b.inverse(position) - p.position).length() < 1e-5);
    assert!((a.vector(p.position) - b.vector(p.position)).length() > 0.01);
    assert!(unit(position - b.position).dot(b.vector(p.y)) > 0.99);
}
#[test]
fn simulation_is_consistent_at_different_presentation_rates() {
    let s = scene();
    let mut a = Flight::new(s);
    let mut b = Flight::new(s);
    let input = Input {
        thrust: 0.4,
        yaw: 0.1,
        ..Input::default()
    };
    for _ in 0..180 {
        a.advance(s, 1. / 30., input);
    }
    for _ in 0..864 {
        b.advance(s, 1. / 144., input);
    }
    assert!((a.ship.position - b.ship.position).length() < 0.1);
    assert!((a.ship.velocity - b.ship.velocity).length() < 0.05);
    assert!((a.time - b.time).abs() < 0.01);
}
#[test]
fn gravity_changes_unpowered_trajectory() {
    let s = scene();
    let mut f = Flight::new(s);
    let before = f.ship.velocity;
    for _ in 0..120 {
        f.advance(s, 1. / 120., Input::default());
    }
    assert!((f.ship.velocity - before).length() > 0.01);
}
#[test]
fn fast_ship_cannot_cross_a_moving_planet() {
    let s = scene();
    let i = 0;
    let center = s.center(i, 12.);
    let previous = s.center(i, 11.9);
    let (p, n) = resolve_scene(
        s,
        Region::System,
        11.9,
        12.,
        previous + v(-20., 0., 0.),
        center + v(20., 0., 0.),
        SHIP_RADIUS,
    );
    assert!(n.is_some());
    assert!((p - center).length() >= s.config.worlds[i].radius + SHIP_RADIUS);
    assert!(p.x < center.x);
}
#[test]
fn flat_world_blocks_top_bottom_and_rim() {
    let s = scene();
    let i = 5;
    let c = s.center(i, 0.);
    let radius = s.config.worlds[i].radius;
    for (from, to) in [
        (v(0., 20., 0.), v(0., -20., 0.)),
        (v(0., -20., 0.), v(0., 20., 0.)),
        (v(20., 0., 0.), v(-20., 0., 0.)),
    ] {
        let (p, n) = resolve_scene(s, Region::System, 0., 0., c + from, c + to, SHIP_RADIUS);
        assert!(n.is_some());
        let p = p - c;
        assert!(
            p.y >= radius * 0.14 + SHIP_RADIUS - 0.001
                || p.y <= -radius * 0.38 - SHIP_RADIUS + 0.001
                || (p.x * p.x + p.z * p.z).sqrt() >= radius + SHIP_RADIUS - 0.001
        );
    }
}
#[test]
fn warp_round_trip_restores_safe_system_state() {
    let s = scene();
    let mut f = Flight::new(s);
    let before = f.ship.position;
    f.jump();
    f.jump();
    for _ in 0..40 {
        f.advance(s, 0.1, Input::default());
    }
    assert_eq!(f.region, Region::Nucleus);
    assert!(f.warp.is_none());
    assert!(f.ship.position.length() > 7.5);
    f.jump();
    for _ in 0..31 {
        f.advance(s, 0.1, Input::default());
    }
    assert_eq!(f.region, Region::System);
    assert!((f.ship.position - before).length() < 1.);
    assert!(f.camera.eye.x.is_finite());
}
fn test_mesh() -> Mesh {
    Mesh {
        vertices: vec![
            Vertex {
                position: v(-1., -1., 0.),
                normal: v(0., 0., 1.),
            },
            Vertex {
                position: v(1., -1., 0.),
                normal: v(0., 0., 1.),
            },
            Vertex {
                position: v(0., 1., 0.),
                normal: v(0., 0., 1.),
            },
        ],
        triangles: vec![[0, 1, 2]],
    }
}
#[test]
fn shared_depth_keeps_nearer_object_independent_of_draw_order() {
    let mesh = test_mesh();
    let camera = Camera::new(v(0., 0., 5.), V3::zero());
    for reverse in [false, true] {
        let mut calls = vec![
            DrawCall {
                mesh: &mesh,
                transform: Transform::new(V3::zero(), 1., 0.),
                material: 1,
                object: 0,
                radius: 2.,
                double_sided: false,
            },
            DrawCall {
                mesh: &mesh,
                transform: Transform::new(v(0., 0., 1.), 1., 0.),
                material: 2,
                object: 1,
                radius: 2.,
                double_sided: false,
            },
        ];
        if reverse {
            calls.reverse();
        }
        let mut fb = Framebuffer::new(64, 64);
        let mut renderer = Renderer::new(64, 64);
        renderer.rasterize(&mut fb, camera, &calls);
        assert_eq!(renderer.fragments[32 * 64 + 32].unwrap().material, 2);
    }
}
#[test]
fn six_worlds_render_and_culling_preserves_the_image() {
    let mut config = scene().config.clone();
    config.width = 320;
    config.height = 180;
    let mut app = App::new(Scene::new(config).unwrap());
    for i in 0..6 {
        let camera = app.showcase(i, 12.);
        app.renderer.culling = false;
        let full = app.render_view(camera, None, false, 12., None);
        let pixels = app.fb.pixels.clone();
        let depths = app.fb.depths.clone();
        app.renderer.culling = true;
        let culled = app.render_view(camera, None, false, 12., None);
        assert_eq!(pixels, app.fb.pixels, "mundo {i}");
        assert_eq!(depths, app.fb.depths);
        assert!(culled.triangles < full.triangles);
        assert!(culled.fragments > 300);
        assert!(
            app.renderer
                .fragments
                .iter()
                .flatten()
                .all(|f| f.world.x.is_finite() && f.normal.y.is_finite())
        );
    }
}
#[test]
fn lod_reduces_geometry_at_distance() {
    let mut config = scene().config.clone();
    config.width = 320;
    config.height = 180;
    let mut s = Scene::new(config).unwrap();
    let c = Camera::new(v(0., 120., 250.), V3::zero());
    s.lod = false;
    s.update_lod(c, 180, 0.);
    let high: usize = s
        .calls(0., None, false)
        .iter()
        .map(|d| d.mesh.triangles.len())
        .sum();
    s.lod = true;
    s.update_lod(c, 180, 0.);
    let low: usize = s
        .calls(0., None, false)
        .iter()
        .map(|d| d.mesh.triangles.len())
        .sum();
    assert!(low < high / 2);
}
