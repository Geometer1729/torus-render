mod trig;
mod proj;

extern crate image;

use std::f64::consts::TAU;
use image::{imageops::interpolate_bilinear, open, RgbImage};
use proj::project;

use clap::Parser;

use rayon::prelude::*;

#[derive(Parser, Debug)]
#[clap(author = "Brian Kuhns", version, about)]
/// Application configuration
struct Args {
    /// whether to be verbose
    #[arg(short = 'v')]
    verbose: bool,

    /// an optional name to greet
    #[arg(long = "lat")]
    lat: Option<f64>,

    #[arg(long = "long")]
    long: Option<f64>,

    #[arg(long = "width", short='w')]
    width: Option<u32>,

    #[arg(long = "height", short='g')]
    height: Option<u32>,

    #[arg(long = "scale", short='s')]
    scale: Option<f64>,

    #[arg(long = "map", short='m')]
    map: Option<String>,
}

fn main() {
    let args = Args::parse();
    if args.verbose {
        println!("DEBUG {args:?}");
    }

    let map_name = args.map.unwrap_or("testMap.bmp".to_string());
    let source_map = open(map_name).expect("failed to open").into_rgb8();
    let long = args.long.unwrap_or(0.0) * TAU/360.0;
    let lat = (args.lat.unwrap_or(0.0) + 90.0) * TAU/360.0;
    let width = args.width.unwrap_or(512);
    let height = args.height.unwrap_or(512);
    let scale = args.scale.unwrap_or(400.0);

    let mut img = RgbImage::new(width, height);
    img.par_enumerate_pixels_mut()
            .for_each(|(x,y,p)|
                { let (px,py) =
                    pixel_for
                    (source_map.height() as f64
                    ,source_map.width() as f64
                    ,scale
                    ,width
                    ,height
                    ,lat
                    ,long
                    ,x
                    ,y
                    );
                *p = interpolate_bilinear(&source_map,px as f32,py as f32).unwrap();
            });
    img.save("img.jpg").unwrap();
}

const TARGET_STEP : f64 = 100.0;

fn pixel_for
    (source_h : f64
    ,source_w : f64
    ,scale:f64
    ,width : u32
    ,height :u32
    ,lat : f64
    ,long : f64
    ,x : u32
    ,y :u32) -> (f64,f64) {
    let x1 = (x as f64 - (width/2) as f64)*scale as f64;
    let y1 = (y as f64 - (height/2) as f64)*scale as f64;
    let [x2,y2] = project([long,lat],[x1,y1],TARGET_STEP);
    let x3 = (x2 + TAU/2.0)/TAU*(source_w-1.0);
    let y3 = (y2 + TAU/2.0)/TAU*(source_h-1.0);
    (x3,y3)
}

