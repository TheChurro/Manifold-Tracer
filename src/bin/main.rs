extern crate image;
extern crate indicatif;
extern crate rand;
extern crate rand_distr;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate clap;

extern crate nalgebra as na;
extern crate ocl;

extern crate piston_window;

extern crate manifold_tracer;
mod cameras;

arg_enum! {
    #[derive(Debug)]
    enum ChoosenScene {
        MaterialTest,
        Cornell,
        CornellHaze,
        CornellProjectiveSpace
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "manifold-tracer")]
struct Options {
    #[structopt(short, long, default_value = "200")]
    width: u32,
    #[structopt(short, long, default_value = "200")]
    height: u32,
    #[structopt(short, long, default_value = "10")]
    samples: u32,
    #[structopt(short, long, default_value = "output/test.png")]
    out_file: std::path::PathBuf,
    #[structopt(long, default_value = "MaterialTest")]
    scene: ChoosenScene,
    #[structopt(short, long)]
    update: Option<u32>,
}

use manifold_tracer::geometry::three_sphere::kernels::wavefront::Wavefront;
fn build_kernel(width: u32, height: u32, num_samples: u32) -> Wavefront {
    use manifold_tracer::geometry::three_sphere::*;
    let mut wavefront = Wavefront::new();
    let mut triangles = Vec::new();
    triangles.push(Triangle::new(Point::i(), Point::j(), Point::k()).unwrap());
    triangles.push(Triangle::new(Point::i(), Point::k(), -Point::j()).unwrap());
    triangles.push(Triangle::new(Point::i(), -Point::j(), -Point::k()).unwrap());
    triangles.push(Triangle::new(Point::i(), -Point::k(), Point::j()).unwrap());
    triangles.push(Triangle::new(Point::j(), -Point::i(), Point::k()).unwrap());
    triangles.push(Triangle::new(Point::k(), -Point::i(), -Point::j()).unwrap());
    triangles.push(Triangle::new(-Point::j(), -Point::i(), -Point::k()).unwrap());
    triangles.push(Triangle::new(-Point::k(), -Point::i(), Point::j()).unwrap());
    for tri in triangles {
        wavefront.triangle(tri, [0.75, 0.75, 0.75], MaterialType::Lambertian);
    }
    let mut balls = Vec::new();
    balls.push(Object::new(
        Ball::new(Point::i(), 0.1),
        [0.75, 0.25, 0.25],
        MaterialType::Emissive,
    ));
    balls.push(Object::new(
        Ball::new(-Point::i(), 0.1),
        [0.25, 0.75, 0.25],
        MaterialType::Emissive,
    ));
    balls.push(Object::new(
        Ball::new(Point::j(), 0.1),
        [0.25, 0.25, 0.75],
        MaterialType::Emissive,
    ));
    balls.push(Object::new(
        Ball::new(-Point::j(), 0.1),
        [0.75, 0.25, 0.75],
        MaterialType::Emissive,
    ));
    balls.push(Object::new(
        Ball::new(Point::k(), 0.1),
        [0.75, 0.75, 0.25],
        MaterialType::Emissive,
    ));
    balls.push(Object::new(
        Ball::new(-Point::k(), 0.1),
        [0.25, 0.75, 0.75],
        MaterialType::Emissive,
    ));
    // balls.push(Object::new(
    //     Ball::new(-Point::one(), 0.1),
    //     [2.0, 2.0, 2.0],
    //     MaterialType::Emissive,
    // ));
    for ball in balls {
        wavefront.ball(ball.geometry, ball.color, ball.material);
    }
    match wavefront.build(width, height, num_samples) {
        Err(e) => panic!("Failed to build kernel: {}", e),
        _ => {}
    };
    wavefront
}

use piston_window::{AdvancedWindow, PistonWindow};
use piston_window::{Button, Key, MouseRelativeEvent, PressEvent, ReleaseEvent, UpdateEvent};
use piston_window::{G2dTexture, G2dTextureContext, Texture, TextureSettings};

struct App {
    pub window: PistonWindow,
    pub texture: G2dTexture,
    pub texture_context: G2dTextureContext,
    pub camera: cameras::CameraS3,
    pub kernel: Wavefront,
    pub horiz_speed: f32,
    pub forward_speed: f32,
    pub vert_speed: f32,
    pub d_phi: f32,
    pub d_theta: f32,
    pub track_cursor: bool,
}

