use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use serde::{Deserialize, Serialize};


#[derive(Parser)]
enum Mode {
    Create(CreateOpts),
    Open(OpenOpts),
    ToPng(ToPngOpts),
}

#[derive(Parser)]
struct CreateOpts {
    #[arg(short = 'W', long)]
    pub width: u32,

    #[arg(short = 'H', long)]
    pub height: u32,

    pub filename: PathBuf,
}

#[derive(Parser)]
struct OpenOpts {
    pub filename: PathBuf,
}

#[derive(Parser)]
struct ToPngOpts {
    pub field_filename: PathBuf,
    pub png_filename: PathBuf,
}


#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Option<bool>>,
}
impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count: usize = (width * height).try_into().unwrap();
        let pixels = vec![None; pixel_count];
        Self {
            width,
            height,
            pixels,
        }
    }
}


struct UiState {
    pub image: Image,
    pub x: u32,
    pub y: u32,
    pub setting_mode: bool,
    pub going_right: bool,
}
impl UiState {
    pub fn new(image: Image) -> Self {
        Self {
            image,
            x: 0,
            y: 0,
            setting_mode: false,
            going_right: true,
        }
    }
}


const COLOR_TRUE: Color = Color::RGB(0xFF, 0xFF, 0xFF);
const COLOR_FALSE: Color = Color::RGB(0x00, 0x00, 0x00);
const COLOR_NONE: Color = Color::RGB(0x7F, 0x7F, 0x7F);
const COLOR_CURSOR: Color = Color::RGB(0xFF, 0x00, 0x00);
const COLOR_PREVIEW_FRAME: Color = Color::RGB(0x00, 0x00, 0xFF);
const COLOR_FULL_FRAME: Color = Color::RGB(0x33, 0x33, 0x33);


macro_rules! u32 { ($val:expr) => (u32::try_from($val).unwrap()); }
macro_rules! i32 { ($val:expr) => (i32::try_from($val).unwrap()); }
macro_rules! usize { ($val:expr) => (usize::try_from($val).unwrap()); }


#[inline]
fn color_for_db_bool(value: Option<bool>) -> Color {
    // I call three-state booleans (true, false, null) "database booleans"
    match value {
        Some(true) => COLOR_TRUE,
        Some(false) => COLOR_FALSE,
        None => COLOR_NONE,
    }
}


