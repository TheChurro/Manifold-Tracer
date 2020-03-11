extern crate clap;
extern crate image;
extern crate indicatif;
extern crate rand;
extern crate rand_distr;
extern crate structopt;

extern crate nalgebra as na;
extern crate ocl;

extern crate piston_window;

extern crate manifold_tracer;
mod cameras;

use crate::structopt::StructOpt;
use manifold_tracer::geometry::three_sphere::kernels::wavefront::Wavefront;
use manifold_tracer::geometry::three_sphere::scene_description::SceneDescription;
fn build_kernel(scene: &SceneDescription, width: u32, height: u32, num_samples: u32) -> Wavefront {
    let mut wavefront = Wavefront::new();
    scene.dump_to_wavefront(&mut wavefront);
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
    pub width: u32,
    pub height: u32,
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
    pub rotation_frustrum: bool,
}

impl App {
    pub fn new(
        camera: cameras::CameraS3,
        kernel: Wavefront,
        width: u32,
        height: u32,
        rotation_frustrum: bool,
    ) -> App {
        let mut window: PistonWindow =
            piston_window::WindowSettings::new("Spherical Trace", (width, height))
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
            width: width,
            height: height,
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
            rotation_frustrum: rotation_frustrum,
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
            ref mut width,
            ref mut height,
            ref mut camera,
            ref mut kernel,
            ref mut texture,
            ref mut texture_context,
            ref mut rotation_frustrum,
            ..
        } = self;
        window.draw_2d(event, |c, g, device| {
            // Load new camera data and write to texture.
            let rays = if *rotation_frustrum {
                camera.generate_rays_rotationally()
            } else {
                camera.generate_rays_frustrum()
            };
            kernel
                .run(rays, &mut camera.image)
                .unwrap_or_else(|e| panic!("Failed to trace: {}", e));
            texture
                .update(texture_context, &camera.image)
                .unwrap_or_else(|e| panic!("Failed to update texture: {}", e));
            // Draw image to screen.
            texture_context.encoder.flush(device);
            piston_window::clear([1.0; 4], g);
            let img = piston_window::Image::new()
                .rect([0.0, 0.0, *width as f64, *height as f64])
                .src_rect([
                    0.0,
                    0.0,
                    camera.image.width() as f64,
                    camera.image.height() as f64,
                ]);
            img.draw(texture, &Default::default(), c.transform, g)
        });
    }

    pub fn press(&mut self, button: piston_window::Button) {
        match button {
            Button::Keyboard(Key::W) => self.forward_speed = 10.0f32,
            Button::Keyboard(Key::S) => self.forward_speed = -10.0f32,
            Button::Keyboard(Key::A) => self.horiz_speed = -10.0f32,
            Button::Keyboard(Key::D) => self.horiz_speed = 10.0f32,
            Button::Keyboard(Key::Space) => self.vert_speed = 10.0f32,
            Button::Keyboard(Key::LShift) => self.vert_speed = -10.0f32,
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
                    self.d_theta -= (theta_change as f32) * 0.005f32;
                    self.d_phi -= (phi_change as f32) * 0.005f32;
                }
            }
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "manifold-tracer")]
struct Options {
    scene: String,
    #[structopt(short, default_value = "256")]
    width: u32,
    #[structopt(short, default_value = "256")]
    height: u32,
    #[structopt(long)]
    vwidth: Option<u32>,
    #[structopt(long)]
    vheight: Option<u32>,
    #[structopt(short, default_value = "10")]
    samples: u32,
    #[structopt(short)]
    rotation_frustrum: bool,
    #[structopt(long)]
    snapshot: Option<String>,
    #[structopt(long)]
    sequences: bool,
}

fn main() {
    use manifold_tracer::geometry::three_sphere::scene_description::*;
    let options = Options::from_args();
    let scene = SceneDescription::load(options.scene.clone()).unwrap_or_else(|e| {
        panic!(
            "Failed to load scene {} due to error {:?}",
            options.scene.clone(),
            e
        )
    });
    for warning in scene.failed_mesh_loads {
        print!("WARNING: {:?}", warning);
    }
    let scene = scene.scene;
    use cameras::CameraS3;
    let mut camera = CameraS3::new(
        std::f32::consts::PI,
        options.width,
        options.height,
        options.samples,
    );
    let mut kernel = build_kernel(&scene, options.width, options.height, options.samples);

    if options.sequences {
        for sequence in scene.sequences {
            let num_frames = (sequence.fps as f32 * sequence.angle / sequence.speed).round() as u32;
            let frame_move = sequence.angle / num_frames as f32;
            camera.theta = sequence.view_angles[0];
            camera.azimuth = sequence.view_angles[1];
            use na::{Unit, Vector3};
            let frame_disp = Unit::new_normalize(Vector3::new(sequence.move_dir[0], sequence.move_dir[1], sequence.move_dir[2])).into_inner() * frame_move;
            println!("MOVING {} per frame", frame_disp);
            std::fs::create_dir_all(&sequence.folder).expect("UNABLE TO CREATE DIRECTORIES!");
            for frame in 0..num_frames + 1 {
                let mut accumulator = vec![Vector3::zeros(); (options.width * options.height) as usize];
                for _ in 0..sequence.oversamples {
                    let rays = if options.rotation_frustrum {
                        camera.generate_rays_rotationally()
                    } else {
                        camera.generate_rays_frustrum()
                    };
                    kernel
                        .run(rays, &mut camera.image)
                        .unwrap_or_else(|e| panic!("Failed to trace: {}", e));
                    for x in 0..options.width {
                        for y in 0..options.height {
                            let pixel = camera.image.get_pixel(x, y);
                            accumulator[(x + y * options.width) as usize] += Vector3::new(pixel[0] as f32 / 255.0, pixel[1] as f32 / 255.0, pixel[2] as f32 / 255.0);
                        }
                    }
                }
                for x in 0..options.width {
                    for y in 0..options.height {
                        use image::Rgba;
                        let accum = accumulator[(x + y * options.width) as usize];
                        let color = accum / (sequence.oversamples as f32);
                        let color = Vector3::new(color.x.min(1.0).max(0.0), color.y.min(1.0).max(0.0), color.z.min(1.0).max(0.0));
                        camera.image.put_pixel(x, y, Rgba([(255.0 * color.x) as u8, (255.0 * color.y) as u8, (255.0 * color.z) as u8, 255]));
                    }
                }
                camera.image.save(format!("{}/frame{}.png", &sequence.folder, frame)).expect("FAILED TO SAVE FRAME!");
                camera.translate(frame_disp.x, frame_disp.y, frame_disp.z);
                camera.theta += sequence.angular_speed[0] / sequence.fps as f32;
                camera.azimuth += sequence.angular_speed[1] / sequence.fps as f32;
            }
        }
        return;
    }

    if let Some(filepath) = options.snapshot {
        // Load new camera data and write to texture.
        let rays = if options.rotation_frustrum {
            camera.generate_rays_rotationally()
        } else {
            camera.generate_rays_frustrum()
        };
        kernel
            .run(rays, &mut camera.image)
            .unwrap_or_else(|e| panic!("Failed to trace: {}", e));
        camera
            .image
            .save(filepath)
            .unwrap_or_else(|e| panic!("Failed to save: {}", e));
    } else {
        let mut app = App::new(
            camera,
            kernel,
            options.vwidth.unwrap_or(options.width),
            options.vheight.unwrap_or(options.height),
            options.rotation_frustrum,
        );
        app.run();
    }
}
