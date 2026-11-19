use nadir::{
    config::Config,
    flight::{Flight, Input},
    scene::Scene,
};
#[test]
fn ten_minutes_of_simulated_flight_stays_finite() {
    let scene = Scene::new(Config::load(Config::default_path()).unwrap()).unwrap();
    let s = &scene;
    let mut f = Flight::new(s);
    for frame in 0..36000 {
        if frame % 7200 == 0 {
            f.jump();
        }
        if frame % 1800 == 0 {
            f.local = !f.local;
        }
        let t = frame as f32 / 60.;
        f.advance(
            s,
            1. / 60.,
            Input {
                thrust: (t * 0.3).sin(),
                yaw: (t * 0.11).sin() * 0.25,
                pitch: (t * 0.2).cos() * 0.1,
                strafe: 0.15,
                boost: frame % 1200 < 300,
                brake: frame % 1200 > 1000,
                ..Input::default()
            },
        );
        assert!(
            [
                f.ship.position.x,
                f.ship.position.y,
                f.ship.position.z,
                f.camera.eye.x,
                f.camera.eye.y,
                f.camera.eye.z
            ]
            .iter()
            .all(|x| x.is_finite())
        );
    }
}