fn render(canvas: &mut Canvas<Window>, ui_state: &UiState) {
    let (canvas_width, canvas_height) = canvas.window().size();

    canvas.set_draw_color(COLOR_NONE);
    canvas.clear();

    // paint a detailed preview of the current image
    const DETAIL_LOOKAROUND: i32 = 5;
    const DETAIL_PIXEL_SCALE: u32 = 32;
    const DETAIL_BORDER_OFFSET: u32 = 4;
    for y_offset in -DETAIL_LOOKAROUND..=DETAIL_LOOKAROUND {
        let y_coord = i32!(ui_state.y) + y_offset;
        if y_coord < 0 || y_coord >= i32!(ui_state.image.height) {
            continue;
        }
        let render_y = i32!(DETAIL_BORDER_OFFSET) + (y_offset + DETAIL_LOOKAROUND) * i32!(DETAIL_PIXEL_SCALE);

        for x_offset in -DETAIL_LOOKAROUND..=DETAIL_LOOKAROUND {
            let x_coord = i32!(ui_state.x) + x_offset;
            if x_coord < 0 || x_coord >= i32!(ui_state.image.width) {
                continue;
            }
            let render_x = i32!(DETAIL_BORDER_OFFSET) + (x_offset + DETAIL_LOOKAROUND) * i32!(DETAIL_PIXEL_SCALE);

            let index = usize!(y_coord) * usize!(ui_state.image.width) + usize!(x_coord);
            let color = color_for_db_bool(ui_state.image.pixels[index]);

            canvas.set_draw_color(color);
            canvas.fill_rect(Rect::new(
                render_x,
                render_y,
                DETAIL_PIXEL_SCALE,
                DETAIL_PIXEL_SCALE,
            )).unwrap();
        }
    }

    // draw the cursor in the detailed preview
    let cursor_coord = i32!(DETAIL_BORDER_OFFSET) + DETAIL_LOOKAROUND * i32!(DETAIL_PIXEL_SCALE);
    canvas.set_draw_color(COLOR_CURSOR);
    canvas.draw_rect(Rect::new(
        cursor_coord,
        cursor_coord,
        DETAIL_PIXEL_SCALE,
        DETAIL_PIXEL_SCALE,
    )).unwrap();

    // paint the full image in the bottom right
    const FULL_IMAGE_PIXEL_SCALE: u32 = 4;
    const FULL_IMAGE_BORDER_OFFSET: u32 = 4;
    for y in 0..ui_state.image.height {
        let draw_y = canvas_height - (FULL_IMAGE_BORDER_OFFSET + FULL_IMAGE_PIXEL_SCALE * (ui_state.image.height - y));
        for x in 0..ui_state.image.width {
            let draw_x = canvas_width - (FULL_IMAGE_BORDER_OFFSET + FULL_IMAGE_PIXEL_SCALE * (ui_state.image.width - x));
            
            let i = usize!(y * ui_state.image.width + x);
            let color = color_for_db_bool(ui_state.image.pixels[i]);
            canvas.set_draw_color(color);
            canvas.fill_rect(Rect::new(
                draw_x.try_into().unwrap(),
                draw_y.try_into().unwrap(),
                FULL_IMAGE_PIXEL_SCALE,
                FULL_IMAGE_BORDER_OFFSET,
            )).expect("failed to draw rectangle");
        }
    }

    // frame it
    let image_frame_x = canvas_width - (FULL_IMAGE_BORDER_OFFSET + FULL_IMAGE_PIXEL_SCALE * ui_state.image.width);
    let image_frame_y = canvas_height - (FULL_IMAGE_BORDER_OFFSET + FULL_IMAGE_PIXEL_SCALE * ui_state.image.height);
    let image_frame_width = ui_state.image.width * FULL_IMAGE_PIXEL_SCALE;
    let image_frame_height = ui_state.image.height * FULL_IMAGE_PIXEL_SCALE;
    canvas.set_draw_color(COLOR_FULL_FRAME);
    canvas.draw_rect(Rect::new(
        image_frame_x.try_into().unwrap(),
        image_frame_y.try_into().unwrap(),
        image_frame_width,
        image_frame_height,
    )).expect("failed to draw rectangle");

    // draw the cursor in the full image
    let image_cursor_x = i32!(canvas_width) - (i32!(FULL_IMAGE_BORDER_OFFSET) + i32!(FULL_IMAGE_PIXEL_SCALE) * (i32!(ui_state.image.width) - (i32!(ui_state.x) + 1 - DETAIL_LOOKAROUND)));
    let image_cursor_y = i32!(canvas_height) - (i32!(FULL_IMAGE_BORDER_OFFSET) + i32!(FULL_IMAGE_PIXEL_SCALE) * (i32!(ui_state.image.height) - (i32!(ui_state.y) + 1 - DETAIL_LOOKAROUND)));
    let image_cursor_size = FULL_IMAGE_PIXEL_SCALE * (2 * u32!(DETAIL_LOOKAROUND) + 1);
    canvas.set_draw_color(COLOR_PREVIEW_FRAME);
    canvas.draw_rect(Rect::new(
        image_cursor_x,
        image_cursor_y,
        image_cursor_size,
        image_cursor_size,
    )).expect("failed to draw rectangle");


    // paint the current color in the top right
    const CURRENT_COLOR_PIXEL_SCALE: u32 = 16;
    const CURRENT_COLOR_BORDER_OFFSET: u32 = 4;
    let current_color_x = canvas_width - (CURRENT_COLOR_BORDER_OFFSET + CURRENT_COLOR_PIXEL_SCALE);
    let current_color_y = CURRENT_COLOR_BORDER_OFFSET;
    canvas.set_draw_color(if ui_state.setting_mode { COLOR_TRUE } else { COLOR_FALSE });
    canvas.fill_rect(Rect::new(
        current_color_x.try_into().unwrap(),
        current_color_y.try_into().unwrap(),
        CURRENT_COLOR_PIXEL_SCALE,
        CURRENT_COLOR_PIXEL_SCALE,
    )).expect("failed to draw current color");

    canvas.present();
}


fn keycode_to_digit(keycode: Keycode) -> Option<u32> {
    match keycode {
        Keycode::Num0|Keycode::Kp0 => Some(0),
        Keycode::Num1|Keycode::Kp1 => Some(1),
        Keycode::Num2|Keycode::Kp2 => Some(2),
        Keycode::Num3|Keycode::Kp3 => Some(3),
        Keycode::Num4|Keycode::Kp4 => Some(4),
        Keycode::Num5|Keycode::Kp5 => Some(5),
        Keycode::Num6|Keycode::Kp6 => Some(6),
        Keycode::Num7|Keycode::Kp7 => Some(7),
        Keycode::Num8|Keycode::Kp8 => Some(8),
        Keycode::Num9|Keycode::Kp9 => Some(9),
        _ => None,
    }
}