impl App {
    pub fn new(camera: cameras::CameraS3, kernel: Wavefront) -> App {
        let mut window: PistonWindow = piston_window::WindowSettings::new(
            "Spherical Trace",
            (camera.image.width(), camera.image.height()),
        )
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));
        window.set_capture_cursor(true);

        let mut tex_context: G2dTextureContext = window.create_texture_context();
        let tex: G2dTexture =
            Texture::from_image(&mut tex_context, &camera.image, &TextureSettings::new())
                .unwrap_or_else(|e| panic!("Failed to create texture: {}", e));
        App {
            window: window,
            texture: tex,
            texture_context: tex_context,
            camera: camera,
            kernel: kernel,
            horiz_speed: 0.0,
            vert_speed: 0.0,
            forward_speed: 0.0,
            d_phi: 0.0,
            d_theta: 0.0,
            track_cursor: true,
        }
    }

    // pub fn render<'a, A, B>(c: piston_window::Context, g: &mut piston_window::Gfx2d<'a, A, B>, device: &mut piston_window::Device) {
    //
    // }

    pub fn update(&mut self, args: piston_window::UpdateArgs) {
        self.camera.translate(
            self.horiz_speed * (args.dt as f32),
            self.vert_speed * (args.dt as f32),
            self.forward_speed * (args.dt as f32),
        );
        self.camera.rotate(self.d_theta, self.d_phi);
        self.d_theta = 0.0;
        self.d_phi = 0.0;
    }

    pub fn draw(&mut self, event: &piston_window::Event) {
        let App {
            ref mut window,
            ref mut camera,
            ref mut kernel,
            ref mut texture,
            ref mut texture_context,
            ..
        } = self;
        window.draw_2d(event, |c, g, device| {
            // Load new camera data and write to texture.
            let rays = camera.generate_rays_frustrum();
            kernel
                .run(rays, &mut camera.image)
                .unwrap_or_else(|e| panic!("Failed to trace: {}", e));
            texture
                .update(texture_context, &camera.image)
                .unwrap_or_else(|e| panic!("Failed to update texture: {}", e));
            // Draw image to screen.
            texture_context.encoder.flush(device);
            piston_window::clear([1.0; 4], g);
            piston_window::image(texture, c.transform, g);
        });
    }

    pub fn press(&mut self, button: piston_window::Button) {
        match button {
            Button::Keyboard(Key::W) => self.forward_speed = 1.0f32,
            Button::Keyboard(Key::S) => self.forward_speed = -1.0f32,
            Button::Keyboard(Key::A) => self.horiz_speed = -1.0f32,
            Button::Keyboard(Key::D) => self.horiz_speed = 1.0f32,
            Button::Keyboard(Key::Space) => self.vert_speed = 1.0f32,
            Button::Keyboard(Key::LShift) => self.vert_speed = -1.0f32,
            Button::Mouse(_) => {
                self.track_cursor = !self.track_cursor;
                self.window.set_capture_cursor(self.track_cursor);
            }
            _ => {}
        }
    }

    pub fn release(&mut self, button: piston_window::Button) {
        match button {
            Button::Keyboard(Key::W) => self.forward_speed = 0.0f32,
            Button::Keyboard(Key::S) => self.forward_speed = -0.0f32,
            Button::Keyboard(Key::A) => self.horiz_speed = -0.0f32,
            Button::Keyboard(Key::D) => self.horiz_speed = 0.0f32,
            Button::Keyboard(Key::Space) => self.vert_speed = 0.0f32,
            Button::Keyboard(Key::LShift) => self.vert_speed = 0.0f32,
            _ => {}
        }
    }

    pub fn run(&mut self) {
        while let Some(event) = self.window.next() {
            // Draws when there *is* a draw event. This conditional is handled
            // by the window.draw_2d function.
            self.draw(&event);

            if let Some(args) = event.update_args() {
                self.update(args);
            }

            if let Some(button) = event.press_args() {
                self.press(button);
            }

            if let Some(button) = event.release_args() {
                self.release(button);
            }

            if let Some([theta_change, phi_change]) = event.mouse_relative_args() {
                if self.track_cursor {
                    self.d_theta += (theta_change as f32) * 0.005f32;
                    self.d_phi += (phi_change as f32) * 0.005f32;
                }
            }
        }
    }
}

fn main() {
    use cameras::CameraS3;
    let camera = CameraS3::new(std::f32::consts::PI, 256, 256, 10);
    let kernel = build_kernel(256, 256, 10);

    let mut app = App::new(camera, kernel);
    app.run();
}
