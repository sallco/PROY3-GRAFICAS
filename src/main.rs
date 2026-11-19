use nadir::{
    app::App,
    config::Config,
    geometry::surface_models,
    math::{Camera, Transform, V3, v},
    scene::Scene,
};
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
                    "Nadir\n  --config RUTA\n  --export-captures | --benchmark | --export-models | --record-demo\n  --output RUTA (directorio; MP4 para --record-demo)\nSin opciones: menu principal, exploracion y museo de modelos."
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
        _ => nadir::ui::run(app, &config),
    }
}
fn captures(app: &mut App, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(output)?;
    for i in 0..6 {
        let camera = app.showcase(i, 12.);
        let metrics = app.render_view(camera, None, false, 12., None);
        let name = [
            "tahiti",
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
    app.render_view(
        Camera::new(v(0., 4., 18.), V3::zero()),
        None,
        false,
        12.,
        None,
    );
    app.fb.export_png(&output.join("sol.png"))?;
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
