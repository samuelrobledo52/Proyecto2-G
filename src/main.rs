mod camera;
mod entities;
mod material;
mod math;
mod post;
mod render;
mod scene;
mod skybox;
mod texture;

use camera::OrbitCamera;
use math::Vec3;
use post::Post;
use raylib::prelude::*;
use render::World;
use skybox::Sky;

const WIN_W: i32 = 1280;
const WIN_H: i32 = 720;
const DAY_SECONDS: f32 = 120.0;

struct Target {
    w: usize,
    h: usize,
    hdr: Vec<Vec3>,
    buf: Vec<u8>,
    tex: Texture2D,
    post: Post,
}

fn make_target(rl: &mut RaylibHandle, thread: &RaylibThread, scale: i32) -> Target {
    let w = WIN_W / scale;
    let h = WIN_H / scale;
    let img = Image::gen_image_color(w, h, Color::BLACK);
    let tex = rl
        .load_texture_from_image(thread, &img)
        .expect("No se pudo crear la textura del framebuffer");
    let (w, h) = (w as usize, h as usize);
    Target {
        w,
        h,
        hdr: vec![Vec3::ZERO; w * h],
        buf: vec![0u8; w * h * 4],
        tex,
        post: Post::new(w, h),
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIN_W, WIN_H)
        .title("Diorama Raytracing - Samuel Robledo")
        .build();
    rl.set_target_fps(60);

    let mut world = World::new();
    world.update(0.0, 0.0, 0.0);
    let mut cam = OrbitCamera::new(world.scene.center());

    let mut scale: i32 = 2;
    let mut tgt = make_target(&mut rl, &thread, scale);

    let speeds = [1.0f32, 4.0, 0.25];
    let mut speed_i = 0usize;
    let mut tod = 0.20f32;
    let mut time = 0.0f32;
    let mut frame: u32 = 0;
    let mut photo = false;
    let mut bloom = true;
    let mut auto_rotate = true;
    let mut show_help = true;
    let mut last_cam = (cam.yaw, cam.pitch, cam.distance);

    while !rl.window_should_close() {
        let dt = rl.get_frame_time().min(0.1);

        let rs = 1.3 * dt;
        if rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A) {
            cam.rotate(-rs, 0.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) || rl.is_key_down(KeyboardKey::KEY_D) {
            cam.rotate(rs, 0.0);
        }
        if rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W) {
            cam.rotate(0.0, rs * 0.7);
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S) {
            cam.rotate(0.0, -rs * 0.7);
        }
        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let md = rl.get_mouse_delta();
            cam.rotate(md.x * 0.005, md.y * 0.005);
        }
        if auto_rotate && !photo {
            cam.rotate(0.15 * dt, 0.0);
        }

        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 {
            cam.zoom(1.0 - wheel * 0.08);
        }
        if rl.is_key_down(KeyboardKey::KEY_Q) {
            cam.zoom(1.0 - 1.2 * dt);
        }
        if rl.is_key_down(KeyboardKey::KEY_E) {
            cam.zoom(1.0 + 1.2 * dt);
        }

        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            auto_rotate = !auto_rotate;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_N) {
            tod = (tod + 0.25) % 1.0;
            tgt.post.reset();
        }
        if rl.is_key_pressed(KeyboardKey::KEY_T) {
            speed_i = (speed_i + 1) % speeds.len();
        }
        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            photo = !photo;
            tgt.post.reset();
        }
        if rl.is_key_pressed(KeyboardKey::KEY_B) {
            bloom = !bloom;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_H) {
            show_help = !show_help;
        }
        let mut new_scale = scale;
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            new_scale = 1;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            new_scale = 2;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            new_scale = 3;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
            new_scale = 4;
        }
        if new_scale != scale {
            scale = new_scale;
            tgt = make_target(&mut rl, &thread, scale);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F12) {
            rl.take_screenshot(&thread, "captura.png");
        }

        if !photo {
            time += dt;
            tod = (tod + dt * speeds[speed_i] / DAY_SECONDS) % 1.0;
        }
        let st = world.sky.state(tod);
        if !photo {
            world.update(dt, time, st.night);
        }
        let cam_now = (cam.yaw, cam.pitch, cam.distance);
        if cam_now != last_cam {
            tgt.post.reset();
            last_cam = cam_now;
        }

        let cf = cam.frame(tgt.w as f32 / tgt.h as f32);
        world.render(&mut tgt.hdr, tgt.w, tgt.h, &cf, &st, time, frame, photo);
        tgt.post.resolve(&tgt.hdr, photo, bloom, &mut tgt.buf);
        let _ = tgt.tex.update_texture(&tgt.buf);
        frame = frame.wrapping_add(1);

        let fps = rl.get_fps();
        let hours = (tod * 24.0 + 6.0) % 24.0;
        let hh = hours as i32;
        let mm = ((hours - hh as f32) * 60.0) as i32;
        let phase = Sky::phase_name(&st, tod);
        let samples = tgt.post.samples();
        let mobs_on = world.ents.mobs_visible;

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture_ex(&tgt.tex, Vector2::new(0.0, 0.0), 0.0, scale as f32, Color::WHITE);

        let ink = Color::new(235, 235, 235, 255);
        let dim = Color::new(170, 170, 170, 255);
        let key = Color::new(255, 205, 110, 255);

        let top = format!(
            "{} FPS   {}x{}   {:02}:{:02} {}",
            fps, tgt.w, tgt.h, hh, mm, phase
        );
        let status = if photo {
            format!("Pausado - mejorando imagen ({} pasadas)", samples)
        } else if mobs_on {
            "Noche: hay mobs en la isla".to_string()
        } else {
            format!("Tiempo x{}", speeds[speed_i])
        };

        let controls: [(&str, &str); 11] = [
            ("Flechas / WASD", "Girar la camara"),
            ("Arrastrar mouse", "Girar la camara"),
            ("Rueda / Q, E", "Acercar y alejar"),
            ("R", "Giro automatico"),
            ("N", "Adelantar 6 horas"),
            ("T", "Velocidad del tiempo"),
            ("P", "Pausar y mejorar imagen"),
            ("B", "Resplandor on/off"),
            ("1  2  3  4", "Resolucion (1 = maxima)"),
            ("F12", "Guardar captura"),
            ("H", "Ocultar controles"),
        ];

        let panel_h = if show_help { 70 + controls.len() as i32 * 22 + 8 } else { 90 };
        d.draw_rectangle(8, 8, 380, panel_h, Color::new(0, 0, 0, 160));
        d.draw_text(&top, 18, 18, 18, ink);
        d.draw_text(&status, 18, 42, 16, if photo { key } else { dim });

        if show_help {
            d.draw_line(18, 66, 378, 66, Color::new(255, 255, 255, 40));
            for (i, (k, a)) in controls.iter().enumerate() {
                let y = 76 + i as i32 * 22;
                d.draw_text(k, 18, y, 16, key);
                d.draw_text(a, 170, y, 16, ink);
            }
        } else {
            d.draw_text("H: ver controles", 18, 68, 14, dim);
        }
    }
}
