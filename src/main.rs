mod proj;
mod trig;

extern crate image;

use image::{imageops::interpolate_bilinear, open, Pixel, RgbImage, RgbaImage};
use proj::{project, Pt2};
use std::{collections::HashMap, f64::consts::TAU};
use vecmath::{vec2_dot, vec2_len, vec2_sub};

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

    #[arg(long = "width", short = 'w')]
    width: Option<u32>,

    #[arg(long = "height", short = 'g')]
    height: Option<u32>,

    #[arg(long = "scale", short = 's')]
    scale: Option<f64>,

    #[arg(long = "in", short = 'i')]
    input: String,

    #[arg(long = "out", short = 'o')]
    out: String,

    #[arg(long = "match", short = 'm')]
    mat: Option<String>,

    #[arg(long = "reverse", short = 'r')]
    rev: bool,
}

fn main() {
    let args = Args::parse();
    if args.verbose {
        println!("DEBUG {args:?}");
    }

    let input = args.input;
    let source_map = open(input).expect("failed to open").into_rgb8();
    let long = args.long.unwrap_or(0.0) * TAU / 360.0;
    let lat = (args.lat.unwrap_or(0.0) + 90.0) * TAU / 360.0;
    let mut width = args.width.unwrap_or(1024);
    let mut height = args.height.unwrap_or(1024);
    let scale = args.scale.unwrap_or(100.0);
    let rev = args.rev;
    let out = args.out;

    match args.mat {
        None => {}
        Some(path) => {
            let m = open(path).expect("failed to open").into_rgb8();
            width = m.width();
            height = m.height();
        }
    }

    if rev {
        let mut img = RgbaImage::new(width, height);
        let mut source = source_map.clone();
        let forward_map: HashMap<[u32; 2], [f64; 2]> = source
            .par_enumerate_pixels_mut()
            .map(|(x, y, _p)| {
                (
                    [x, y],
                    pixel_for(
                        width as f64,
                        height as f64,
                        scale,
                        source_map.width(),
                        source_map.height(),
                        lat,
                        long,
                        x,
                        y,
                    )
                    .into(),
                )
            })
            .collect();
        let dims = [width as f64, height as f64];
        for (&[px, py], &v1) in forward_map.iter() {
            let v2 = match forward_map.get(&[px + 1, py]) {
                Some(&v2) => v2,
                None => {
                    continue;
                }
            };
            let v3 = match forward_map.get(&[px, py + 1]) {
                Some(&v3) => v3,
                None => {
                    continue;
                }
            };
            for ((x, y), (sxp, syp)) in triangle_from(v1, v2, v3, dims) {
                *img.get_pixel_mut(x, y) =
                    interpolate_bilinear(&source, px as f32 + sxp, py as f32 + syp)
                        .unwrap()
                        .to_rgba();
            }
            let &v4 = forward_map.get(&[px + 1, py + 1]).unwrap();
            for ((x, y), (sxp, syp)) in triangle_from(v4, v2, v3, dims) {
                *img.get_pixel_mut(x, y) =
                    interpolate_bilinear(&source, px as f32 + 1.0 - sxp, py as f32 + 1.0 - syp)
                        .unwrap()
                        .to_rgba();
            }
        }
        img.save(out).unwrap();
    } else {
        let mut img = RgbImage::new(width, height);
        img.par_enumerate_pixels_mut().for_each(|(x, y, p)| {
            let (px, py) = pixel_for(
                source_map.width() as f64,
                source_map.height() as f64,
                scale,
                width,
                height,
                lat,
                long,
                x,
                y,
            );
            *p = interpolate_bilinear(&source_map, px as f32, py as f32).unwrap();
        });
        img.save(out).unwrap();
    }
}

fn rot_90([x, y]: [f64; 2]) -> [f64; 2] {
    [-y, x]
}

const TOLERANCE: f64 = 1.0;

fn pixel_for(
    source_w: f64,
    source_h: f64,
    scale: f64,
    width: u32,
    height: u32,
    lat: f64,
    long: f64,
    x: u32,
    y: u32,
) -> (f64, f64) {
    let x1 = (x as f64 - (width / 2) as f64) * scale as f64;
    let y1 = (y as f64 - (height / 2) as f64) * scale as f64;
    let mut step = 100.0;
    let mut v = project([long, lat], [x1, y1], step);
    let mut v_last;
    loop {
        v_last = v;
        step = step / 2.0;
        v = project([long, lat], [x1, y1], step);
        if vec2_len(vec2_sub(v, v_last)) < TOLERANCE {
            break;
        }
    }
    let [x2, y2] = v;
    let x3 = (x2 + TAU / 2.0) / TAU * (source_w - 1.0);
    let y3 = (y2 + TAU / 2.0) / TAU * (source_h - 1.0);
    (x3, y3)
}

fn triangle_from(v1: Pt2, v2_: Pt2, v3_: Pt2, dims: Pt2) -> Vec<((u32, u32), (f32, f32))> {
    let mut v2 = v2_;
    let mut v3 = v3_;
    let mut l1 = vec2_sub(v2, v1);
    let mut l2 = vec2_sub(v3, v1);

    // orient triangle
    let mut det = l1[0] * l2[1] - l2[0] * l1[1];
    let fliped = det < 0.0;
    if fliped {
        let vtemp = v2;
        v2 = v3;
        v3 = vtemp;
        let ltemp = l1;
        l1 = l2;
        l2 = ltemp;
        det = -det;
    }

    let xmax = f64::max(v1[0], f64::max(v2[0], v3[0])).round() as u32;
    let xmin = f64::min(v1[0], f64::min(v2[0], v3[0])).round() as u32;
    let ymax = f64::max(v1[1], f64::max(v2[1], v3[1])).round() as u32;
    let ymin = f64::min(v1[1], f64::min(v2[1], v3[1])).round() as u32;
    // normals and the dot product for each side
    let nv1 = rot_90(vec2_sub(v2, v1));
    let nvt1 = vec2_dot(v1, nv1);
    let nv2 = rot_90(vec2_sub(v3, v2));
    let nvt2 = vec2_dot(v2, nv2);
    let nv3 = rot_90(vec2_sub(v1, v3));
    let nvt3 = vec2_dot(v3, nv3);
    // basis for the triangle
    let b1 = [l2[1] / det, -l2[0] / det];
    let b2 = [-l1[1] / det, l1[0] / det];
    if (xmax - xmin) as f64 > dims[0] / 2.0 || (ymax - ymin) as f64 > dims[1] / 2.0 {
        //TODO handle edges somewhat reasonably
        //println!("{xmax},{xmin},{ymax},{ymin}");
        return Vec::new();
    }
    let mut ret = Vec::new();
    // TODO break early on these loops when trivial
    for x in xmin..=xmax {
        for y in ymin..=ymax {
            let v = [x as f64, y as f64];
            if vec2_dot(v, nv1) >= nvt1 && vec2_dot(v, nv2) >= nvt2 && vec2_dot(v, nv3) >= nvt3 {
                let vr = vec2_sub(v, v1);
                // relative coords in the triangle
                let pxp = vec2_dot(vr, b1) as f32;
                let pyp = vec2_dot(vr, b2) as f32;
                if pyp < 0.0 {
                    println!("{pyp}");
                }
                let pt = if fliped { (pyp, pxp) } else { (pxp, pyp) };
                ret.push(((x, y), pt));
            }
        }
    }
    return ret;
}
