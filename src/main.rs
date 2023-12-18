use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::gfx::framerate::FPSManager;

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
        // Real part (Zn²)
        let zr: f64 = self.real.powi(2)  - self.imaginary.powi(2); // + self.real
        // Imaginary part (C)
        let zi: f64 = 2.0 * self.real * self.imaginary; // + self.imaginary

        Complex {
            real: zr + constant.real,
            imaginary: zi + constant.imaginary,
        }
    }
}

fn compute_iterations(mut z: Complex, constant: Complex, max_iterations: i32) -> f64 {
    let mut current_iteration: i32 = 0;
    while z.modn(2) < 4.0 && current_iteration < max_iterations {
        z = z.compute_next(constant);
        current_iteration += 1;
    }

    // Weird smoothing algorithm I found on the interwebs but not sure how it works yet.
    let m = z.modn(2).sqrt();
    return current_iteration as f64 - (core::cmp::max_by(1.0, m.log2(), |a, b| a.partial_cmp(b).unwrap()).log2())
}

fn get_color(value: i32, max_value: i32) -> Color {
    let gray: u8 = ((value * 255) / max_value).try_into().expect(&format!("value({:?}) * 255 ({:?}) larger than max_value ({:?})", value, value*255, max_value));
    Color {
        r: gray,
        g: gray,
        b: gray,
        a: 0,
    }
}

fn render(x_size: i32, y_size: i32, constant: Complex, max_iterations: i32, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    let mut colors: Vec<Vec<Point>> = vec![vec![Point::new(0, 0); y_size as usize * x_size as usize]; 256];
    canvas.clear();
    let scale: f64 = 1.0 / (y_size as f64 / 2.0);
    for y in 0..y_size {
        for x in 0..x_size {
            let current: Complex = Complex {
                real: (x - x_size / 2) as f64 * scale,
                imaginary: (y - y_size / 2) as f64* scale,
            };

            let iteration = compute_iterations(current, constant, max_iterations);
            let color = get_color(iteration as i32, max_iterations);

            canvas.set_draw_color(color);
            canvas.draw_point(Point::new(x, y)).expect("could not draw point");
        }
    }

    canvas.present();
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("DRG: The Boardgame", 900, 900)
        .position_centered()
        .build()
        .expect("Could not build window");

    let mut canvas: sdl2::render::Canvas<sdl2::video::Window> =
        window.into_canvas().build().unwrap();

    let mut event_pump = sdl_context
        .event_pump()
        .expect("Could not start event pump");

    let sets = vec![
        Complex { real: -0.8, imaginary: 0.156},
        Complex { real: -0.1528, imaginary: 1.0397}
    ];

    let mut fps_manager: FPSManager = FPSManager::new();

    let mut current_set = 0;

    fps_manager.set_framerate(24).expect("could not set framerate");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                | Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => current_set = (current_set + 1) % sets.len(),
                _ => {}
            }
        }
        let start = Instant::now();
        render(900, 900, *sets.get(current_set).expect("set not found"),50, &mut canvas);
        let duration: Duration = start.elapsed();
        println!("Rendering time: {:?}", duration);
        fps_manager.delay();
    }
}