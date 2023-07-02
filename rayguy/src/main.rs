use std::{
    num::NonZeroU32,
    time::{Duration, Instant},
};

mod math;

use ezbuffer::WrapBuffer;
use ezbuffer::{Color, BLUE, GREEN, RED, WHITE, YELLOW};
use math::Vec2;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const TEST_LEVEL_WIDTH: usize = 24;
const TEST_LEVEL_HEIGHT: usize = 24;

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 480;

// Gameboy resolution:)
const SURFACE_WIDTH: u32 = 160;
const SURFACE_HEIGHT: u32 = 144;

#[rustfmt::skip]
const TEST_LEVEL: [[u8; TEST_LEVEL_WIDTH]; TEST_LEVEL_HEIGHT] = [
  [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
  [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,0,0,0,0,0,2,2,2,2,2,0,0,0,0,3,0,3,0,3,0,0,0,1],
  [1,0,0,0,0,0,2,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,0,0,0,0,0,2,0,0,0,2,0,0,0,0,3,0,0,0,3,0,0,0,1],
  [1,0,0,0,0,0,2,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,0,0,0,0,0,2,2,0,2,2,0,0,0,0,3,0,3,0,3,0,0,0,1],
  [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,4,4,4,4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,4,0,4,0,0,0,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,4,0,0,0,0,5,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,4,0,4,0,0,0,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,4,0,4,4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,4,4,4,4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
  [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]
];

#[rustfmt::skip]
const WALL_TEXTURE: [[u32; 8]; 8] = [
    [0xff, 0xff,   0xff,   0xff,     0xff,     0xff,   0xff,   0xff],
    [0xff, 0xff00, 0xff00, 0xff0000, 0xff0000, 0xff00, 0xff00, 0xff],
    [0xff, 0xff00, 0xff00, 0xff0000, 0xff0000, 0xff00, 0xff00, 0xff],
    [0xff, 0xff00, 0xff00, 0xff0000, 0xff0000, 0xff00, 0xff00, 0xff],
    [0xff, 0xff00, 0xff00, 0xff0000, 0xff0000, 0xff00, 0xff00, 0xff],
    [0xff, 0xff00, 0xff00, 0xff0000, 0xff0000, 0xff00, 0xff00, 0xff],
    [0xff, 0xff00, 0xff00, 0xff0000, 0xff0000, 0xff00, 0xff00, 0xff],
    [0xff, 0xff,   0xff,   0xff,     0xff,     0xff,   0xff,   0xff],
];

fn get_time() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let stop = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    stop.as_millis()
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT))
        .with_title("Raycaster")
        .build(&event_loop)
        .unwrap();

    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();

    // FOV = 2 * atan(0.66/1.0)=66°
    let mut pos = Vec2::new(22., 12.); // x and y start position
    let mut dir = Vec2::new(-1., 0.); // initial direction vector
    let mut plane = Vec2::new(0., 0.66); // the 2d raycaster version of camera plane

    let mut time = 0u64;
    let mut frame_time = 0u64;

    let mut pressed_keys = [false; 256];
    event_loop.run(move |event, _, control_flow| {
        // Hard cap at 144 FPS
        *control_flow = ControlFlow::WaitUntil(
            Instant::now()
                .checked_add(Duration::from_micros(1_000_000 / 144))
                .unwrap(),
        );

        match event {
            Event::MainEventsCleared => {
                let old_time = time;
                time = get_time() as u64;
                frame_time = time - old_time;
                window.request_redraw();
            }

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let (width, height) = {
                    let size = window.inner_size();
                    (size.width, size.height)
                };

                surface
                    .resize(
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    )
                    .unwrap();

                let mut buffer = surface.buffer_mut().unwrap();
                buffer.fill(0);

                let mut buf = WrapBuffer::new(
                    buffer,
                    (SURFACE_WIDTH as usize, SURFACE_HEIGHT as usize),
                    (width as usize, height as usize),
                );
                let (width, height) = (SURFACE_WIDTH, SURFACE_HEIGHT);

                for x in 0..width {
                    // https://lodev.org/cgtutor/raycasting.html:
                    // " cameraX is the x-coordinate on the camera plane that the current x-coordinate of the screen represents,
                    // done this way so that the right side of the screen will get coordinate 1, the center of the screen gets
                    // coordinate 0, and the left side of the screen gets coordinate -1. "
                    // So if x is at the left side of the screen camera_x becomes -1 and causes makes ray_dir point to the left.
                    // If x is -0.5, ray_dir will point to the left (left of the direction vector) but only
                    // as much as half of the plane vector.
                    let camera_x: f64 = 2. * x as f64 / width as f64 - 1.;
                    let ray_dir = &dir + &(&plane * camera_x);
                    let mut map_pos = Vec2::new(pos.x as i32 as f64, pos.y as i32 as f64);

                    let mut side_dist = Vec2::new(0., 0.);

                    // When you try to derive this formula, you will get |ray_dir| / ray_dir.x and |ray_dir| / ray_dir.y
                    // (Where |ray_dir| is the length of the ray_dir vector) When you simplify the entire equation
                    // (including some calculations after this one), you will see that |ray_dir| can be discarded
                    // (because we only really need the ratio between ray_dir.x and .y).
                    let delta_dist = Vec2::new((1. / ray_dir.x).abs(), (1. / ray_dir.y).abs());

                    // let perp_wall_dist: f64; // will be used to calculate |ray_dir|

                    let mut step = Vec2::new(0., 0.); // should be either +1 or -1. //TODO make this less ambigious

                    let mut hit = 0;
                    let mut side = 0;

                    if ray_dir.x < 0. {
                        step.x = -1.;
                        side_dist.x = (pos.x - map_pos.x) * delta_dist.x;
                    } else {
                        step.x = 1.;
                        // map_pos.x < pos.x < map_pos.x + 1 and the expression below needs to be positive
                        // and also map_pos.x + 1 - pos.x is the actaul geometrically correct length to use here
                        // since map_pos is the left-bottom corner of a square.

                        // delta_dist.x is the amount of distance the ray needs to go to reach the closest vertical line
                        // that is 1 unit away from the starting position on the x axis.
                        side_dist.x = (map_pos.x + 1. - pos.x) * delta_dist.x;
                    }

                    if ray_dir.y < 0. {
                        step.y = -1.;
                        side_dist.y = (pos.y - map_pos.y) * delta_dist.y;
                    } else {
                        step.y = 1.;
                        // delta_dist.y is the amount of distance the ray needs to go to reach the closest horizontal line
                        // that is 1 unit away from the starting position on the y axis.
                        side_dist.y = (map_pos.y + 1. - pos.y) * delta_dist.y;
                    }

                    // Perform DDA (Digital Differential Analysis)
                    while hit == 0 {
                        // Step towards the closest side
                        if side_dist.x < side_dist.y {
                            side_dist.x += delta_dist.x;
                            map_pos.x += step.x;
                            side = 0;
                        } else {
                            side_dist.y += delta_dist.y;
                            map_pos.y += step.y;
                            side = 1;
                        }

                        // Check if it was a hit
                        if TEST_LEVEL[map_pos.x as usize][map_pos.y as usize] > 0 {
                            hit = 1;
                        }
                    }

                    // To avoid the fisheye effect, we calculate the distance between the point and the camera _plane_.
                    // (Hence the name, perp(enducilar)_wall_distance)
                    // You can calculate the actual euclidean distance between the camera and the hit point but that would be more
                    // work and would result in the aforementioned fisheye effect.

                    // The equation below can be derived from a bunch of similar triangles and ratios between the hit point and the
                    // camera plane. If you do not want to sit down and derive the same equation, you can imagine the
                    // -delta_dist part as just going a step back to get out of the wall after the last DDA step.
                    let perp_wall_dist: f64 = if side == 0 {
                        side_dist.x - delta_dist.x
                    } else {
                        side_dist.y - delta_dist.y
                    };

                    let hit_point = ray_dir * perp_wall_dist;
                    let hit_relative = Vec2::new(
                        hit_point.x - hit_point.x as i32 as f64,
                        hit_point.y - hit_point.y as i32 as f64,
                    );
                    let strip = if side == 0 {
                        (hit_relative.x * 8.) as usize
                    } else {
                        (hit_relative.y * 8.) as usize
                    };
                    let colors = &WALL_TEXTURE[strip];

                    let line_height = (width as f64 / perp_wall_dist) as i32;
                    let draw_start = {
                        let mut y = -line_height / 2 + height as i32 / 2;
                        if y < 0 {
                            y = 0;
                        }
                        y
                    };
                    let draw_end = {
                        let mut y = line_height / 2 + height as i32 / 2;
                        if y >= height as i32 {
                            y = height as i32 - 1;
                        }
                        y
                    };

                    // texture rendering i guess?
                    // for y in draw_start..draw_end {
                    //     let color_index = (8 * y / line_height).clamp(0, 7) as usize;
                    //     let mut color = colors[color_index];
                    //     if side == 1 {
                    //         let r = ((color & 0xff0000) >> 16) / 2;
                    //         let g = ((color & 0xff00) >> 8) / 2;
                    //         let b = (color & 0xff) / 2;
                    //         color = (r << 16) | (g << 8) | b;
                    //     }
                    //     buf.set_raw(x as usize, y as usize, color);
                    // }

                    let mut color = match TEST_LEVEL[map_pos.x as usize][map_pos.y as usize] {
                        1 => RED,
                        2 => GREEN,
                        3 => BLUE,
                        4 => WHITE,
                        _ => YELLOW,
                    };

                    if side == 1 {
                        match color {
                            Color::Rgb(r, g, b) => color = Color::Rgb(r / 2, g / 2, b / 2),
                        }
                    }

                    buf.vert_line(x as usize, draw_start as usize, draw_end as usize, color);
                }

                buf.present().unwrap();

                let move_speed = frame_time as f64 * 2. / 1000.;
                let rot_speed = frame_time as f64 * 1.5 / 1000. * std::f64::consts::PI;

                if pressed_keys[VirtualKeyCode::Up as usize] {
                    if TEST_LEVEL[(pos.x + dir.x * move_speed) as usize][pos.y as usize] == 0 {
                        pos.x += dir.x * move_speed;
                    }
                    if TEST_LEVEL[pos.x as usize][(pos.y + dir.y * move_speed) as usize] == 0 {
                        pos.y += dir.y * move_speed;
                    }
                }
                if pressed_keys[VirtualKeyCode::Down as usize] {
                    if TEST_LEVEL[(pos.x - dir.x * move_speed) as usize][pos.y as usize] == 0 {
                        pos.x -= dir.x * move_speed;
                    }
                    if TEST_LEVEL[pos.x as usize][(pos.y - dir.y * move_speed) as usize] == 0 {
                        pos.y -= dir.y * move_speed;
                    }
                }
                if pressed_keys[VirtualKeyCode::Right as usize] {
                    let old_dir_x = dir.x;
                    dir.x = dir.x * (-rot_speed).cos() - dir.y * (-rot_speed).sin();
                    dir.y = old_dir_x * (-rot_speed).sin() + dir.y * (-rot_speed).cos();
                    let old_plane_x = plane.x;
                    plane.x = plane.x * (-rot_speed).cos() - plane.y * (-rot_speed).sin();
                    plane.y = old_plane_x * (-rot_speed).sin() + plane.y * (-rot_speed).cos();
                }
                if pressed_keys[VirtualKeyCode::Left as usize] {
                    let old_dir_x = dir.x;
                    dir.x = dir.x * (rot_speed).cos() - dir.y * (rot_speed).sin();
                    dir.y = old_dir_x * (rot_speed).sin() + dir.y * (rot_speed).cos();
                    let old_plane_x = plane.x;
                    plane.x = plane.x * (rot_speed).cos() - plane.y * (rot_speed).sin();
                    plane.y = old_plane_x * (rot_speed).sin() + plane.y * (rot_speed).cos();
                }
            }

            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                window_id,
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                        ..
                    },
            } if window_id == window.id() => match state {
                ElementState::Pressed => pressed_keys[keycode as usize] = true,
                ElementState::Released => pressed_keys[keycode as usize] = false,
            },

            _ => (),
        }
    });
}
