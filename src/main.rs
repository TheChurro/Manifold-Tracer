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

pub mod camera;
pub mod geometry;
pub mod integrator;
pub mod materials;

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

fn main() {}
