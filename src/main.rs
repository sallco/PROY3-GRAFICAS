use nadir::{
    app::App,
    config::Config,
    flight::{Input, Region},
    geometry::surface_models,
    math::{Camera, Transform, V3, v},
    scene::Scene,
};
use raylib::prelude::*;
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Nadir: {e}");
            ExitCode::FAILURE
        }
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut config = Config::default_path();
    let mut mode = String::new();
    let mut output = PathBuf::from("output");
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => config = args.next().ok_or("Falta ruta de configuración")?.into(),
            "--export-captures" | "--benchmark" | "--export-models" | "--record-demo" => {
                if !mode.is_empty() {
                    return Err("Seleccione un solo modo de ejecución".into());
                }
                mode = arg;
            }
            "--output" => output = args.next().ok_or("Falta ruta de salida")?.into(),
            "--help" => {
                println!(
                    "Nadir\n  --config RUTA\n  --export-captures | --benchmark | --export-models | --record-demo\n  --output RUTA (directorio; MP4 para --record-demo)\nSin opciones: vuelo interactivo."
                );
                return Ok(());
            }
            _ => return Err(format!("Opción desconocida: {arg}").into()),
        }
    }
    let mut app = App::new(Scene::new(Config::load(&config)?)?);
    match mode.as_str() {
        "--export-captures" => captures(&mut app, &output),
        "--benchmark" => benchmark(&mut app, &output),
        "--export-models" => {
            std::fs::create_dir_all(&output)?;
            for (mesh, name) in surface_models()
                .iter()
                .zip(["arbol", "basalto", "cristal", "monolito"])
            {
                mesh.export(output.join(format!("{name}.obj")))?;
            }
            Ok(())
        }
        "--record-demo" => nadir::demo::record(&mut app, &output),
        _ => interactive(app, &config),
    }
}
fn captures(app: &mut App, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output)?;
    for i in 0..6 {
        let camera = app.showcase(i, 12.);
        let metrics = app.render_view(camera, None, false, 12., None);
        let name = [
            "mneme",
            "obsidia",
            "prisma",
            "artaud",
            "palimpsesto",
            "tierra-plana",
        ][i];
        app.fb.export_png(&output.join(format!("{name}.png")))?;
        println!(
            "{name}: {:.2} ms; {} / {} triángulos",
            metrics.render_ms, metrics.triangles, metrics.triangles_available
        );
    }
    let center = app.scene.center(5, 12.);
    app.render_view(
        Camera::new(center + v(10., -8., 12.), center),
        None,
        false,
        12.,
        None,
    );
    app.fb
        .export_png(&output.join("tierra-plana-inferior.png"))?;
    app.render();
    app.fb.export_png(&output.join("vuelo.png"))?;
    app.render_view(
        Camera::new(v(0., 8., 35.), V3::zero()),
        None,
        true,
        12.,
        None,
    );
    app.fb.export_png(&output.join("umbra.png"))?;
    let c = app.flight.camera;
    app.render_view(c, None, false, 12., Some(1.1));
    app.fb.export_png(&output.join("warping.png"))?;
    let mut original = nadir::geometry::Mesh::load(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/models/nave-original.obj"),
    )?;
    original.normalize();
    let center = v(0., 7., 35.);
    let camera = Camera::new(center + v(2., 1.7, 4.3), center);
    let transform = Transform::new(center, 1.4, 0.5);
    app.render_view(camera, Some(transform), true, 12., None);
    app.fb.export_png(&output.join("nave-vuelo.png"))?;
    let optimized = std::mem::replace(&mut app.scene.ship, original);
    app.render_view(camera, Some(transform), true, 12., None);
    app.fb.export_png(&output.join("nave-original.png"))?;
    app.scene.ship = optimized;
    Ok(())
}
fn benchmark(app: &mut App, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output)?;
    let mut csv = String::from(
        "lod,culling,frame,render_ms,objects,culled,triangles_available,triangles_processed,fragments\n",
    );
    for lod in [false, true] {
        for culling in [false, true] {
            app.scene.lod = lod;
            app.renderer.culling = culling;
            let mut times = Vec::new();
            let mut triangles = 0;
            for frame in 0..72 {
                let i = frame % 6;
                let time = 12. + frame as f64 * 0.125;
                let camera = if frame % 3 == 0 {
                    let center = app.scene.center(i, time);
                    Camera::new(center + v(0., 12., 45.), center)
                } else {
                    app.showcase(i, time)
                };
                let m = app.render_view(camera, None, false, time, None);
                if frame >= 12 {
                    times.push(m.render_ms);
                    triangles += m.triangles;
                }
                csv.push_str(&format!(
                    "{lod},{culling},{frame},{:.3},{},{},{},{},{}\n",
                    m.render_ms,
                    m.objects,
                    m.culled,
                    m.triangles_available,
                    m.triangles,
                    m.fragments
                ));
            }
            times.sort_by(f64::total_cmp);
            let mean = times.iter().sum::<f64>() / times.len() as f64;
            println!(
                "LOD={lod} culling={culling}: media={mean:.2} ms, p95={:.2} ms, {:.1} FPS, {} triángulos/cuadro",
                times[(times.len() as f32 * 0.95) as usize],
                1000. / mean,
                triangles / times.len()
            );
        }
    }
    std::fs::write(output.join("benchmark.csv"), csv)?;
    app.scene.lod = true;
    app.renderer.culling = true;
    Ok(())
}
fn interactive(mut app: App, config: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use KeyboardKey::*;
    let (mut window, thread) = raylib::init()
        .size(1280, 720)
        .title("NADIR | Observatorio de mundos imposibles")
        .msaa_4x()
        .build();
    window.set_target_fps(60);
    let image = Image::gen_image_color(app.fb.width as i32, app.fb.height as i32, Color::BLACK);
    let mut texture = window.load_texture_from_image(&thread, &image)?;
    let mut help = true;
    let mut stats = false;
    let mut captured = false;
    let mut notice = String::new();
    let mut notice_time = 0.;
    let mut fps = 0.;
    while !window.window_should_close() {
        let dt = window.get_frame_time().clamp(0.0001, 0.1);
        if window.is_key_pressed(KEY_H) {
            help = !help;
        }
        if window.is_key_pressed(KEY_F3) {
            stats = !stats;
        }
        if window.is_key_pressed(KEY_F4) {
            app.scene.lod = !app.scene.lod;
        }
        if window.is_key_pressed(KEY_F5) {
            app.renderer.culling = !app.renderer.culling;
        }
        if window.is_key_pressed(KEY_F6) {
            app.materials.normal_mapping = !app.materials.normal_mapping;
        }
        if window.is_key_pressed(KEY_P) {
            app.flight.paused = !app.flight.paused;
        }
        if window.is_key_pressed(KEY_F) {
            app.flight.local = !app.flight.local;
        }
        if window.is_key_pressed(KEY_TAB) {
            app.flight.target = (app.flight.target + 1) % 6;
        }
        if window.is_key_pressed(KEY_C) {
            captured = !captured;
            if captured {
                window.disable_cursor();
            } else {
                window.enable_cursor();
            }
        }
        let mouse = window.get_mouse_position();
        let button = Rectangle::new(1050., 28., 202., 42.);
        let hover = !captured && button.check_collision_point_rec(mouse);
        if window.is_key_pressed(KEY_G)
            || (hover && window.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT))
        {
            app.flight.jump();
        }
        if window.is_key_pressed(KEY_R) {
            match Config::load(config).and_then(Scene::new) {
                Ok(scene) => {
                    app = App::new(scene);
                    let image = Image::gen_image_color(
                        app.fb.width as i32,
                        app.fb.height as i32,
                        Color::BLACK,
                    );
                    texture = window.load_texture_from_image(&thread, &image)?;
                    notice = "Configuracion recargada; vuelo restablecido".into();
                }
                Err(e) => notice = format!("No se aplico la configuracion: {e}"),
            };
            notice_time = 6.;
        }
        let axis = |a, b| {
            i32::from(window.is_key_down(a)) as f32 - i32::from(window.is_key_down(b)) as f32
        };
        let delta = if captured {
            window.get_mouse_delta()
        } else {
            Vector2::zero()
        };
        let input = Input {
            thrust: axis(KEY_W, KEY_S),
            strafe: axis(KEY_D, KEY_A),
            lift: axis(KEY_SPACE, KEY_LEFT_CONTROL),
            yaw: axis(KEY_RIGHT, KEY_LEFT) + delta.x * 0.0018 / dt,
            pitch: axis(KEY_UP, KEY_DOWN) - delta.y * 0.0018 / dt,
            roll: axis(KEY_E, KEY_Q),
            boost: window.is_key_down(KEY_LEFT_SHIFT),
            brake: window.is_key_down(KEY_X),
        };
        app.flight.advance(&app.scene, dt, input);
        let metrics = app.render();
        texture.update_texture(&app.fb.pixels)?;
        fps += ((1. / dt) - fps) * (1. - (-dt * 2.).exp());
        notice_time -= dt;
        let mut draw = window.begin_drawing(&thread);
        draw.clear_background(Color::BLACK);
        draw.draw_texture_pro(
            &texture,
            Rectangle::new(0., 0., app.fb.width as f32, app.fb.height as f32),
            Rectangle::new(0., 0., 1280., 720.),
            Vector2::zero(),
            0.,
            Color::WHITE,
        );
        let ink = Color::new(220, 232, 239, 255);
        let muted = Color::new(145, 168, 186, 255);
        let accent = Color::new(110, 226, 206, 255);
        draw.draw_rectangle(0, 0, 1280, 96, Color::new(3, 9, 18, 185));
        draw.draw_text("N A D I R", 28, 24, 30, ink);
        draw.draw_text("OBSERVATORIO DE MUNDOS IMPOSIBLES", 30, 62, 12, muted);
        let nucleus = app.flight.region == Region::Nucleus;
        draw.draw_rectangle_rec(
            button,
            if hover {
                Color::new(43, 80, 95, 240)
            } else {
                Color::new(24, 44, 62, 230)
            },
        );
        draw.draw_rectangle_lines_ex(button, 1., Color::new(81, 119, 140, 255));
        draw.draw_text(
            if app.flight.warp.is_some() {
                "SALTO EN CURSO"
            } else if nucleus {
                "G  REGRESAR"
            } else {
                "G  NUCLEO GALACTICO"
            },
            1065,
            43,
            16,
            ink,
        );
        let status = if nucleus {
            "UMBRA / NUCLEO GALACTICO".to_string()
        } else {
            let i = app.flight.nearest(&app.scene);
            let alt = (app.flight.ship.position - app.scene.center(i, app.flight.time)).length()
                - app.scene.config.worlds[i].radius;
            format!(
                "{}  /  DISTANCIA RADIAL {alt:.1}",
                app.scene.config.worlds[i].name.to_uppercase()
            )
        };
        draw.draw_text(&status, 30, 626, 20, ink);
        draw.draw_text(
            &format!(
                "VELOCIDAD {:05.1}   |   {}   |   {}",
                app.flight.ship.velocity.length(),
                if app.flight.local {
                    "ASISTENCIA LOCAL"
                } else {
                    "VUELO LIBRE"
                },
                if app.flight.paused {
                    "PAUSA"
                } else {
                    "SIMULACION ACTIVA"
                }
            ),
            30,
            657,
            15,
            accent,
        );
        draw.draw_text(
            "H ayuda   C raton   TAB destino   F asistencia   X freno   P pausa   R restablecer",
            30,
            691,
            13,
            muted,
        );
        if !nucleus {
            let i = app.flight.target;
            let target = app.scene.center(i, app.flight.time);
            let clip = app.flight.camera.clip(target, 1280. / 720.);
            let mut x = 640. + clip[0] / clip[3].abs().max(0.1) * 640.;
            let mut y = 360. - clip[1] / clip[3].abs().max(0.1) * 360.;
            if clip[3] < 0. {
                x = 1280. - x;
                y = 720. - y;
            }
            x = x.clamp(30., 1130.);
            y = y.clamp(115., 580.);
            draw.draw_circle_lines(x as i32, y as i32, 8., accent);
            draw.draw_text(
                &format!(
                    "{}  {:.0}",
                    app.scene.config.worlds[i].name,
                    (target - app.flight.ship.position).length()
                ),
                x as i32 + 14,
                y as i32 - 7,
                14,
                ink,
            );
        }
        if help {
            draw.draw_rectangle(900, 110, 352, 380, Color::new(3, 10, 22, 225));
            draw.draw_text("NAVEGACION", 922, 131, 19, accent);
            for (i, text) in [
                "W / S      Empuje longitudinal",
                "A / D      Traslacion lateral",
                "ESP / CTRL Altura",
                "FLECHAS    Orientacion",
                "Q / E      Alabeo",
                "SHIFT      Impulso de crucero",
                "X          Frenado progresivo",
                "F          Asistencia local",
                "TAB        Siguiente destino",
                "G          Viajar / regresar",
                "C          Capturar raton",
                "H          Ocultar ayuda",
                "F3         Metricas tecnicas",
            ]
            .iter()
            .enumerate()
            {
                draw.draw_text(text, 922, 167 + i as i32 * 23, 15, ink);
            }
        }
        if stats {
            draw.draw_rectangle(28, 110, 510, 144, Color::new(3, 10, 22, 225));
            for (i, line) in [
                format!(
                    "CPU {:.1} ms  |  {:.0} FPS  |  {} x {}",
                    metrics.render_ms, fps, app.fb.width, app.fb.height
                ),
                format!(
                    "Objetos {}  descartados {}",
                    metrics.objects, metrics.culled
                ),
                format!(
                    "Triangulos {} / {}",
                    metrics.triangles, metrics.triangles_available
                ),
                format!("Fragmentos visibles {}", metrics.fragments),
                format!(
                    "F4 LOD {}  F5 descarte {}  F6 normales {}",
                    app.scene.lod, app.renderer.culling, app.materials.normal_mapping
                ),
            ]
            .iter()
            .enumerate()
            {
                draw.draw_text(line, 44, 126 + i as i32 * 24, 16, ink);
            }
        }
        if notice_time > 0. {
            draw.draw_rectangle(28, 552, 1224, 42, Color::new(12, 25, 38, 240));
            draw.draw_text(&notice, 42, 566, 15, ink);
        }
        draw.draw_line(633, 360, 647, 360, Color::new(180, 208, 220, 120));
        draw.draw_line(640, 353, 640, 367, Color::new(180, 208, 220, 120));
    }
    Ok(())
}
