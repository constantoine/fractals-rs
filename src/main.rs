use sdl2::event::Event;
use sdl2::gfx::framerate::FPSManager;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::Texture;
use std::time::{Duration, Instant};

#[derive(Copy, Clone)]
struct Complex {
    pub real: f64,
    pub imaginary: f64,
}

impl Complex {
    // mod to the power of n
    pub fn modn(self, n: i32) -> f64 {
        self.real.powi(n) + self.imaginary.powi(n)
    }

    // Zn² + C
    fn compute_next(self, constant: Complex) -> Complex {
        // Real part
        let zr: f64 = self.real.powi(2) - self.imaginary.powi(2); // + self.real
                                                                  // Imaginary part
        let zi: f64 = 2.0 * self.real * self.imaginary; // + self.imaginary

        Complex {
            real: zr + constant.real,
            imaginary: zi + constant.imaginary,
        }
    }
}

fn compute_iterations(mut z: Complex, constant: Complex, max_iterations: i32) -> (Complex, i32) {
    let mut current_iteration: i32 = 0;
    while z.modn(2) < 16.0 && current_iteration < max_iterations {
        z = z.compute_next(constant);
        current_iteration += 1;
    }

    (z, current_iteration)
}

fn get_color_smooth(point: Complex, iteration: i32) -> Color {
    let size: f64 = point.modn(2);
    let smoothed: f64 = iteration as f64 - size.log2().log2() + 4.0;

    /*
    let m = point.modn(2).sqrt();
    let smoothed: f64 = iteration as f64
        - (core::cmp::max_by(1.0, m.log2(), |a, b| a.partial_cmp(b).unwrap()).log2());
    */
    /*
    Color {
        r: (0.5 + 0.5 * (3.0 + smoothed * 0.15 + 0.0).cos()* 255.0) as u8,
        g: (0.5 + 0.5 * (3.0 + smoothed * 0.15 + 0.6).cos()* 255.0) as u8 ,
        b: (0.5 + 0.5 * (3.0 + smoothed * 0.15 + 1.0).cos()* 255.0) as u8 ,
        a: 0,
    }
     */
    Color {
        r: (128.0 + 128.0 * (3.0 + smoothed * 0.15 + 0.0).cos()) as u8,
        g: (128.0 + 128.0 * (3.0 + smoothed * 0.15 + 0.6).cos()) as u8,
        b: (128.0 + 128.0 * (3.0 + smoothed * 0.15 + 1.0).cos()) as u8,
        a: 0,
    }
}

fn render(x_size: i32, y_size: i32, scale: f64, max_iterations: i32, texture: &mut Texture) {
    let scale: f64 = 1.0 / (y_size as f64 / 2.0) * scale;
    texture
        .with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..y_size {
                for x in 0..x_size {
                    let offset = y as usize * pitch + x as usize * 3;
                    let current: Complex = Complex {
                        real: (x - x_size / 2) as f64 * scale,
                        imaginary: (y - y_size / 2) as f64 * scale + 1.0,
                    };

                    let (end_point, iteration) =
                        compute_iterations(current, current, max_iterations);
                    let color;
                    if iteration == max_iterations {
                        color = Color {
                            r: 0,
                            b: 0,
                            g: 0,
                            a: 0,
                        };
                    } else {
                        color = get_color_smooth(end_point, iteration);
                    }
                    buffer[offset] = color.r;
                    buffer[offset + 1] = color.g;
                    buffer[offset + 2] = color.b;
                }
            }
        })
        .expect("texture.withlock");
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let (x_size, y_size) = (256, 256);

    let window = video_subsystem
        .window("fractal-rs", x_size, y_size)
        .position_centered()
        .build()
        .expect("Could not build window");

    let mut canvas: sdl2::render::Canvas<sdl2::video::Window> =
        window.into_canvas().software().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, x_size, y_size)
        .expect("cannot create texture");

    let mut event_pump = sdl_context
        .event_pump()
        .expect("Could not start event pump");

    let mut fps_manager: FPSManager = FPSManager::new();

    let mut scale: f64 = 1.0;
    let mut frame: i32 = 1;

    fps_manager
        .set_framerate(60)
        .expect("could not set framerate");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        if frame == 3460 {
            break 'running;
        }
        let start = Instant::now();
        canvas.clear();
        render(x_size as i32, y_size as i32, scale, 500, &mut texture);
        canvas
            .copy(&texture, None, None)
            .expect("could not copy texture");
        canvas.present();

        canvas
            .window()
            .surface(&event_pump)
            .unwrap()
            .save_bmp(format!("./{:?}.bmp", frame))
            .expect("TODO: panic message");
        let duration: Duration = start.elapsed();
        println!("Rendering time: {:?}; frame: {:?}", duration, frame);
        fps_manager.delay();

        scale *= 0.99;
        frame += 1;
    }
}
