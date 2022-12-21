use std::{collections::HashMap, time::Instant};

use camera::Camera;
use glam::{vec2, vec3, Vec3};
use image::DynamicImage;
use renderer::Renderer;

use winit::{
    event::{DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};
use world::World;

mod camera;
mod instance;
mod renderer;
mod texture;
mod world;

fn load_tex(name: &str) -> DynamicImage {
    let path = format!("{}.png", name);
    image::load_from_memory(
        std::fs::read(path.as_str())
            .unwrap_or_else(|_| panic!("File {path} not found."))
            .as_slice(),
    )
    .unwrap_or_else(|_| panic!("Couldn't load {path} into an image."))
}

fn main() {
    env_logger::init();
    let ev = EventLoop::new();
    let window = WindowBuilder::new().build(&ev).unwrap();

    let aspect_ratio = (window.inner_size().width / window.inner_size().height) as f32;

    let mut camera =
        Camera::new_projection(Vec3::new(0.0, 0.0, 1.0), 75.0, aspect_ratio, 0.001, 100.0);

    let mut input_state = InputState::new();

    let mut state = State::new();

    let mut renderer = Renderer::new(&window, &camera);

    let textures = vec![
        ("dirt".into(), load_tex("dirt")),
        ("stone".into(), load_tex("stone")),
        ("cobble".into(), load_tex("cobble")),
        ("water".into(), load_tex("water")),
        ("sand".into(), load_tex("sand")),
    ];

    state.world.setup_textures(&mut renderer, textures);

    let mut now = Instant::now();
    let target_fps = 60.0;

    #[allow(clippy::collapsible_match)]
    ev.run(move |event, _, cf| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => cf.set_exit(),
            WindowEvent::Resized(size) => println!("Resized {:?}", size),
            WindowEvent::KeyboardInput {
                device_id: _,
                input,
                is_synthetic: _,
            } => {
                match input.virtual_keycode.unwrap() {
                    VirtualKeyCode::W => input_state
                        .kbd_map
                        .insert("w".into(), input.state == ElementState::Pressed),
                    VirtualKeyCode::S => input_state
                        .kbd_map
                        .insert("s".into(), input.state == ElementState::Pressed),
                    VirtualKeyCode::A => input_state
                        .kbd_map
                        .insert("a".into(), input.state == ElementState::Pressed),
                    VirtualKeyCode::D => input_state
                        .kbd_map
                        .insert("d".into(), input.state == ElementState::Pressed),
                    VirtualKeyCode::Q => input_state
                        .kbd_map
                        .insert("q".into(), input.state == ElementState::Pressed),
                    VirtualKeyCode::E => input_state
                        .kbd_map
                        .insert("e".into(), input.state == ElementState::Pressed),
                    VirtualKeyCode::LShift => input_state
                        .kbd_map
                        .insert("shift".into(), input.state == ElementState::Pressed),
                    _ => {
                        println!("{:?}", input);
                        None
                    }
                };
            }
            _ => (),
        },
        #[allow(clippy::single_match)]
        Event::DeviceEvent {
            device_id: _,
            event,
        } => match event {
            DeviceEvent::MouseMotion { delta } => {
                // println!("mousemove");
                camera.look_add(vec2(-delta.0 as f32 / 100.0, -delta.1 as f32 / 100.0));
                renderer.update_camera(&camera);
            }
            _ => (),
        },
        Event::MainEventsCleared => {
            if now.elapsed().as_secs_f32() >= 1.0 / target_fps {
                now = Instant::now();
                state.update(&input_state, &mut camera);
                state.world.draw(&mut renderer);
                renderer.update_camera(&camera);
                renderer.draw();
            }
        }
        _ => (),
    });
}

/// Creates a Hashmap<String, bool> with value false, accepting a key array.
macro_rules! kbd_map {
    ($($a:expr),*) => {
        {
        let map: HashMap<String, bool> = HashMap::from([
            $(
                ($a.into(), false),
            )*
        ]);
        map
        }
    }
}

struct InputState {
    pub kbd_map: HashMap<String, bool>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            kbd_map: kbd_map!("w", "s", "a", "d", "q", "e", "shift"),
        }
    }
}

fn bool_move(b: bool) -> f32 {
    if b {
        1.0
    } else {
        0.0
    }
}

struct State {
    world: World,
}

impl State {
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }

    pub fn update(&mut self, input_state: &InputState, camera: &mut Camera) {
        let mut movement = Vec3::splat(0.0);
        movement.z = bool_move(*input_state.kbd_map.get("w").unwrap())
            - bool_move(*input_state.kbd_map.get("s").unwrap());
        movement.x = bool_move(*input_state.kbd_map.get("d").unwrap())
            - bool_move(*input_state.kbd_map.get("a").unwrap());
        movement.y = bool_move(*input_state.kbd_map.get("q").unwrap())
            - bool_move(*input_state.kbd_map.get("e").unwrap());

        let shift = *input_state.kbd_map.get("shift").unwrap();

        movement = movement.normalize_or_zero();
        let speed = if !shift { 0.05 } else { 0.5 };
        camera.translate(
            (movement.x * camera.right()
                + movement.y * camera.up()
                + movement.z * camera.look_dir())
            .normalize_or_zero()
                * speed,
        );
        // println!("moving: {}", movement);
    }
}

// fn main() {
//     engine::app::App::new().window_size((600.0, 600.0));
// }
