use crate::{
    app::App,
    config::Config,
    flight::{Input, Region},
    museum::{EXHIBIT_COUNT, Exhibit, Museum},
    renderer::Metrics,
    scene::Scene,
};
use raylib::prelude::*;
use std::path::Path;
const INK: Color = Color {
    r: 220,
    g: 232,
    b: 239,
    a: 255,
};
const MUTED: Color = Color {
    r: 145,
    g: 168,
    b: 186,
    a: 255,
};
const ACCENT: Color = Color {
    r: 110,
    g: 226,
    b: 206,
    a: 255,
};
const VIEW: Rectangle = Rectangle {
    x: 332.,
    y: 126.,
    width: 920.,
    height: 517.5,
};
const EXPLORE: Rectangle = Rectangle {
    x: 52.,
    y: 320.,
    width: 260.,
    height: 52.,
};
const OPEN_MUSEUM: Rectangle = Rectangle {
    x: 52.,
    y: 388.,
    width: 260.,
    height: 52.,
};
const QUIT: Rectangle = Rectangle {
    x: 52.,
    y: 456.,
    width: 260.,
    height: 52.,
};
const FLIGHT_MUSEUM: Rectangle = Rectangle {
    x: 840.,
    y: 28.,
    width: 190.,
    height: 42.,
};
const BACK: Rectangle = Rectangle {
    x: 28.,
    y: 610.,
    width: 276.,
    height: 38.,
};
const SECONDARY: Rectangle = Rectangle {
    x: 28.,
    y: 658.,
    width: 276.,
    height: 38.,
};
const PAUSE: Rectangle = Rectangle {
    x: 354.,
    y: 86.,
    width: 136.,
    height: 30.,
};
const AUTO: Rectangle = Rectangle {
    x: 500.,
    y: 86.,
    width: 176.,
    height: 30.,
};
const RESET: Rectangle = Rectangle {
    x: 686.,
    y: 86.,
    width: 164.,
    height: 30.,
};
const SHIP: Rectangle = Rectangle {
    x: 860.,
    y: 86.,
    width: 384.,
    height: 30.,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    Menu,
    Flight,
    Museum,
}
struct Interface {
    help: bool,
    stats: bool,
    captured: bool,
    notice: String,
    notice_time: f32,
    fps: f32,
}
impl Default for Interface {
    fn default() -> Self {
        Self {
            help: true,
            stats: false,
            captured: false,
            notice: String::new(),
            notice_time: 0.,
            fps: 0.,
        }
    }
}
fn hit(rect: Rectangle, mouse: Vector2) -> bool {
    rect.check_collision_point_rec(mouse)
}
fn row(index: usize) -> Rectangle {
    Rectangle::new(28., 154. + index as f32 * 52., 276., 47.)
}
fn draw_button(
    draw: &mut RaylibDrawHandle<'_>,
    rect: Rectangle,
    label: &str,
    hover: bool,
    active: bool,
) {
    draw.draw_rectangle_rec(
        rect,
        if active {
            Color::new(29, 69, 69, 255)
        } else if hover {
            Color::new(38, 64, 81, 255)
        } else {
            Color::new(17, 32, 49, 245)
        },
    );
    draw.draw_rectangle_lines_ex(
        rect,
        1.,
        if active {
            ACCENT
        } else {
            Color::new(66, 95, 117, 255)
        },
    );
    draw.draw_text(
        label,
        rect.x as i32 + 14,
        (rect.y + (rect.height - 16.) * 0.5) as i32,
        16,
        if active { ACCENT } else { INK },
    );
}
pub fn run(mut app: App, config: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use KeyboardKey::*;
    let (mut window, thread) = raylib::init()
        .size(1280, 720)
        .title("NADIR | Observatorio de mundos imposibles")
        .msaa_4x()
        .build();
    window.set_target_fps(60);
    window.set_exit_key(None);
    let image = Image::gen_image_color(app.fb.width as i32, app.fb.height as i32, Color::BLACK);
    let mut texture = window.load_texture_from_image(&thread, &image)?;
    let mut ui = Interface::default();
    let mut screen = Screen::Menu;
    let mut return_to = Screen::Menu;
    let mut museum = Museum::default();
    let mut home = Museum::default();
    let (mut orbiting, mut panning) = (false, false);
    while !window.window_should_close() {
        let dt = window.get_frame_time().clamp(0.0001, 0.1);
        let mouse = window.get_mouse_position();
        let click = window.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let mut next = screen;
        if window.is_key_pressed(KEY_F3) {
            ui.stats = !ui.stats;
        }
        if window.is_key_pressed(KEY_ESCAPE) {
            match screen {
                Screen::Menu => break,
                Screen::Flight => next = Screen::Menu,
                Screen::Museum => next = return_to,
            }
        } else {
            match screen {
                Screen::Menu => {
                    if window.is_key_pressed(KEY_ENTER) || (click && hit(EXPLORE, mouse)) {
                        next = Screen::Flight;
                    }
                    if window.is_key_pressed(KEY_M) || (click && hit(OPEN_MUSEUM, mouse)) {
                        return_to = Screen::Menu;
                        next = Screen::Museum;
                    }
                    if click && hit(QUIT, mouse) {
                        break;
                    }
                }
                Screen::Flight => {
                    if window.is_key_pressed(KEY_M)
                        || (!ui.captured && click && hit(FLIGHT_MUSEUM, mouse))
                    {
                        return_to = Screen::Flight;
                        next = Screen::Museum;
                    }
                }
                Screen::Museum => {
                    if window.is_key_pressed(KEY_M) || (click && hit(BACK, mouse)) {
                        next = return_to;
                    }
                    if click && hit(SECONDARY, mouse) {
                        next = if return_to == Screen::Flight {
                            Screen::Menu
                        } else {
                            Screen::Flight
                        };
                    }
                }
            }
        }
        let changed = screen != next;
        if changed {
            screen = next;
            orbiting = false;
            panning = false;
            if screen == Screen::Flight && ui.captured {
                window.disable_cursor();
            } else {
                window.enable_cursor();
            }
        }
        let metrics = match screen {
            Screen::Menu => {
                home.advance(dt);
                app.render_exhibit(&home)
            }
            Screen::Flight => {
                if !changed {
                    flight_input(
                        &mut app,
                        config,
                        &mut window,
                        &thread,
                        &mut texture,
                        &mut ui,
                        dt,
                    )?;
                }
                app.render()
            }
            Screen::Museum => {
                if !changed {
                    museum_input(&window, &mut museum, dt, &mut orbiting, &mut panning);
                }
                museum.advance(dt);
                app.render_exhibit(&museum)
            }
        };
        texture.update_texture(&app.fb.pixels)?;
        ui.fps += ((1. / dt) - ui.fps) * (1. - (-dt * 2.).exp());
        ui.notice_time -= dt;
        let mut draw = window.begin_drawing(&thread);
        draw.clear_background(Color::new(4, 9, 17, 255));
        let destination = if screen == Screen::Flight {
            Rectangle::new(0., 0., 1280., 720.)
        } else {
            VIEW
        };
        draw.draw_texture_pro(
            &texture,
            Rectangle::new(0., 0., app.fb.width as f32, app.fb.height as f32),
            destination,
            Vector2::zero(),
            0.,
            Color::WHITE,
        );
        match screen {
            Screen::Menu => draw_menu(&mut draw, mouse),
            Screen::Flight => draw_flight(&mut draw, &app, &ui, mouse, metrics),
            Screen::Museum => draw_museum(&mut draw, &app, &museum, &ui, mouse, metrics, return_to),
        }
    }
    Ok(())
}
fn flight_input(
    app: &mut App,
    config: &Path,
    window: &mut RaylibHandle,
    thread: &RaylibThread,
    texture: &mut Texture2D,
    ui: &mut Interface,
    dt: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    use KeyboardKey::*;
    if window.is_key_pressed(KEY_H) {
        ui.help = !ui.help;
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
        ui.captured = !ui.captured;
        if ui.captured {
            window.disable_cursor();
        } else {
            window.enable_cursor();
        }
    }
    let mouse = window.get_mouse_position();
    let button = Rectangle::new(1050., 28., 202., 42.);
    let hover = !ui.captured && button.check_collision_point_rec(mouse);
    if window.is_key_pressed(KEY_G)
        || (hover && window.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT))
    {
        app.flight.jump();
    }
    if window.is_key_pressed(KEY_R) {
        match Config::load(config).and_then(Scene::new) {
            Ok(scene) => {
                *app = App::new(scene);
                let image =
                    Image::gen_image_color(app.fb.width as i32, app.fb.height as i32, Color::BLACK);
                *texture = window.load_texture_from_image(thread, &image)?;
                ui.notice = "Configuracion recargada; vuelo restablecido".into();
            }
            Err(e) => ui.notice = format!("No se aplico la configuracion: {e}"),
        };
        ui.notice_time = 6.;
    }
    let axis =
        |a, b| i32::from(window.is_key_down(a)) as f32 - i32::from(window.is_key_down(b)) as f32;
    let delta = if ui.captured {
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
    Ok(())
}
fn draw_flight(
    draw: &mut RaylibDrawHandle<'_>,
    app: &App,
    ui: &Interface,
    mouse: Vector2,
    metrics: Metrics,
) {
    let button = Rectangle::new(1050., 28., 202., 42.);
    let hover = !ui.captured && hit(button, mouse);
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
        "H ayuda   C raton   TAB destino   F asistencia   X freno   P pausa   M museo   ESC menu",
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
    if ui.help {
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
            "M          Museo de modelos",
        ]
        .iter()
        .enumerate()
        {
            draw.draw_text(text, 922, 167 + i as i32 * 23, 15, ink);
        }
    }
    if ui.stats {
        draw.draw_rectangle(28, 110, 510, 144, Color::new(3, 10, 22, 225));
        for (i, line) in [
            format!(
                "CPU {:.1} ms  |  {:.0} FPS  |  {} x {}",
                metrics.render_ms, ui.fps, app.fb.width, app.fb.height
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
    if ui.notice_time > 0. {
        draw.draw_rectangle(28, 552, 1224, 42, Color::new(12, 25, 38, 240));
        draw.draw_text(&ui.notice, 42, 566, 15, ink);
    }
    draw.draw_line(633, 360, 647, 360, Color::new(180, 208, 220, 120));
    draw.draw_line(640, 353, 640, 367, Color::new(180, 208, 220, 120));
    draw_button(
        draw,
        FLIGHT_MUSEUM,
        "M  MUSEO DE MODELOS",
        !ui.captured && hit(FLIGHT_MUSEUM, mouse),
        false,
    );
}
fn museum_input(
    window: &RaylibHandle,
    museum: &mut Museum,
    dt: f32,
    orbiting: &mut bool,
    panning: &mut bool,
) {
    use KeyboardKey::*;
    let mouse = window.get_mouse_position();
    let click = window.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
    for (i, key) in [
        KEY_ONE, KEY_TWO, KEY_THREE, KEY_FOUR, KEY_FIVE, KEY_SIX, KEY_SEVEN, KEY_EIGHT,
    ]
    .iter()
    .enumerate()
    {
        if window.is_key_pressed(*key) || (click && hit(row(i), mouse)) {
            museum.select(i);
        }
    }
    if window.is_key_pressed(KEY_TAB) {
        museum.next(window.is_key_down(KEY_LEFT_SHIFT));
    }
    if window.is_key_pressed(KEY_P) || (click && hit(PAUSE, mouse)) {
        museum.paused = !museum.paused;
    }
    if window.is_key_pressed(KEY_SPACE) || (click && hit(AUTO, mouse)) {
        museum.auto_rotate = !museum.auto_rotate;
    }
    if window.is_key_pressed(KEY_R) || (click && hit(RESET, mouse)) {
        museum.reset_view();
    }
    if museum.exhibit() == Exhibit::Ship
        && (window.is_key_pressed(KEY_V) || (click && hit(SHIP, mouse)))
    {
        museum.original_ship = !museum.original_ship;
    }
    if click && hit(VIEW, mouse) {
        *orbiting = true;
    }
    if window.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) && hit(VIEW, mouse) {
        *panning = true;
    }
    *orbiting &= window.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
    *panning &= window.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT);
    let delta = window.get_mouse_delta();
    if *orbiting {
        museum.orbit(-delta.x * 0.008, delta.y * 0.008);
    }
    if *panning {
        museum.pan_by(-delta.x * 0.008, delta.y * 0.008);
    }
    if hit(VIEW, mouse) {
        museum.zoom_by(window.get_mouse_wheel_move());
    }
    let axis = |positive, negative| {
        i32::from(window.is_key_down(positive)) as f32
            - i32::from(window.is_key_down(negative)) as f32
    };
    let horizontal = axis(KEY_RIGHT, KEY_LEFT);
    let vertical = axis(KEY_UP, KEY_DOWN);
    if window.is_key_down(KEY_LEFT_SHIFT) {
        museum.pan_by(horizontal * dt, vertical * dt);
    } else {
        museum.orbit(horizontal * dt * 1.2, vertical * dt * 1.2);
    }
    let zoom_in = window.is_key_down(KEY_EQUAL) || window.is_key_down(KEY_KP_ADD);
    let zoom_out = window.is_key_down(KEY_MINUS) || window.is_key_down(KEY_KP_SUBTRACT);
    museum.zoom_by((i32::from(zoom_in) - i32::from(zoom_out)) as f32 * dt * 4.);
}
fn draw_menu(draw: &mut RaylibDrawHandle<'_>, mouse: Vector2) {
    draw.draw_text("N A D I R", 52, 140, 44, INK);
    draw.draw_text("OBSERVATORIO DE MUNDOS IMPOSIBLES", 54, 204, 13, MUTED);
    draw.draw_text("Seis mundos. Una nave.", 54, 244, 18, INK);
    draw.draw_text("Un viaje al nucleo galactico.", 54, 273, 15, MUTED);
    draw_button(
        draw,
        EXPLORE,
        "ENTER  EXPLORAR SISTEMA",
        hit(EXPLORE, mouse),
        false,
    );
    draw_button(
        draw,
        OPEN_MUSEUM,
        "M  MUSEO DE MODELOS",
        hit(OPEN_MUSEUM, mouse),
        false,
    );
    draw_button(draw, QUIT, "SALIR", hit(QUIT, mouse), false);
    draw.draw_text(
        "Explora el sistema o inspecciona cada modelo",
        54,
        556,
        13,
        MUTED,
    );
    draw.draw_text(
        "con zoom, orbita y animaciones en tiempo real.",
        54,
        580,
        13,
        MUTED,
    );
    draw.draw_text("SOL / VISTA EN TIEMPO REAL", 354, 663, 14, MUTED);
    draw.draw_text("ESC salir", 54, 681, 13, MUTED);
}
fn draw_museum(
    draw: &mut RaylibDrawHandle<'_>,
    app: &App,
    museum: &Museum,
    ui: &Interface,
    mouse: Vector2,
    metrics: Metrics,
    return_to: Screen,
) {
    draw.draw_text("MUSEO", 28, 28, 28, INK);
    draw.draw_text("N A D I R", 30, 72, 14, MUTED);
    draw.draw_text("COLECCION / 8 MODELOS", 28, 116, 13, MUTED);
    for index in 0..EXHIBIT_COUNT {
        let rect = row(index);
        let selected = index == museum.selected();
        draw.draw_rectangle_rec(
            rect,
            if selected {
                Color::new(23, 55, 59, 255)
            } else if hit(rect, mouse) {
                Color::new(24, 38, 54, 255)
            } else {
                Color::new(10, 20, 32, 255)
            },
        );
        if selected {
            draw.draw_rectangle(rect.x as i32, rect.y as i32, 3, rect.height as i32, ACCENT);
        }
        draw.draw_text(
            &format!("{:02}", index + 1),
            43,
            rect.y as i32 + 15,
            15,
            if selected { ACCENT } else { MUTED },
        );
        draw.draw_text(
            Museum::label_at(&app.scene, index),
            82,
            rect.y as i32 + 13,
            19,
            if selected { INK } else { MUTED },
        );
    }
    draw.draw_text("TAB siguiente / 1-8 seleccionar", 28, 582, 13, MUTED);
    draw_button(
        draw,
        BACK,
        if return_to == Screen::Flight {
            "ESC  VOLVER AL VUELO"
        } else {
            "ESC  MENU PRINCIPAL"
        },
        hit(BACK, mouse),
        false,
    );
    draw_button(
        draw,
        SECONDARY,
        if return_to == Screen::Flight {
            "MENU PRINCIPAL"
        } else {
            "EXPLORAR SISTEMA"
        },
        hit(SECONDARY, mouse),
        false,
    );
    draw.draw_text(&museum.label(&app.scene).to_uppercase(), 354, 26, 28, INK);
    draw.draw_text(museum.description(&app.scene), 354, 64, 15, MUTED);
    draw.draw_text(
        &format!("FIGURA {:02} / 08", museum.selected() + 1),
        1106,
        35,
        14,
        ACCENT,
    );
    draw_button(
        draw,
        PAUSE,
        if museum.paused {
            "P  REANUDAR"
        } else {
            "P  PAUSAR"
        },
        hit(PAUSE, mouse),
        museum.paused,
    );
    draw_button(
        draw,
        AUTO,
        "ESP  GIRO AUTO",
        hit(AUTO, mouse),
        museum.auto_rotate,
    );
    draw_button(draw, RESET, "R  RESTABLECER", hit(RESET, mouse), false);
    if museum.exhibit() == Exhibit::Ship {
        draw_button(
            draw,
            SHIP,
            if museum.original_ship {
                "V  VER MALLA DE VUELO"
            } else {
                "V  VER MODELO ORIGINAL"
            },
            hit(SHIP, mouse),
            false,
        );
    } else {
        draw.draw_text("MAXIMO DETALLE", 1092, 96, 13, MUTED);
    }
    draw.draw_rectangle_lines_ex(VIEW, 1., Color::new(36, 55, 73, 255));
    draw.draw_text(
        "Arrastrar: orbitar  |  Rueda: zoom  |  Boton derecho: desplazar",
        354,
        657,
        13,
        MUTED,
    );
    draw.draw_text(
        "Flechas / +/-   R vista   TAB figura   P pausa   ESP giro   ESC volver",
        354,
        682,
        13,
        MUTED,
    );
    if ui.stats {
        draw.draw_rectangle(348, 140, 380, 66, Color::new(3, 10, 20, 225));
        draw.draw_text(
            &format!("CPU {:.1} ms  |  {:.0} FPS", metrics.render_ms, ui.fps),
            362,
            152,
            14,
            INK,
        );
        draw.draw_text(
            &format!(
                "{} triangulos  |  zoom {:.2}x",
                metrics.triangles_available, museum.zoom
            ),
            362,
            178,
            14,
            INK,
        );
    }
}
