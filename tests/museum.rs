use nadir::{
    app::App,
    config::Config,
    math::*,
    museum::{EXHIBIT_COUNT, Exhibit, Museum},
    scene::Scene,
};
fn app() -> App {
    let mut c = Config::load(Config::default_path()).unwrap();
    c.width = 320;
    c.height = 180;
    App::new(Scene::new(c).unwrap())
}
#[test]
fn all_eight_models_render_in_isolation_at_highest_detail() {
    let mut app = app();
    let mut museum = Museum::default();
    for index in 0..EXHIBIT_COUNT {
        museum.select(index);
        let m = app.render_exhibit(&museum);
        assert!(m.fragments > 100, "figura {index}");
        match museum.exhibit() {
            Exhibit::Sun => assert!(
                app.renderer
                    .fragments
                    .iter()
                    .flatten()
                    .all(|f| f.material == 0)
            ),
            Exhibit::Planet(i) => {
                assert!(
                    app.renderer
                        .fragments
                        .iter()
                        .flatten()
                        .all(|f| f.object == i)
                );
                let expected = if i == 5 {
                    app.scene.flat[2].triangles.len()
                } else {
                    12288
                };
                assert!(m.triangles_available >= expected);
            }
            Exhibit::Ship => {
                assert_eq!(m.triangles_available, 116480);
                assert!(
                    app.renderer
                        .fragments
                        .iter()
                        .flatten()
                        .all(|f| f.material == 7)
                );
            }
        }
    }
}
#[test]
fn museum_does_not_modify_saved_flight_or_lod() {
    let mut app = app();
    app.flight.local = true;
    app.flight.paused = true;
    let before = (
        app.flight.time,
        app.flight.ship.position,
        app.flight.ship.velocity,
        app.flight.ship.forward,
        app.flight.camera.eye,
        app.flight.region,
        app.flight.accumulator,
        app.flight.paused,
    );
    let levels = app.scene.levels.clone();
    let mut museum = Museum::default();
    for i in 0..8 {
        museum.select(i);
        museum.advance(0.1);
        app.render_exhibit(&museum);
    }
    let after = (
        app.flight.time,
        app.flight.ship.position,
        app.flight.ship.velocity,
        app.flight.ship.forward,
        app.flight.camera.eye,
        app.flight.region,
        app.flight.accumulator,
        app.flight.paused,
    );
    assert_eq!(before, after);
    assert_eq!(levels, app.scene.levels);
}
#[test]
fn paused_animation_still_allows_camera_inspection() {
    let mut app = app();
    let mut museum = Museum::default();
    museum.select(2);
    app.render_exhibit(&museum);
    let first = app.fb.pixels.clone();
    for _ in 0..20 {
        museum.advance(0.1);
    }
    app.render_exhibit(&museum);
    assert_ne!(first, app.fb.pixels);
    museum.paused = true;
    let time = museum.time;
    let paused = app.fb.pixels.clone();
    museum.advance(0.1);
    app.render_exhibit(&museum);
    assert_eq!(time, museum.time);
    assert_eq!(paused, app.fb.pixels);
    museum.orbit(0.5, 0.2);
    museum.zoom_by(2.);
    app.render_exhibit(&museum);
    assert_ne!(paused, app.fb.pixels);
}
#[test]
fn camera_controls_stay_finite_and_reset_without_rewinding() {
    let mut museum = Museum::default();
    museum.select(7);
    museum.advance(0.1);
    let time = museum.time;
    museum.zoom_by(1000.);
    assert_eq!(museum.zoom, 8.);
    museum.zoom_by(-1000.);
    assert_eq!(museum.zoom, 0.45);
    museum.orbit(100000., 100000.);
    museum.pan_by(10000., -10000.);
    let app = app();
    let c = museum.camera(&app.scene);
    assert!(
        [c.eye.x, c.eye.y, c.eye.z, c.fov]
            .iter()
            .all(|f| f.is_finite())
    );
    assert!((c.target - c.eye).length() > 1.);
    museum.reset_view();
    assert_eq!(museum.time, time);
    assert_eq!(museum.zoom, 1.);
    assert_eq!(museum.pan.x, 0.);
    museum.select(99);
    assert_eq!(museum.selected(), 7);
    museum.next(false);
    assert_eq!(museum.selected(), 0);
    museum.next(true);
    assert_eq!(museum.selected(), 7);
}
#[test]
fn ship_comparison_preserves_view_and_uses_real_meshes() {
    let mut app = app();
    let mut museum = Museum::default();
    museum.select(7);
    let camera = museum.camera(&app.scene);
    let original = app.render_exhibit(&museum);
    museum.original_ship = false;
    let optimized = app.render_exhibit(&museum);
    assert_eq!(original.triangles_available, 116480);
    assert_eq!(optimized.triangles_available, 3261);
    assert_eq!(camera.eye, museum.camera(&app.scene).eye);
    assert_eq!(Museum::label_at(&app.scene, 1), "Tahiti");
    assert!(unit(camera.target - camera.eye).length() > 0.99);
}
