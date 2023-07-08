use std::{
    num::NonZeroU32,
    time::{Duration, Instant},
};

mod consts;
mod math;
use consts::*;

use ezbuffer::WrapBuffer;
// use ezbuffer::{Color, BLUE, GREEN, RED, WHITE, YELLOW};
use math::Vec2;
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

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

    // TEMPORARY CODE TO GENERATE TEXTURES TODO remove later
    let mut texture = vec![vec![0u32; (TEXTURE_WIDTH * TEXTURE_HEIGHT) as usize]; 8];
    for x in 0..TEXTURE_WIDTH {
        for y in 0..TEXTURE_HEIGHT {
            let xor_color = (x * 256 / TEXTURE_WIDTH) ^ (y * 256 / TEXTURE_HEIGHT);
            //int xcolor = x * 256 / texWidth;
            let y_color = y * 256 / TEXTURE_HEIGHT;
            let xy_color = y * 128 / TEXTURE_HEIGHT + x * 128 / TEXTURE_WIDTH;
            texture[0][(TEXTURE_WIDTH * y + x) as usize] =
                65536 * 254 * ((x != y) && x != (TEXTURE_WIDTH - y)) as u32; //flat red texture with black cross
            texture[1][(TEXTURE_WIDTH * y + x) as usize] =
                xy_color + 256 * xy_color + 65536 * xy_color; //sloped greyscale
            texture[2][(TEXTURE_WIDTH * y + x) as usize] = 256 * xy_color + 65536 * xy_color; //sloped yellow gradient
            texture[3][(TEXTURE_WIDTH * y + x) as usize] =
                xor_color + 256 * xor_color + 65536 * xor_color; //xor greyscale
            texture[4][(TEXTURE_WIDTH * y + x) as usize] = 256 * xor_color; //xor green
            texture[5][(TEXTURE_WIDTH * y + x) as usize] =
                65536 * 192 * ((x % 16 != 0) && (y % 16 != 0)) as u32; //red bricks
            texture[6][(TEXTURE_WIDTH * y + x) as usize] = 65536 * y_color; //red gradient
            texture[7][(TEXTURE_WIDTH * y + x) as usize] = 128 + 256 * 128 + 65536 * 128;
            //flat grey texture
        }
    }
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

                // let mut buf = WrapBuffer::new(
                //     buffer,
                //     (width as usize, height as usize),
                //     (width as usize, height as usize),
                // );

                let mut buf = WrapBuffer::new(
                    buffer,
                    (SURFACE_WIDTH as usize, SURFACE_HEIGHT as usize),
                    (width as usize, height as usize),
                );
                let (width, height) = (SURFACE_WIDTH, SURFACE_HEIGHT);

                // Floor
                for y in 0..height {
                    let ray_dir_leftmost = &dir - &plane;
                    let ray_dir_rightmost = &dir + &plane;

                    let horizon_distance = y - height / 2;
                    let horizon_distance = horizon_distance as f64;

                    // Camera vertical position
                    let pos_z = height as f64 * 0.5;

                    // If you were to put a point in front of the camera that is with a
                    // horizontal distance of 1 and a vertical distance of horizon_distance and extend it
                    // to hit the floor (make the vertical distance equal to pos_z) you would multiply the vector by
                    // pos_z / p which would make the horizontal distance 1 * pos_z / p = pos_z / p
                    let row_distance = pos_z / horizon_distance;

                    let floor_step =
                        (&ray_dir_rightmost - &ray_dir_leftmost) * (row_distance / width as f64);

                    let mut floor = &pos + &(ray_dir_leftmost * row_distance);

                    for x in 0..width {
                        let (cell_x, cell_y) = (floor.x as usize, floor.y as usize);

                        let texture_x = (TEXTURE_WIDTH as f64 * (floor.x - cell_x as f64)) as usize
                            & (TEST_LEVEL_WIDTH - 1);
                        let texture_y = (TEXTURE_HEIGHT as f64 * (floor.y - cell_y as f64))
                            as usize
                            & (TEST_LEVEL_HEIGHT - 1);

                        floor.x += floor_step.x;
                        floor.y += floor_step.y;

                        let mut floor_texture = if (cell_x + cell_y) % 2 == 0 { 2 } else { 4 };
                        if cell_x < 24 && cell_y < 24 {
                            if TEST_LEVEL[cell_x][cell_y] < 0 {
                                floor_texture = -TEST_LEVEL[cell_x][cell_y] as usize;
                            }
                        }
                        // let ceiling_texture = 4;
                        let ceiling_texture = 7 - floor_texture;

                        // Floor
                        let color =
                            texture[floor_texture][TEXTURE_WIDTH as usize * texture_y + texture_x];
                        let color = (color >> 1) & 8355711;
                        buf.set_raw(x as usize, y as usize, color);

                        // Ceiling (symmetrical)

                        let color = texture[ceiling_texture]
                            [TEXTURE_WIDTH as usize * texture_y + texture_x];
                        let color = (color >> 1) & 8355711;
                        buf.set_raw(x as usize, (height - y - 1) as usize, color);
                    }
                }

                // Walls
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

                    let line_height = (width as f64 / perp_wall_dist) as i32;
                    let draw_start = {
                        let mut y = -line_height / 2 + height as i32 / 2;
                        if y < 0 {
                            y = 0;
                        }
                        y as usize
                    };
                    let draw_end = {
                        let mut y = line_height / 2 + height as i32 / 2;
                        if y >= height as i32 {
                            y = height as i32 - 1;
                        }
                        y as usize
                    };

                    // - 1 so that texture 0 can be used
                    let tex_num = TEST_LEVEL[map_pos.x as usize][map_pos.y as usize] - 1;

                    let wall_x = {
                        // in my version x and y were flipped, which is probably one of the reasons why it didnt work
                        if side == 0 {
                            pos.y + perp_wall_dist * ray_dir.y
                        } else {
                            pos.x + perp_wall_dist * ray_dir.x
                        }
                    };
                    let wall_x = wall_x - wall_x.floor(); // basically what i did previously

                    let texture_x = (wall_x * (TEXTURE_WIDTH as f64)) as u32;
                    let texture_x = {
                        if side == 0 && ray_dir.x > 0. || side == 1 && ray_dir.y < 0. {
                            TEXTURE_WIDTH - texture_x - 1
                        } else {
                            texture_x
                        }
                    };

                    let step = 1. * TEXTURE_HEIGHT as f64 / line_height as f64;
                    let mut tex_pos =
                        (draw_start as f64 - height as f64 / 2. + line_height as f64 / 2.) * step;

                    for y in draw_start..draw_end {
                        let texture_y = tex_pos as usize & (TEXTURE_HEIGHT as usize - 1);
                        tex_pos += step;
                        let mut color = texture[tex_num as usize]
                            [TEXTURE_HEIGHT as usize * texture_y + texture_x as usize];
                        if side == 1 {
                            // 8355711 is the decimal value of 0b00000000011111110111111101111111 which is the mask we use the divide all 3 values by 2
                            color = (color >> 1) & 8355711;
                        };
                        buf.set_raw(x as usize, y, color);
                    }

                    // let mut color = match TEST_LEVEL[map_pos.x as usize][map_pos.y as usize] {
                    //     1 => RED,
                    //     2 => GREEN,
                    //     3 => BLUE,
                    //     4 => WHITE,
                    //     _ => YELLOW,
                    // };

                    // if side == 1 {
                    //     match color {
                    //         Color::Rgb(r, g, b) => color = Color::Rgb(r / 2, g / 2, b / 2),
                    //     }
                    // }

                    // buf.vert_line(x as usize, draw_start as usize, draw_end as usize, color);
                }

                buf.present().unwrap();

                let move_speed = frame_time as f64 * 2. / 1000.;
                let move_speed = {
                    if pressed_keys[VirtualKeyCode::Space as usize] {
                        move_speed * 2.5
                    } else {
                        move_speed
                    }
                };
                let rot_speed = frame_time as f64 * 0.85 / 1000. * std::f64::consts::PI;

                if pressed_keys[VirtualKeyCode::Up as usize] {
                    if TEST_LEVEL[(pos.x + dir.x * move_speed) as usize][pos.y as usize] <= 0 {
                        pos.x += dir.x * move_speed;
                    }
                    if TEST_LEVEL[pos.x as usize][(pos.y + dir.y * move_speed) as usize] <= 0 {
                        pos.y += dir.y * move_speed;
                    }
                }
                if pressed_keys[VirtualKeyCode::Down as usize] {
                    if TEST_LEVEL[(pos.x - dir.x * move_speed) as usize][pos.y as usize] <= 0 {
                        pos.x -= dir.x * move_speed;
                    }
                    if TEST_LEVEL[pos.x as usize][(pos.y - dir.y * move_speed) as usize] <= 0 {
                        pos.y -= dir.y * move_speed;
                    }
                }
                if pressed_keys[VirtualKeyCode::Right as usize] {
                    dir.rotate(-rot_speed);
                    plane.rotate(-rot_speed);
                }
                if pressed_keys[VirtualKeyCode::Left as usize] {
                    dir.rotate(rot_speed);
                    plane.rotate(rot_speed);
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
