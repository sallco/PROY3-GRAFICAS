//! Recorrido reproducible: vuelo mediante entradas del controlador y montaje de vistas.
use crate::{
    app::App,
    flight::{Flight, Input, Ship},
    math::*,
};
use std::f32::consts::TAU;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
const FPS: usize = 24;
const SECONDS: usize = 96;
fn route(app: &App, t: f32) -> V3 {
    let center = app.scene.center(0, t as f64);
    let (radius, angle) = if t < 12. {
        let s = (t / 12.).clamp(0., 1.);
        let ease = s * s * (3. - 2. * s);
        (16. - 9.7 * ease, 0.)
    } else if t < 38. {
        (6.3, (t - 12.) / 26. * TAU)
    } else {
        let s = ((t - 38.) / 8.).clamp(0., 1.);
        (6.3 + 13. * s * s, TAU)
    };
    center + v(angle.sin() * radius, 1.8, angle.cos() * radius)
}
fn pilot(app: &mut App, t: f32) {
    let target = route(app, t);
    let velocity = (route(app, t + 0.03) - target) / 0.03;
    let ship = app.flight.ship;
    let forward = unit(app.scene.center(0, app.flight.time) - ship.position);
    let right = unit(ship.forward.cross(ship.up));
    let acceleration = (target - ship.position) * 1.6 + (velocity - ship.velocity) * 2.2;
    let input = Input {
        thrust: (acceleration.dot(ship.forward) / 10.).clamp(-1., 1.),
        strafe: (acceleration.dot(right) / 10.).clamp(-1., 1.),
        lift: (acceleration.dot(ship.up) / 10.).clamp(-1., 1.),
        yaw: (right.dot(forward) * 2.).clamp(-1., 1.),
        pitch: (ship.up.dot(forward) * 2.).clamp(-1., 1.),
        ..Input::default()
    };
    app.flight.advance(&app.scene, 1. / FPS as f32, input);
}
pub fn record(app: &mut App, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let output = if output.extension().is_some() {
        output.to_path_buf()
    } else {
        output.join("nadir.mp4")
    };
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let chapters = [
        (0, 12, "01 / VUELO Y APROXIMACION"),
        (12, 38, "02 / VUELTA COMPLETA A MNEME"),
        (38, 46, "03 / ALEJAMIENTO CONTINUO"),
        (46, 52, "04 / OBSIDIA - CORTEZA Y LAVA"),
        (52, 58, "05 / PRISMA - HIELO Y CRISTALES"),
        (58, 64, "06 / ARTAUD - BANDAS Y VORTICE"),
        (64, 70, "07 / PALIMPSESTO - OBSERVATORIOS"),
        (70, 76, "08 / TIERRA PLANA - OCEANOS Y BORDE HELADO"),
        (76, 79, "09 / SALTO AL NUCLEO GALACTICO"),
        (79, 90, "10 / UMBRA - VUELO EN EL NUCLEO"),
        (90, 93, "11 / RETORNO AL SISTEMA"),
        (93, 96, "NADIR / RASTERIZACION INTEGRA EN CPU"),
    ];
    let mut filters = String::from(
        "scale=1280:720:flags=lanczos,drawbox=x=0:y=0:w=iw:h=76:color=0x030912@0.85:t=fill,drawtext=fontfile=/usr/share/fonts/TTF/DejaVuSans.ttf:text='N A D I R':x=30:y=20:fontsize=28:fontcolor=0xdce8ef",
    );
    for (start, end, label) in chapters {
        filters.push_str(&format!(",drawtext=fontfile=/usr/share/fonts/TTF/DejaVuSans.ttf:text='{label}':x=30:y=h-45:fontsize=18:fontcolor=0xc6e8e2:box=1:boxcolor=0x030912@0.75:boxborderw=12:enable='gte(t,{start})*lt(t,{end})'"));
    }
    let mut encoder = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "rawvideo",
            "-pixel_format",
            "rgba",
            "-video_size",
            &format!("{}x{}", app.fb.width, app.fb.height),
            "-framerate",
            &FPS.to_string(),
            "-i",
            "-",
            "-vf",
            &filters,
            "-an",
            "-c:v",
            "libx264",
            "-preset",
            "fast",
            "-crf",
            "22",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
        ])
        .arg(&output)
        .stdin(Stdio::piped())
        .spawn()?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut stream = encoder
            .stdin
            .take()
            .ok_or("No se pudo abrir la entrada de FFmpeg")?;
        app.flight = Flight::new(&app.scene);
        let start = route(app, 0.);
        app.flight.ship = Ship::new(start, app.scene.center(0, 0.) - start);
        app.flight.previous = app.flight.ship;
        app.flight.camera = Camera::new(
            start - app.flight.ship.forward * 2.8 + v(0., 1., 0.),
            app.scene.center(0, 0.),
        );
        for frame in 0..FPS * SECONDS {
            let t = frame as f32 / FPS as f32;
            if t < 46. {
                pilot(app, t);
                app.render();
            } else if t < 76. {
                let i = ((t - 46.) / 6.) as usize + 1;
                let time = t as f64;
                let center = app.scene.center(i, time);
                let base = app.showcase(i, time);
                let mut c = base;
                let a = (t - 46.).rem_euclid(6.) * 0.09 - 0.27;
                c.eye = center + rotate_y(base.eye - center, a);
                app.render_view(c, None, false, time, None);
                app.flight.time = time;
            } else {
                if frame == 76 * FPS {
                    app.flight.jump();
                }
                if frame == 90 * FPS {
                    app.flight.jump();
                }
                let input = if (80.0..89.).contains(&t) {
                    Input {
                        strafe: 0.35,
                        yaw: 0.045,
                        ..Input::default()
                    }
                } else {
                    Input::default()
                };
                app.flight.advance(&app.scene, 1. / FPS as f32, input);
                app.render();
            }
            stream.write_all(&app.fb.pixels)?;
            if frame % (FPS * 8) == 0 {
                println!("Video: {t:.0} / {SECONDS} s");
            }
        }
        drop(stream);
        Ok(())
    })();
    if result.is_err() {
        let _ = encoder.kill();
    }
    let status = encoder.wait()?;
    result?;
    if !status.success() {
        return Err("FFmpeg no pudo completar el video".into());
    }
    println!("Video guardado en {}", output.display());
    Ok(())
}
