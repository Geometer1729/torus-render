mod trig;
mod proj;

extern crate bmp;

use std::f64::consts::TAU;
use proj::project;

use clap::Parser;
use bmp::{Image, Pixel};

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
    let source_map = bmp::open(map_name).expect("failed to open");
    let long = args.long.unwrap_or(0.0) * TAU/360.0;
    let lat = (args.lat.unwrap_or(0.0) + 90.0) * TAU/360.0;
    let width = args.width.unwrap_or(256);
    let height = args.height.unwrap_or(256);
    let scale = args.scale.unwrap_or(1.0);

    let mut img = Image::new(width, height);
    let writes : Vec<(u32,u32,Pixel)> =
        img.coordinates()
            .collect::<Vec<_>>()
            .par_iter()
            .map(|&(x,y)|(x,y,pixel_for(scale,width,height,lat,long,&source_map,x,y)))
            .collect();
    for &(x,y,p) in writes.iter() {
        img.set_pixel(x,y,p);
    }
    let _ = img.save("img.bmp");

}

const TARGET_STEP : f64 = 100.0;

fn pixel_for(scale:f64,width : u32, height :u32, lat : f64, long : f64, source_map : &Image, x : u32,y :u32) -> Pixel {
    let x1 = (x as f64 - (width/2) as f64)*scale as f64;
    let y1 = (y as f64 - (height/2) as f64)*scale as f64;
    let [x2,y2] = project([long,lat],[x1,y1],TARGET_STEP);
    let x3 = (x2 + TAU/2.0)/TAU*(source_map.get_width() as f64);
    let y3 = (y2 + TAU/2.0)/TAU*(source_map.get_height() as f64);
    source_map.get_pixel(x3 as u32,y3 as u32)
}

