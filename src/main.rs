use pixels::{Error, Pixels, SurfaceTexture};
use std::process::exit;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::WindowBuilder;

#[derive(Debug, Clone, Copy)]
enum Facing {
    NORTH,
    EAST,
    SOUTH,
    WEST,
}

struct Ant {
    x: i32,
    y: i32,
    facing: Facing,
}

impl Ant {
    fn new(x: i32, y: i32, facing: Facing) -> Ant {
        Ant { x, y, facing }
    }
}

struct State {
    ant: Ant,
    pattern: String,
    length: i32,
    board: Vec<u8>,
}

impl State {
    fn new(length: i32, pattern: String) -> State {
        let board_size = (length * length) as usize;
        State {
            board: vec![0; board_size],
            length,
            pattern,
            ant: Ant::new(length / 2, length / 2, Facing::NORTH),
        }
    }

    fn render_to_pixels(&self, frame: &mut [u8], scale: usize) {
        let size = self.length as usize;

        for y in 0..size {
            for x in 0..size {
                let idx = y * size + x;
                let value = self.board[idx];

                let (r, g, b) = match value {
                    0 => (0, 0, 0),
                    1 => (255, 255, 255),
                    _ => (200, 50, 200),
                };

                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = x * scale + dx;
                        let py = y * scale + dy;
                        let pidx = (py * (size * scale) + px) * 4;
                        frame[pidx] = r;
                        frame[pidx + 1] = g;
                        frame[pidx + 2] = b;
                        frame[pidx + 3] = 255;
                    }
                }
            }
        }

        let ax = self.ant.x as usize;
        let ay = self.ant.y as usize;
        for dy in 0..scale {
            for dx in 0..scale {
                let px = ax * scale + dx;
                let py = ay * scale + dy;
                let pidx = (py * (size * scale) + px) * 4;
                frame[pidx] = 255;
                frame[pidx + 1] = 0;
                frame[pidx + 2] = 0;
                frame[pidx + 3] = 255;
            }
        }
    }
}

fn switch_facing(facing: Facing, is_clock: bool) -> Facing {
    match (facing, is_clock) {
        (Facing::NORTH, true) => Facing::EAST,
        (Facing::EAST, true) => Facing::SOUTH,
        (Facing::SOUTH, true) => Facing::WEST,
        (Facing::WEST, true) => Facing::NORTH,
        (Facing::NORTH, false) => Facing::WEST,
        (Facing::EAST, false) => Facing::NORTH,
        (Facing::SOUTH, false) => Facing::EAST,
        (Facing::WEST, false) => Facing::SOUTH,
    }
}

fn idx_from_xy(s: &State, x: i32, y: i32) -> i32 {
    y * s.length + x
}

fn move_forward(ant: &mut Ant) {
    match ant.facing {
        Facing::NORTH => ant.y -= 1,
        Facing::EAST => ant.x += 1,
        Facing::SOUTH => ant.y += 1,
        Facing::WEST => ant.x -= 1,
    }
}

fn next_state(state: &mut State) {
    let ant_x = state.ant.x;
    let ant_y = state.ant.y;
    let length = state.length;

    if ant_x < 0 || ant_x >= length || ant_y < 0 || ant_y >= length {
        println!("\nFATAL: The ant has moved out of bounds. Exiting.");
        exit(1);
    }

    let idx = idx_from_xy(state, ant_x, ant_y);
    let board_index = idx as usize;
    let current_state = state.board[board_index] as usize;

    let total_states = state.pattern.len();

    if current_state >= total_states {
        println!(
            "\nFatal: The state of the board is invalid ({} >= {}). Exiting.",
            current_state, total_states
        );
        exit(1);
    }

    let turn: char = state.pattern.chars().nth(current_state).unwrap();

    let is_clockwise = turn == 'R';
    state.ant.facing = switch_facing(state.ant.facing, is_clockwise);

    move_forward(&mut state.ant);

    let next_state = (current_state + 1) % total_states;
    state.board[board_index] = next_state as u8;
}

fn main() -> Result<(), Error> {
    const WIDTH: usize = 800;
    const HEIGHT: usize = 800;
    const STEPS_PER_FRAME: u32 = 100;

    let event_loop = EventLoop::new().unwrap();
    let logical_size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);

    let window = WindowBuilder::new()
        .with_title("Langton Ant")
        .with_inner_size(logical_size)
        .with_min_inner_size(logical_size)
        .build(&event_loop)
        .expect("Error al crear la ventana winit");

    let mut state = State::new(WIDTH as i32, "RLR".to_string());

    let physical_size = window.inner_size();

    let surface_texture = SurfaceTexture::new(physical_size.width, physical_size.height, &window);

    let mut pixels = Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?;

    let _ = event_loop.run(move |event, elwt: &EventLoopWindowTarget<()>| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                if let Err(e) = pixels.resize_surface(size.width, size.height) {
                    eprintln!("Error resizing surface: {}", e);
                    elwt.exit();
                }
            }

            Event::AboutToWait => {
                for _ in 0..STEPS_PER_FRAME {
                    next_state(&mut state);
                }
                window.request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let frame = pixels.frame_mut();
                state.render_to_pixels(frame, 1);

                if pixels.render().is_err() {
                    eprintln!("Error rendering pixels");
                    elwt.exit();
                }
            }

            _ => {}
        }
    });
    Ok(())
}