fn main() {
    let args = Mode::parse();
    let (image_filename, image) = match &args {
        Mode::Create(create_opts) => {
            let image = Image::new(create_opts.width, create_opts.height);
            let f = File::create(&create_opts.filename)
                .expect("failed to create image file");
            serde_json::to_writer_pretty(f, &image)
                .expect("failed to serialize initial image");
            (&create_opts.filename, image)
        },
        Mode::Open(open_opts) => {
            let f = File::open(&open_opts.filename)
                .expect("failed to open image file");
            let image = serde_json::from_reader(f)
                .expect("failed to deserialize image file");
            (&open_opts.filename, image)
        },
        Mode::ToPng(to_png_opts) => {
            let image: Image = {
                let f = File::open(&to_png_opts.field_filename)
                    .expect("failed to open field image file");
                serde_json::from_reader(f)
                    .expect("failed to deserialize field image file")
            };

            {
                let f = File::create(&to_png_opts.png_filename)
                    .expect("failed to open PNG file");
                let mut png_image = png::Encoder::new(f, image.width, image.height);
                png_image.set_color(png::ColorType::Grayscale);
                png_image.set_depth(png::BitDepth::Eight);
                let mut png_writer = png_image.write_header()
                    .expect("failed to write PNG header");

                let mut pixel_data = Vec::with_capacity(image.pixels.len());
                for pixel in &image.pixels {
                    match pixel {
                        Some(true) => pixel_data.push(0xFF),
                        Some(false) => pixel_data.push(0x00),
                        None => pixel_data.push(0x7F),
                    }
                }
                png_writer.write_image_data(&pixel_data)
                    .expect("failed to write pixel data");
            }
            return;
        }
    };

    let mut ui_state = UiState::new(image);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("pixelfield", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        render(&mut canvas, &ui_state);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: keycode_opt, .. } => {
                    if let Some(keycode) = keycode_opt {
                        match keycode {
                            Keycode::Escape => break 'running,
                            Keycode::R => ui_state.going_right = !ui_state.going_right,
                            Keycode::X => ui_state.setting_mode = !ui_state.setting_mode,
                            Keycode::S => {
                                // save the current image
                                let f = File::create(image_filename)
                                    .expect("failed to create image file");
                                serde_json::to_writer_pretty(f, &ui_state.image)
                                    .expect("failed to serialize initial image");
                            },
                            Keycode::Left => {
                                if ui_state.x > 0 {
                                    ui_state.x -= 1;
                                }
                            },
                            Keycode::Right => {
                                if ui_state.x < ui_state.image.width - 1 {
                                    ui_state.x += 1;
                                }
                            },
                            Keycode::Up => {
                                if ui_state.y > 0 {
                                    ui_state.y -= 1;
                                }
                            },
                            Keycode::Down => {
                                if ui_state.y < ui_state.image.height - 1 {
                                    ui_state.y += 1;
                                }
                            },
                            Keycode::Home => {
                                ui_state.x = 0;
                                ui_state.y = 0;
                            },
                            Keycode::T => {
                                // set current pixel to true
                                let image_index = usize!(ui_state.y * ui_state.image.width + ui_state.x);
                                ui_state.image.pixels[image_index] = Some(true);
                            },
                            Keycode::F => {
                                // set current pixel to false
                                let image_index = usize!(ui_state.y * ui_state.image.width + ui_state.x);
                                ui_state.image.pixels[image_index] = Some(false);
                            },
                            Keycode::Backspace|Keycode::Delete => {
                                // set current pixel to null
                                let image_index = usize!(ui_state.y * ui_state.image.width + ui_state.x);
                                ui_state.image.pixels[image_index] = None;
                            },
                            other => {
                                if let Some(digit) = keycode_to_digit(other) {
                                    for _ in 0..digit {
                                        let image_index = usize!(ui_state.y * ui_state.image.width + ui_state.x);
                                        ui_state.image.pixels[image_index] = Some(ui_state.setting_mode);
                                        
                                        if ui_state.going_right {
                                            if ui_state.x < ui_state.image.width - 1 {
                                                ui_state.x += 1;
                                            } else {
                                                ui_state.going_right = false;
                                                if ui_state.y < ui_state.image.height - 1 {
                                                    ui_state.y += 1;
                                                }
                                            }
                                        } else {
                                            if ui_state.x > 0 {
                                                ui_state.x -= 1;
                                            } else {
                                                ui_state.going_right = true;
                                                if ui_state.y < ui_state.image.height - 1 {
                                                    ui_state.y += 1;
                                                }
                                            }
                                        }
                                    }
                                    ui_state.setting_mode = !ui_state.setting_mode;
                                }
                            },
                        }
                    }
                },
                _ => {},
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
